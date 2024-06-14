// mod discovery;
mod send_files;

use send_files::send_files;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // std::thread::spawn(|| discovery::get_discovered_by_clients());
    // //get_discovered_by_clients();
    // //discovery::udp_listen();
    // discovery::another_send().unwrap();

    let file_args: Vec<String> = std::env::args().collect();

    if file_args.len() < 2 {
        eprintln!("Please Enter some files");
        std::process::exit(1);
    }

    send_files(file_args).await?;

    Ok(())
}
