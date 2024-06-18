// mod discovery;
mod send_files;
mod send_folder;

use std::path::Path;

use send_files::send;
use send_folder::send_folder;

const HOST: &'static str = "http://192.168.2.107:53317";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_args: Vec<String> = std::env::args().collect();

    if file_args.len() < 2 {
        eprintln!("Please Enter some files");
        std::process::exit(1);
    }

    // let my_path = Path::new(&file_args[1]);
    //
    // if my_path.is_dir() {
    //     println!("This is a directory");
    //     send_folder(file_args[1].clone()).await?;
    // } else {
    //     println!("This is a file or files");
    //     send_files(file_args).await?;
    // }

    send(file_args).await?;


    Ok(())
}
