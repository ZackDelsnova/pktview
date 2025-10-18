use pnet::datalink;
use pnet::packet::{ Packet, ethernet::EthernetPacket, ip::IpNextHeaderProtocols, ipv4::Ipv4Packet, tcp::TcpPacket, udp::UdpPacket };
use chrono::Local;

fn main() {
    println!("pktview - network packet visualizer ");
    println!("------------------------------------");

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

    println!("\ncapturing on interface :{}\n", interface.name);

    // create channel to receive
    let (mut tx, mut rx) = match datalink::channel(interface, Default::default()) {
        Ok(datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("unhandled channel type"),
        Err(e) => panic!("error creating datalink channel: {}", e),
    };

    loop {
        match rx.next() {
            Ok(packet) => {
                let eth = EthernetPacket::new(packet).unwrap();
                let timestamp = Local::now().format("%H:%M:%S%.3f");

                match eth.get_ethertype() {
                    pnet::packet::ethernet::EtherTypes::Ipv4 => {
                        if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                            let src = ipv4.get_source();
                            let dst = ipv4.get_destination();
                            let proto = ipv4.get_next_level_protocol();

                            print!("[{}] {} -> {} ({:?})", timestamp, src, dst, proto);

                            match proto {
                                IpNextHeaderProtocols::Tcp => {
                                    if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                                        println!(" | tcp {} -> {}", tcp.get_source(), tcp.get_destination());
                                    } else {
                                        println!();
                                    }
                                }

                                IpNextHeaderProtocols::Udp => {
                                    if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                                        println!(" | udp {} -> {}", udp.get_source(), udp.get_destination());
                                    } else {
                                        println!();
                                    }
                                }
                                _ => println!(),
                            }
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
