// mod discovery;
mod send_files;
// mod send_folder;

use std::path::Path;

use send_files::send;
// use send_folder::send_folder;

const HOST: &'static str = "http://192.168.2.107:53317";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_args: Vec<String> = std::env::args().collect();

    if file_args.len() < 2 {
        eprintln!("Please Enter some files");
        std::process::exit(1);
    }

    send(file_args).await?;

    Ok(())
}
