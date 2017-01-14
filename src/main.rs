#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate ipnetwork;
extern crate pnet;
extern crate rips;
extern crate tic;

use std::fmt;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::process;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use std::thread;

use tic::{Clocksource, Interest, Receiver, Sample, Sender};
use ipnetwork::Ipv4Network;
use pnet::datalink::{self, NetworkInterface};
use rips::udp::UdpSocket;

mod logging;
use logging::set_log_level;

lazy_static! {
    static ref DEFAULT_ROUTE: Ipv4Network = "0.0.0.0/0".parse().unwrap();
}

macro_rules! eprintln {
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Metric {
    Ok,
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Metric::Ok => write!(f, "ok"),
        }
    }
}

fn main() {
    set_log_level(1);
    let args = ArgumentParser::new();

    let (_, iface) = args.get_iface();
    let src_net = args.get_src_net();
    let gateway = args.get_gw();
    let src_port = args.get_src_port();
    let channel = args.create_channel();
    let src = SocketAddr::V4(SocketAddrV4::new(src_net.ip(), src_port));

    let mut stack = rips::NetworkStack::new();
    stack.add_interface(iface.clone(), channel).unwrap();
    stack.add_ipv4(&iface, src_net).unwrap();
    {
        let routing_table = stack.routing_table();
        routing_table.add_route(*DEFAULT_ROUTE, Some(gateway), iface);
    }

    // initialize a tic::Receiver to ingest stats
    let mut receiver = Receiver::configure()
        .windows(24*3600) // 1 day
        .duration(1)
        .capacity(1024)
        .http_listen("0.0.0.0:42024".to_owned())
        .build();

    receiver.add_interest(Interest::Count(Metric::Ok));

    let stack = Arc::new(Mutex::new(stack));
    let socket = UdpSocket::bind(stack, src).unwrap();
    let cs = receiver.get_clocksource();
    let sender = receiver.get_sender();
    thread::spawn(move || {
        handle(socket, cs, sender);
    });

    receiver.run();
}

fn handle(mut socket: UdpSocket, clocksource: Clocksource, stats: Sender<Metric>) {
    let response = "PONG\r\n".to_owned().into_bytes();
    let mut buffer = vec![0; 1024*2];
    loop {
        let (_, src) = socket.recv_from(&mut buffer).expect("Unable to read from socket");
        let t0 = clocksource.counter();
        let _ = socket.send_to(&response, src);
        let t1 = clocksource.counter();
        let _ = stats.send(Sample::new(t0, t1, Metric::Ok));
    }
}

struct ArgumentParser {
    app: clap::App<'static, 'static>,
    matches: clap::ArgMatches<'static>,
}

impl ArgumentParser {
    pub fn new() -> ArgumentParser {
        let app = Self::create_app();
        let matches = app.clone().get_matches();
        ArgumentParser {
            app: app,
            matches: matches,
        }
    }

    pub fn get_iface(&self) -> (NetworkInterface, rips::Interface) {
        let iface_name = self.matches.value_of("iface").unwrap();
        for iface in datalink::interfaces() {
            if iface.name == iface_name {
                if let Ok(rips_iface) = rips::convert_interface(&iface) {
                    return (iface, rips_iface);
                } else {
                    self.print_error(&format!("Interface {} can't be used with rips", iface_name));
                }
            }
        }
        self.print_error(&format!("Found no interface named {}", iface_name));
    }

    pub fn get_src_net(&self) -> Ipv4Network {
        if let Some(src_net) = self.matches.value_of("src_net") {
            match src_net.parse() {
                Ok(src_net) => src_net,
                Err(_) => self.print_error("Invalid CIDR"),
            }
        } else {
            let (iface, _) = self.get_iface();
            if let Some(ips) = iface.ips.as_ref() {
                for ip in ips {
                    if let IpAddr::V4(ip) = *ip {
                        return Ipv4Network::new(ip, 24).unwrap();
                    }
                }
            }
            self.print_error("No IPv4 to use on given interface");
        }
    }

    pub fn get_src_port(&self) -> u16 {
        let matches = &self.matches;
        value_t!(matches, "src_port", u16).unwrap()
    }

    pub fn get_gw(&self) -> Ipv4Addr {
        if let Some(gw_str) = self.matches.value_of("gw") {
            if let Ok(gw) = Ipv4Addr::from_str(gw_str) {
                gw
            } else {
                self.print_error("Unable to parse gateway ip");
            }
        } else {
            let src_net = self.get_src_net();
            if let Some(gw) = src_net.nth(1) {
                gw
            } else {
                self.print_error(&format!("Could not guess a default gateway inside {}", src_net));
            }
        }
    }

    pub fn create_channel(&self) -> rips::EthernetChannel {
        let (iface, _) = self.get_iface();
        let mut config = datalink::Config::default();
        config.write_buffer_size = 1024 * 64;
        config.read_buffer_size = 1024 * 64;
        match datalink::channel(&iface, config) {
            Ok(datalink::Channel::Ethernet(tx, rx)) => rips::EthernetChannel(tx, rx),
            _ => self.print_error(&format!("Unable to open network channel on {}", iface.name)),
        }
    }

    fn create_app() -> clap::App<'static, 'static> {
        let src_net_arg = clap::Arg::with_name("src_net")
            .long("ip")
            .value_name("CIDR")
            .help("Local IP and prefix to send from, in CIDR format. Will default to first IP on \
                   given iface and prefix 24.")
            .takes_value(true);
        let src_port_arg = clap::Arg::with_name("src_port")
            .long("sport")
            .value_name("PORT")
            .help("Local port to bind to and send from.")
            .default_value("12221");
        let gw = clap::Arg::with_name("gw")
            .long("gateway")
            .short("gw")
            .value_name("IP")
            .help("The default gateway to use if the destination is not on the local network. \
                   Must be inside the network given to --ip. Defaults to the first address in \
                   the network given to --ip")
            .takes_value(true);
        let iface_arg = clap::Arg::with_name("iface")
            .help("Network interface to use")
            .required(true)
            .index(1);

        clap::App::new("UDP ping server")
            .version(crate_version!())
            .author(crate_authors!())
            .about("A simple UDP ping server with a userspace network stack")
            .arg(src_net_arg)
            .arg(src_port_arg)
            .arg(gw)
            .arg(iface_arg)
    }

    fn print_error(&self, error: &str) -> ! {
        eprintln!("ERROR: {}\n", error);
        self.app.write_help(&mut ::std::io::stderr()).unwrap();
        eprintln!("");
        process::exit(1);
    }
}
