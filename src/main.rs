mod discovery;
mod receive_files;
mod send_files;

use send_files::send;

const HOST: &'static str = "http://192.168.2.109:53317";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_args: Vec<String> = std::env::args().collect();

    if file_args.len() < 2 {
        eprintln!("Please Enter some files or folder");
        std::process::exit(1);
    }

    if file_args[1] == "receive" {
        tokio::spawn(async { discovery::get_discovered_by_clients() });
        receive_files::start_server().await;
    }

    send(file_args).await?;
    Ok(())
}
