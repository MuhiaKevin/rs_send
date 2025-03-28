use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

#[cfg(target_os = "unix")]
use std::os::unix::fs::MetadataExt;

use std::path::Path;
// use std::io::BufReader;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio_util::io::ReaderStream;

use crate::HOST;

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub files: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenFiles {
    pub id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "size")]
    pub file_size: u64,
    #[serde(rename = "fileType")]
    pub file_type: String,
    #[serde(skip)]
    pub real_file_path: String,
}

#[derive(Debug, Serialize)]
pub struct PreUpload<'a> {
    pub info: Settings,
    pub files: HashMap<String, &'a OpenFiles>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub alias: String,
    pub version: String,
    #[serde(rename = "deviceModel")]
    pub device_model: String,
    #[serde(rename = "deviceType")]
    pub device_type: String,
    pub fingerprint: String,
    pub port: u64,
    pub protocol: String,
    pub download: bool,
}

impl<'a> PreUpload<'a> {
    fn build(opened_files: &'a Vec<OpenFiles>) -> Self {
        let mut foo_bar: HashMap<String, &OpenFiles> = HashMap::new();

        for file in opened_files {
            foo_bar.insert(file.id.clone(), file);
        }

        PreUpload {
            info: Settings {
                alias: "Chifu wa Kizunu Files".to_string(),
                version: "2.0".to_string(),
                device_model: "Burner Phone".to_string(),
                device_type: "mobile".to_string(),
                fingerprint: "random string".to_string(),
                port: 53317,
                protocol: "http".to_string(),
                download: true,
            },
            files: foo_bar,
        }
    }
}

pub async fn send(file_args: Vec<String>) -> anyhow::Result<()> {
    let my_path = Path::new(&file_args[0]);
    let mut send_file_list: Vec<OpenFiles> = vec![];
    let path_file_args = Path::new(&file_args[0]);
    let root_dir: &str = path_file_args.file_name().unwrap().to_str().unwrap();

    if my_path.is_dir() {
        process_directory(path_file_args, root_dir, &mut send_file_list).unwrap();
    } else {
        send_file_list = open_files_send(file_args).await;
    }

    let pre_upload = PreUpload::build(&send_file_list);
    let pre_upload_json = serde_json::to_string(&pre_upload)?;

    let pre_upload_url = format!("{HOST}/api/localsend/v2/prepare-upload");

    let client = reqwest::Client::new();

    let response = client
        .post(pre_upload_url)
        .header("Content-Type", "application/json")
        .body(pre_upload_json)
        .send()
        .await?;

    let response_body: Response = response.json().await?;
    let _ = upload_files(response_body, send_file_list, &client).await;

    Ok(())
}

async fn upload_files(
    response_body: Response,
    files: Vec<OpenFiles>,
    client: &Client,
) -> anyhow::Result<()> {
    for file in files {
        let token = response_body.files.get(&file.id).unwrap();
        let url = format!(
            "{HOST}/api/localsend/v2/upload?sessionId={}&fileId={}&token={}",
            response_body.session_id, file.id, token
        );

        let file_descriptor = File::open(&file.real_file_path).await.unwrap();

        let reader = BufReader::new(file_descriptor);
        let file_len = reader.get_ref().metadata().await.unwrap().len();
        let stream = ReaderStream::new(reader);
        let body = reqwest::Body::wrap_stream(stream);

        let client = Client::new();
        let response = client
            .post(url)
            .header("Content-Length", file_len)
            .body(body)
            .send()
            .await?;

        if response.status().is_success() {
            println!("File: {} sent successfully!", file.file_name);
        } else {
            println!(
                "Failed to send file: {} with status {}",
                file.file_name,
                response.status()
            );
        }
    }

    Ok(())
}

async fn open_files_send(file_args: Vec<String>) -> Vec<OpenFiles> {
    let mut open_files: Vec<OpenFiles> = vec![];

    for (index, file_name) in file_args[1..].into_iter().enumerate() {
        let file_path = Path::new(&file_name);

        if file_path.exists() {
            let file_size = file_path.metadata().unwrap().len();
            let id = format!("this_is_id_{}", index);
            let real_file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();

            open_files.push(OpenFiles {
                id,
                file_name: real_file_name,
                file_size,
                file_type: "video/mp4".to_string(),
                real_file_path: file_name.to_string(),
            })
        }
    }

    open_files
}

// FIX: Problem getting all the files in a folder
fn process_directory(
    path: &Path,
    root_dir: &str,
    open_files: &mut Vec<OpenFiles>,
) -> anyhow::Result<()> {
    let mut count = 0;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            process_directory(&path, root_dir, open_files)?;
        } else {
            if entry.path().exists() && entry.path().is_file() && !entry.path().is_symlink() {
                let id = format!("this_is_id_{}", count);
                let file_size = path.metadata().unwrap().len();

                let file_path_str = path.to_str().unwrap();
                let real_file_path = file_path_str.to_string();
                let something = file_path_str.split_once(root_dir).unwrap().1;
                let real_file_name = root_dir.to_string() + something;

                open_files.push(OpenFiles {
                    id,
                    file_name: real_file_name.to_string(),
                    real_file_path,
                    file_size,
                    file_type: "video/mp4".to_string(),
                });

                count += 1;
            }
        }
    }
    Ok(())
}
