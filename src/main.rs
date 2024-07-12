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
            println!("Hello, {:#?}!", files.clone());
            send(files).await?;
        },
        None => {
            tokio::spawn(async { discovery::get_discovered_by_clients() });
            receive_files::start_server().await;
        },
    }
    // let file_args: Vec<String> = std::env::args().collect();
    //
    // if file_args.len() < 2 {
    //     eprintln!("Please Enter some files or folder");
    //     std::process::exit(1);
    // }
    //
    // if file_args[1] == "receive" {
    //     tokio::spawn(async { discovery::get_discovered_by_clients() });
    //     receive_files::start_server().await;
    // }
    //
    // send(file_args).await?;
    Ok(())
}
