use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::HOST;

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub files: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct OpenFiles {
    id: String,

    #[serde(rename = "fileName")]
    file_name: String,

    #[serde(rename = "size")]
    file_size: u64,

    #[serde(rename = "fileType")]
    file_type: String,

    #[serde(skip)]
    real_file_path: String,

    #[serde(rename = "sha256")]
    file_sha256: String,

    preview: String,
}

#[derive(Debug, Serialize)]
struct PreUpload<'a> {
    info: Settings,
    files: HashMap<String, &'a OpenFiles>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Settings {
    alias: String,
    version: String,
    #[serde(rename = "deviceModel")]
    device_model: String,
    #[serde(rename = "deviceType")]
    device_type: String,
    fingerprint: String,
    port: u64,
    protocol: String,
    download: bool,
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
    let my_path = Path::new(&file_args[1]);
    let mut send_file_list: Vec<OpenFiles> = vec![];
    let path_file_args = Path::new(&file_args[1]);
    let root_dir: &str = path_file_args.file_name().unwrap().to_str().unwrap();

    if my_path.is_dir() {
        println!("This is a directory");
        process_directory(path_file_args, root_dir, &mut send_file_list).unwrap();
    } else {
        println!("This is a file or files");
        send_file_list = open_files_send(file_args).await;
    }

    println!("{send_file_list:#?}");

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

        let file_descriptor = File::open(file.real_file_path).await.unwrap();
        let stream = FramedRead::new(file_descriptor, BytesCodec::new());

        // let stream = FramedRead::new(file.file_pointer, BytesCodec::new());
        let file_body = reqwest::Body::wrap_stream(stream);

        let part = multipart::Part::stream(file_body);

        let form = reqwest::multipart::Form::new()
            .text("resourceName", "filename.filetype")
            .part("FileData", part);

        let res = client.post(url).multipart(form).send().await?;
        let status_code = res.status();
        println!(
            "Status Code: {status_code} Finsihed sending {} ",
            file.file_name
        );
    }

    Ok(())
}

async fn open_files_send(file_args: Vec<String>) -> Vec<OpenFiles> {
    let mut open_files: Vec<OpenFiles> = vec![];

    for (index, file_name) in file_args[1..].into_iter().enumerate() {
        let file_path = Path::new(&file_name);

        if file_path.exists() {
            let file_size = file_path.metadata().unwrap().size();
            // let file_sha256 = sha256::try_digest(file_path).unwrap(); FIX: This is is slow for now
            let file_sha256 = String::from("Sha1asomsdashdjhjksad");
            // let fp = File::open(file_path).await.unwrap();
            let id = format!("this_is_id_{}", index);
            let real_file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();

            open_files.push(OpenFiles {
                id,
                file_name: real_file_name,
                file_size,
                real_file_path: file_name.to_string(),
                // file_pointer: fp,
                file_sha256,
                // file_type: "text".to_string(), FIX: Change this
                file_type: "video/mp4".to_string(),
                preview: "*preview data*".to_string(),
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
                //println!("{path:?}");
                let id = format!("this_is_id_{}", count);
                let file_size = path.metadata().unwrap().size();

                let file_sha256 = String::from("Sha1asomsdashdjhjksad");
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
                    preview: "*preview data*".to_string(),
                    file_sha256,
                });

                count += 1;
            }
        }
    }
    Ok(())
}

async fn upload_folder(
    response_body: Response,
    open_files: Vec<OpenFiles>,
    client: &Client,
    absolute_path: String,
) -> anyhow::Result<()> {
    let file = Path::new(&absolute_path);
    let last_section = file.file_name().unwrap();
    let removed = absolute_path
        .as_str()
        .replace(last_section.to_str().unwrap(), "");

    for file in open_files {
        let token = response_body.files.get(&file.id).unwrap();

        let url = format!(
            "{HOST}/api/localsend/v2/upload?sessionId={}&fileId={}&token={}",
            response_body.session_id, file.id, token
        );

        let file_descriptor = File::open(file.real_file_path).await.unwrap();

        let stream = FramedRead::new(file_descriptor, BytesCodec::new());
        let file_body = reqwest::Body::wrap_stream(stream);

        let part = multipart::Part::stream(file_body);

        let form = reqwest::multipart::Form::new()
            .text("resourceName", "filename.filetype")
            .part("FileData", part);

        let res = client.post(url).multipart(form).send().await?;
        let status_code = res.status();
        println!(
            "Status Code: {status_code} Finsihed sending {} ",
            file.file_name
        );
    }
    Ok(())
}
