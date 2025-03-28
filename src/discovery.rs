use serde_json::json;
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;

pub fn get_discovered_by_clients() {
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

    let msg = john.to_string();

    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");
    socket
        .set_broadcast(true)
        .expect("set_broadcast call failed");

    loop {
        socket
            .send_to(&msg.as_bytes(), "255.255.255.255:53317")
            .expect("couldn't send data");
        std::thread::sleep(Duration::from_secs(6))
    }
}

pub fn receive_connection() -> io::Result<()> {
    // Multicast group and port
    let multicast_address = Ipv4Addr::new(224, 0, 0, 167);
    let port = 53317;

    // Bind socket to all interfaces on the specified port
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port))?;
    println!("Listening on {}:{}", Ipv4Addr::UNSPECIFIED, port);

    // Join the multicast group
    socket.join_multicast_v4(&multicast_address, &Ipv4Addr::UNSPECIFIED)?;
    println!("Joined multicast group {}", multicast_address);

    let mut buffer = [0; 1024];

    loop {
        match socket.recv_from(&mut buffer) {
            Ok((size, src)) => {
                println!(
                    "Received {} bytes from {}: {}",
                    size,
                    src,
                    String::from_utf8_lossy(&buffer[..size])
                );
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
            }
        }
    }
}
