use serde_json::json;
use std::net::UdpSocket;
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
