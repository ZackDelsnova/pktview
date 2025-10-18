mod capture;

use capture::{ start_capture, PacketInfo };
use std::sync::mpsc::channel;
use std::thread;

fn main() {
    let interfaces = pnet::datalink::interfaces();
    println!("avaliable interface: ");

    // the description is the human readable ones
    for (i, iface) in interfaces.iter().enumerate() {
        let desc = if !iface.description.is_empty() {
            &iface.description
        } else {
            "no description"
        };

        println!("{} ({})", iface.name, desc);
    }

    let iface_index = 0;
    let interface_name = &interfaces[iface_index].name;
    println!("capturing on interface: {}", interface_name);

    let (tx, rx) = channel();

    let iface_name_clone = interface_name.clone();
    thread::spawn(move || {
        start_capture(tx, &iface_name_clone);
    });

    for pkt in rx {
        println!("{:?}", pkt);
    }
}