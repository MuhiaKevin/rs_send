use std::collections::HashMap;
use std::path::Path;
use serde::{Serialize, Deserialize};
use std::os::unix::fs::MetadataExt;

use crate::send_files::{Response, HOST};


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


fn process_directory(path: &Path, root_dir: &str, open_files: &mut Vec<OpenFolder> ) -> anyhow::Result<()> {
    let mut count = 0;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            process_directory(&path, root_dir, open_files)?;
        } else {
            if entry.path().exists() && entry.path().is_file() && !entry.path().is_symlink() {
                println!("{path:?}");
                let id = format!("this_is_id_{}", count);
                let file_size = path.metadata().unwrap().size();
           
                let file_path_str = path.to_str().unwrap();
                let something = file_path_str.split_once(root_dir).unwrap().1;
                let real_file_name =  root_dir.to_string() + something;
          
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


fn upload_folder( response_body: Response, open_files: &mut Vec<OpenFolder> ) -> anyhow::Result<()> {
    Ok(())
}


pub async fn send_folder(file_args: String) -> anyhow::Result<()> {
    let mut open_folder: Vec<OpenFolder> = vec![];
    let file_args = Path::new(&file_args);
    let root_dir: &str = file_args.file_name().unwrap().to_str().unwrap();
    process_directory(file_args, root_dir, &mut open_folder).unwrap();
    let pre_upload = PreUpload::build(&open_folder);
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
    println!("{response_body:?}");
    // let _ = upload_folder(response_body, open_folder, &client).await;

    Ok(())
}
