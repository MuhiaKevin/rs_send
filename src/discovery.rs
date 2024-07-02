use serde_json::{json, Result};
use std::net::Ipv4Addr;
use std::net::{SocketAddr, UdpSocket};
use std::str::from_utf8;
use std::time::Duration;

pub fn udp_listen() {
    // let address = "0.0.0.0:53317";
    // let address = "224.0.0.167:53317";
    let addr1 = SocketAddr::from(([0, 0, 0, 0], 53317));
    let udp_socket = UdpSocket::bind(addr1).expect("Failed to bind socket to port 53317");

    println!("Waiting for Localsend clients to connect...");

    loop {
        let mut buf: [u8; 1024] = [0; 1024];

        let (number_of_bytes_read, src_addr) = udp_socket
            .recv_from(&mut buf)
            .expect("Failed to get message from remote ip");
        println!("Received something");

        let message = from_utf8(&buf[..number_of_bytes_read]).unwrap();

        if let SocketAddr::V4(addr) = src_addr {
            println!("RECEIVED");
            println!("{:?}, {:?}", addr, message);
        }
    }
}

fn untyped_example() -> Result<Vec<u8>> {
    let john = json!({
      "alias": "Chifu Wa kizunu",
      "version": "2.0",
      "deviceModel": "Chifus Phone",
      "deviceType": "headless",
      "fingerprint": "random string",
      "port": 53317,
      "protocol": "http",
      "download": true,
      "announce": true
    });

    let str_byteer = john.to_string();

    let in_bytes = str_byteer.as_bytes().to_vec();
    Ok(in_bytes)
}

fn send_discover_message() -> Vec<u8> {
    untyped_example().unwrap()
}

pub fn get_discovered_by_clients() {
    let msg = send_discover_message();

    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");
    socket
        .set_broadcast(true)
        .expect("set_broadcast call failed");

    loop {
        // println!("Sending broad cast message");
        // println!("{:?}", msg);
        // socket.send_to(&msg, broadcast_socket_addr).expect("couldn't send data");
        // socket.send_to(&msg, "255.255.255.255:53317").expect("couldn't send data");
        socket
            .send_to(&msg, "255.255.255.255:53317")
            .expect("couldn't send data");
        std::thread::sleep(Duration::from_secs(6))
    }
}

pub fn another_send() -> std::io::Result<()> {
    std::thread::spawn(|| get_discovered_by_clients());

    // Create a UdpSocket and bind it to the desired port
    let socket = UdpSocket::bind("0.0.0.0:53317")?;

    // Set socket options to join the multicast group 224.0.0.0/24
    let multicast_addr = Ipv4Addr::new(224, 0, 0, 1);
    let interface_addr = Ipv4Addr::new(0, 0, 0, 0); // Use default interface
    socket.join_multicast_v4(&multicast_addr, &interface_addr)?;

    let mut buf = [0; 1024];

    println!("Listening for UDP messages on port 53317...");

    loop {
        // Receive a message
        let (len, src) = socket.recv_from(&mut buf)?;
        let msg = String::from_utf8_lossy(&buf[..len]);

        println!("Received from {}: {}", src, msg);
    }
}
