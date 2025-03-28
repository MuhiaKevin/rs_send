mod discovery;
mod receive_files;
mod send_files;

use clap::{Parser, Subcommand};

use send_files::send;

const HOST: &'static str = "http://192.168.2.107:53317";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    files: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.files {
        Some(files) => {
            send(files).await?;
        }
        None => {
            tokio::spawn(async { discovery::receive_connection() });
            tokio::spawn(async { discovery::get_discovered_by_clients() });
            receive_files::start_server().await;
        }
    }

    Ok(())
}
