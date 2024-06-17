use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::send_files::Response;
use crate::HOST;

#[derive(Debug, Serialize)]
struct OpenFolder {
    id: String,

    #[serde(rename = "fileName")]
    file_name: String,

    #[serde(rename = "size")]
    file_size: u64,

    #[serde(rename = "fileType")]
    file_type: String,
}

#[derive(Debug, Serialize)]
pub struct PreUpload<'a> {
    info: Settings,
    files: HashMap<String, &'a OpenFolder>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
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
    pub fn build(opened_files: &'a Vec<OpenFolder>) -> Self {
        let mut foo_bar: HashMap<String, &OpenFolder> = HashMap::new();

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

fn process_directory(
    path: &Path,
    root_dir: &str,
    open_files: &mut Vec<OpenFolder>,
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

                let file_path_str = path.to_str().unwrap();
                let something = file_path_str.split_once(root_dir).unwrap().1;
                let real_file_name = root_dir.to_string() + something;

                open_files.push(OpenFolder {
                    id,
                    file_name: real_file_name.to_string(),
                    file_size,
                    file_type: "video/mp4".to_string(),
                });

                count += 1;
            }
        }
    }
    Ok(())
}

async fn upload_folder(
    response_body: Response,
    open_files: Vec<OpenFolder>,
    client: &Client,
    absolute_path: String,
) -> anyhow::Result<()> {
    let file = Path::new(&absolute_path);
    let last_section = file.file_name().unwrap();
    let removed = absolute_path
        .as_str()
        .replace(last_section.to_str().unwrap(), "");

    for file in open_files {
        let real_path = format!("{removed}{}", file.file_name);
        let token = response_body.files.get(&file.id).unwrap();
        let url = format!(
            "{HOST}/api/localsend/v2/upload?sessionId={}&fileId={}&token={}",
            response_body.session_id, file.id, token
        );
        println!("{real_path}");

        let file_descriptor = File::open(real_path).await.unwrap();

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

pub async fn send_folder(file_args: String) -> anyhow::Result<()> {
    let mut open_folder: Vec<OpenFolder> = vec![];
    let path_file_args = Path::new(&file_args);
    let root_dir: &str = path_file_args.file_name().unwrap().to_str().unwrap();
    process_directory(path_file_args, root_dir, &mut open_folder).unwrap();
    let pre_upload = PreUpload::build(&open_folder);
    let pre_upload_json = serde_json::to_string(&pre_upload)?;

    let pre_upload_url = format!("{HOST}/api/localsend/v2/prepare-upload");

    let client = Client::new();

    let response = client
        .post(pre_upload_url)
        .header("Content-Type", "application/json")
        .body(pre_upload_json)
        .send()
        .await?;

    let response_body: Response = response.json().await?;
    upload_folder(response_body, open_folder, &client, file_args).await?;

    Ok(())
}
