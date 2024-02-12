use std::io;
use std::io::Write;
use bpaf::Bpaf;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{channel, Sender};
use tokio::net::TcpStream;
use tokio::task;

const MAX: u16 = 65535;
const IP_FALLBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Arguments {
    #[bpaf(long, short, fallback(IP_FALLBACK))]
    /// The address that you want to scan. Must be a valid IPv4 address. Falls back to 127.0.0.1.
    pub address: IpAddr,

    #[bpaf(
    long("start"),
    short('s'),
    fallback(1u16),
    guard(start_port_guard, "Must be greater than 0")
    )]
    pub start_port: u16,

    #[bpaf(
    long("end"),
    short('e'),
    fallback(MAX),
    guard(end_port_guard, "Must be less than or equal to 65535")
    )]
    pub end_port: u16,
}

fn start_port_guard(input: &u16) -> bool {
    *input > 0
}

fn end_port_guard(input: &u16) -> bool {
    *input <= MAX
}

async fn scan(sender: Sender<u16>, port: u16, addr: IpAddr) {
    match TcpStream::connect(format!("{}:{}", addr, port)).await {
        Ok(_) => {
            print!(".");
            io::stdout().flush().unwrap();
            sender.send(port).unwrap();
        }

        Err(_) => {}
    }
}

#[tokio::main]
async fn main() {
    let opts: Arguments = arguments().run();
    let (sender, receiver) = channel();

    print!("Scanning...");

    for i in opts.start_port..opts.end_port {
        let sender = sender.clone();
        task::spawn(async move {
            scan(sender, i, opts.address).await
        });
    }

    let mut port_numbers = vec![];
    drop(sender);

    for port_number in receiver {
        port_numbers.push(port_number);
    }

    println!();
    port_numbers.sort();

    port_numbers.iter().for_each(|port_number| {
        println!("[OPEN PORT]: {:5}", port_number);
    })
}
