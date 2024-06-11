use send_files::send_files;

// mod discovery;
mod send_files;

#[tokio::main]
async fn main() {
    // std::thread::spawn(|| discovery::get_discovered_by_clients());
    // //get_discovered_by_clients();
    // //discovery::udp_listen();
    // discovery::another_send().unwrap();

    send_files().await
    // let something = send_files::open_files_send();
    // println!("{something:?}");
}
