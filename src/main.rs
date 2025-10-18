use chrono::Local;
use colored::*;
use pnet::datalink;
use pnet::packet::{ 
    Packet, 
    ethernet::{ EthernetPacket, EtherTypes },  
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    ip::IpNextHeaderProtocols,
    tcp::TcpPacket, 
    udp::UdpPacket 
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
    Arc,
};
use std::thread;
use::std::time::Duration;

fn main() -> anyhow::Result<()> {
    // ctrlc flag
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // register ctrl+c flag
    ctrlc::set_handler(move || {
        eprintln!("\nctrl+c caught - shutting down");
        r.store(false, Ordering::SeqCst);
    }).expect("error setting ctrl+c handler");


    println!("{}", "pktview - network packet visualizer ".bold().cyan());
    println!("{}", "------------------------------------".cyan());

    // get all network interface
    let interfaces = datalink::interfaces();

    println!("avaliable interface: ");
    for (index, iface) in interfaces.iter().enumerate() {
        println!("  [{}] {}", index, iface.name);
    }

    println!("\nselect interface index to capture packets: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let choice: usize = input.trim().parse().unwrap_or(0);

    let interface = interfaces.get(choice).expect("invalid interface index");

    println!("\n{} {}\n",
        "capturing on interface :{}\n".green(), interface.name.yellow());

    // create datalink channel to receive
    let (_, mut rx) = match datalink::channel(interface, Default::default()) {
        Ok(datalink::Channel::Ethernet(_tx, rx)) => (_tx, rx),
        Ok(_) => panic!("unhandled channel type"),
        Err(e) => panic!("error creating datalink channel: {}", e),
    };

    // mpsc channel to send pkt to main thread
    let (pkt_tx, pkt_rx) = mpsc::channel::<Vec<u8>>();

    // reader thread - reads pkt and forward raw bytes
    let reader_running = running.clone();
    thread::spawn(move || {
        while reader_running.load(Ordering::SeqCst) {
            match rx.next() {
                Ok(packet) => {
                    // copy pkt by bytes into owned vec and send
                    let vec = packet.to_vec();
                    // ignore send errors
                    let _ = pkt_tx.send(vec);
                }
                Err(e) => {
                    // read error - print and sleep, avoid busy loop
                    eprintln!("read error: {} -- slepping briefly",e);
                    thread::sleep(Duration::from_millis(200));
                }
            }
        }
        // reader thread exits
    });
    
    // main process loop
    while running.load(Ordering::SeqCst) {
        // wait for a packet with timeout
        match pkt_rx.recv_timeout(Duration::from_millis(500)) {
            Ok(bytes) => {
                if let Some(eth) = EthernetPacket::new(&bytes) {
                    let ts = Local::now().format("%H:%M:%S%.3f").to_string();
                    match eth.get_ethertype() {
                        EtherTypes::Ipv4 => {
                            if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                                let src = ipv4.get_source();
                                let dst = ipv4.get_destination();
                                let proto = ipv4.get_next_level_protocol();
                                match proto {
                                    IpNextHeaderProtocols::Tcp => {
                                        if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                                            println!(
                                                "{} {}:{} {} {}:{} {}",
                                                format!("[{}]", ts).dimmed(),
                                                src.to_string().blue(),
                                                tcp.get_source().to_string().bold(),
                                                "→".bright_black(),
                                                dst.to_string().red(),
                                                tcp.get_destination().to_string().bold(),
                                                "[TCP]".green()
                                            );
                                        }
                                    }
                                    IpNextHeaderProtocols::Udp => {
                                        if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                                            println!(
                                                "{} {}:{} {} {}:{} {}",
                                                format!("[{}]", ts).dimmed(),
                                                src.to_string().blue(),
                                                udp.get_source().to_string().bold(),
                                                "→".bright_black(),
                                                dst.to_string().red(),
                                                udp.get_destination().to_string().bold(),
                                                "[UDP]".yellow()
                                            );
                                        }
                                    }
                                    _ => {
                                        println!(
                                            "{} {} {} {}",
                                            format!("[{}]", ts).dimmed(),
                                            src.to_string().blue(),
                                            "→".bright_black(),
                                            dst.to_string().red()
                                        );
                                    }
                                }
                            }
                        }
                        EtherTypes::Ipv6 => {
                            if let Some(ipv6) = Ipv6Packet::new(eth.payload()) {
                                let src = ipv6.get_source();
                                let dst = ipv6.get_destination();
                                println!(
                                    "{} {} {} {} {}",
                                    format!("[{}]", ts).dimmed(),
                                    src.to_string().blue(),
                                    "→".bright_black(),
                                    dst.to_string().red(),
                                    "[IPv6]".purple()
                                );
                            }
                        }
                        _ => {
                            // ignore other ethertypes for now
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // timeout — loop back and check running flag again
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // reader thread closed the channel — exit
                break;
            }
        }
    }

    println!("{}", "Shutting down pktview...".cyan());
    Ok(())
}