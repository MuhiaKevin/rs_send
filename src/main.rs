// mod discovery;
mod send_files;

use send_files::send_files;

#[tokio::main]
async fn main() {
    // std::thread::spawn(|| discovery::get_discovered_by_clients());
    // //get_discovered_by_clients();
    // //discovery::udp_listen();
    // discovery::another_send().unwrap();

    send_files().await
}
