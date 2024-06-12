use core::panic;
use std::{collections::HashMap, env};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use tokio::fs::File;
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{BytesCodec, FramedRead};


const HOST: &'static str = "http://192.168.2.101:53317";

// Preupload response
#[derive(Debug, Deserialize)]
struct Response {
    #[serde(rename = "sessionId")]
    session_id: String,
    files: HashMap<String, String>,
}


// struct for initially starting the application
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
    file_pointer: File,

    #[serde(rename = "sha256")]
    file_sha256: String,

    preview: String,
}



// For sending preupload
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


impl<'a> PreUpload<'a >  {
    fn build(opened_files: &'a Vec<OpenFiles>) -> Self {
        let mut foo_bar: HashMap<String, &OpenFiles> = HashMap::new();

        for file in opened_files {
            foo_bar.insert(file.id.clone(), file);
        }

        PreUpload {
            info: Settings {
                alias:  "Chifu wa Kizunu Files".to_string(),
                version:  "2.0".to_string(),
                device_model:  "Burner Phone".to_string(),
                device_type:  "mobile".to_string(),
                fingerprint:  "random string".to_string(),
                port:  53317,
                protocol:  "http".to_string(),
                download: true,
            },
            files: foo_bar

        }
    }
}


pub async fn send_files() {
    let somethhing = open_files_send().await;

    let pre_upload = PreUpload::build(&somethhing);
    let foo = serde_json::to_string(&pre_upload).unwrap();

    let url = format!("{HOST}/api/localsend/v2/prepare-upload");
    
    let client = reqwest::Client::new();

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(foo)
        .send()
        .await
        .unwrap();

    let response_body: Response = response.json().await.unwrap();
    upload_files(response_body, somethhing).await;
}

 async fn upload_files(response_body: Response, files: Vec<OpenFiles>) {
    let client = Client::new();

    for file in files {
        let token = response_body.files.get(&file.id).unwrap();
        println!("token for file with id {:?} is {:?}", file.id, token);
        let url =  format!("{HOST}/api/localsend/v2/upload?sessionId={}&fileId={}&token={}", response_body.session_id, file.id, token);

        // read file body stream
        let stream = FramedRead::new(file.file_pointer, BytesCodec::new());
        let file_body = reqwest::Body::wrap_stream(stream);
        
        let part = multipart::Part::stream(file_body);

        let form = reqwest::multipart::Form::new()
            .text("resourceName", "filename.filetype")
            .part("FileData", part);


        let res = client
            .post(url)
            .multipart(form)
            .send().await.unwrap();

        println!("{}", res.status());
        println!("body: \n{}", res.text().await.unwrap());
    }
}

pub async fn open_files_send() -> Vec<OpenFiles> {
    let mut open_files: Vec<OpenFiles> = vec![];

    let files: Vec<String> = env::args().collect();
    println!("{files:?}\n\n\n\n\n\n");

    if files.len() < 2 {
        panic!("Please Enter some files")
    }


    for (index, file_name) in files[1..].into_iter().enumerate() {
        let file_path = Path::new(&file_name);

        if file_path.exists() {
            let file_size = file_path.metadata().unwrap().size();
            // let file_sha256 = sha256::try_digest(file_path).unwrap(); FIX: This is is slow for now
            let file_sha256 = String::from("Sha1asomsdashdjhjksad");
            let fp = File::open(file_path).await.unwrap();
            let id = format!("this_is_id_{}", index);
            let real_file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();

            open_files.push(OpenFiles {
                id,
                file_name: real_file_name,
                file_size,
                file_pointer: fp,
                file_sha256,
                // file_type: "text".to_string(), FIX: Change this
                file_type: "video/mp4".to_string(),
                preview: "*preview data*".to_string(),
            })
        }
    }

    open_files
}
