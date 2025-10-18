use pnet::datalink::{self, Channel, };
use std::sync::mpsc::Sender;
use std::net::IpAddr;

// struct to represent packet info
#[derive(Debug, Clone)]
pub struct PacketInfo {
    pub src: IpAddr,
    pub dst: IpAddr,
    pub proto: String,
    pub info: String,
}

// start capture on given interface name, send pktinfo through channel
pub fn start_capture(tx: Sender<PacketInfo>, interface_name: &str) {
    
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter().find(|iface| iface.name == interface_name).expect("interface not found");

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(_t, rx)) => (_t, rx),
        Ok(_) => panic!("unhandled channel type"),
        Err(e) => panic!("error creating datalink channel: {}", e),
    };

    // packet capture loop
    loop {
        match rx.next() {
            Ok(pkt) => {
                let pkt_info = PacketInfo{
                    src: "0.0.0.0".parse().unwrap(),
                    dst: "0.0.0.0".parse().unwrap(),
                    proto: "UNKNOWN".to_string(),
                    info: format!("raw packet length: {}", pkt.len()),
                };

                if tx.send(pkt_info).is_err() {
                    break;
                }
            }
            Err(_) => continue,
        }
    }
}

