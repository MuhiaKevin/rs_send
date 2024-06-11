use core::panic;
use std::{collections::HashMap, env};
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use serde::{Deserialize, Serialize};
// use sha256::try_digest;


// Preupload response
#[derive(Debug, Deserialize)]
struct Response {
    #[serde(rename = "sessionId")]
    session_id: String,
    files: HashMap<String, String>,
}



// for sending files after receiving the session id and file tokens
#[derive(Debug)]
struct SendFiles {
    session_id: String,
    files: Vec<FileSend>,
}

impl SendFiles {
    fn build(response: Response) -> Self {
        let mut files_to_send: Vec<FileSend> = vec![];

        for item in response.files {
            files_to_send.push(FileSend {
                file_id: item.0,
                file_token: item.1,
            });
        }

        SendFiles {
            session_id: response.session_id,
            files: files_to_send,
        }
    }
}

#[derive(Debug)]
struct FileSend {
    // file_name: String,
    // file_pointer: File,
    file_id: String,
    file_token: String,
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

    fn send_pre_upload() { }
}


pub async fn send_files() {
    let somethhing = open_files_send();

    let pre_upload = PreUpload::build(&somethhing);
    let foo = serde_json::to_string(&pre_upload).unwrap();

    // let url = "http://192.168.2.107:53317/api/localsend/v2/prepare-upload";
    let url = "http://localhost/api/localsend/v2/prepare-upload";
    
    // let json_data = r#"{
    //   "info": {
    //     "alias": "Chifu wa Kizunu Files",
    //     "version": "2.0", 
    //     "deviceModel": "Burner phone", 
    //     "deviceType": "mobile", 
    //     "fingerprint": "random string", 
    //     "port": 53317,
    //     "protocol": "http",
    //     "download": true 
    //   },
    //   "files": {
    //     "this_is_id_0": {
    //       "id": "this_is_id_0",
    //       "fileName": "Readme.md",
    //       "size": 260,
    //       "fileType": "image/jpeg",
    //       "sha256": "15dc2e55c8fe45170e732d6997d0ee1e4058d8c9d2df4a0b57f8e875e8ab994",
    //       "preview": "*preview data*" 
    //     },
    //     "this_is_id_1": {
    //       "id": "this_is_id_1",
    //       "fileName": "Cargo.toml",
    //       "size": 325,
    //       "fileType": "image/jpeg",
    //       "sha256": "7f2f888a48379a5579c053ee25b69c2beec2129f68f7067edc43ba9442400e1e",
    //       "preview": "*preview data*" 
    //     }
    //   }
    // }"#;
    
    let client = reqwest::Client::new();

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(foo)
        .send()
        .await
        .unwrap();

    let response_body: Response = response.json().await.unwrap();
    let something = SendFiles::build(response_body);
    println!("{something:?}")





    // println!("Status Code: {}", response.status());
    // let response_body = response.text().await.unwrap();
    // let response_body: serde_json::Value = response.json().await.unwrap();


    // for item in response_body.files.iter() {
    //     println!("item id: {:?}, item token: {:?}", item.0, item.1);
    // }

    //   // get session id
    //   let session_id = &response_body["sessionId"];
    //   println!("sesionId {:?}", session_id.as_str());
    //
    // // get files
    // let files = &response_body["files"];
    // println!("files {:?}", files);
}

pub fn open_files_send() -> Vec<OpenFiles> {
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
            // let file_sha256 = try_digest(file_path).unwrap(); FIX: This is is slow for now
            let file_sha256 = String::from("Sha1asomsdashdjhjksad");
            let fp = File::open(file_path).unwrap();
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
