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
use chrono::Local;
use colored::*;

fn main() {
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

    // create channel to receive
    let (_tx, mut rx) = match datalink::channel(interface, Default::default()) {
        Ok(datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("unhandled channel type"),
        Err(e) => panic!("error creating datalink channel: {}", e),
    };

    loop {
        match rx.next() {
            Ok(packet) => {
                let eth = EthernetPacket::new(packet).unwrap();
                let ts = Local::now().format("%H:%M:%S%.3f");

                match eth.get_ethertype() {
                    EtherTypes::Ipv4 => {
                        if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                            let src = ipv4.get_source();
                            let dst = ipv4.get_destination();
                            let proto = ipv4.get_next_level_protocol();

                            let prefix = format!("[{}]", ts).dimmed();

                            match proto {
                                IpNextHeaderProtocols::Tcp => {
                                    if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                                        println!(
                                            "{} {}:{} {} {}:{} {}",
                                            prefix,
                                            src.to_string().blue(),
                                            tcp.get_source().to_string().bold(),
                                            "->".bright_black(),
                                            dst.to_string().red(),
                                            tcp.get_destination().to_string().bold(),
                                            "[TCP]".green()
                                        );
                                    } else {
                                        println!();
                                    }
                                }

                                IpNextHeaderProtocols::Udp => {
                                    if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                                        println!(
                                            "{} {}:{} {} {}:{} {}",
                                            prefix,
                                            src.to_string().blue(),
                                            udp.get_source().to_string().bold(),
                                            "->".bright_black(),
                                            dst.to_string().red(),
                                            udp.get_destination().to_string().bold(),
                                            "[UDP]".green()
                                        );
                                    } else {
                                        println!();
                                    }
                                }
                                _ => println!(
                                    "{} {} {} {} {}",
                                    prefix,
                                    src.to_string().blue(),
                                    "->".bright_black(),
                                    dst.to_string().red(),
                                    "[IPv4 OTHER]".dimmed()
                                ),
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
                                "->".bright_black(),
                                dst.to_string().red(),
                                "[IPv6]".purple()
                            )
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("error reading packet: {}", e);
            }
        }
    }
}
