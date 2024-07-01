use std::collections::HashMap;

use axum::{
    extract::{Json, State, Query},
    http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    http::StatusCode,
    http::{HeaderValue, Method},
    response::IntoResponse,
    routing::{get, post, Router},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::send_files::{OpenFiles, Response, Settings};

#[derive(Debug, Serialize, Deserialize)]
pub struct PreUpload {
    pub info: Settings,
    pub files: HashMap<String, OpenFiles>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReceivedFiles {
    file_name: String,
    file_id: String,
    file_token: String,
}


#[derive(Debug, Deserialize, Default)]
struct QueryOptions {
    sessionId: String,
    fileId: String,
    token: String,
}

type DB = Arc<Mutex<Vec<ReceivedFiles>>>;

pub async fn start_server() {
    let server_address = "192.168.2.100:53317".to_string();
    let db = Arc::new(Mutex::new(Vec::new()));

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let listener = TcpListener::bind(server_address).await.unwrap();

    let app = Router::new()
        .route("/health", get(health_checker_handler))
        .route("/api/localsend/v2/upload", post(upload_handler))
        .route("/api/localsend/v2/prepare-upload", post(pre_upload))
        .layer(cors)
        .with_state(db);

    println!("🚀 Server started successfully on port :53317");
    axum::serve(listener, app).await.unwrap();
}

async fn pre_upload(
    State(db): State<DB>,
    Json(body): Json<PreUpload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // hashmap for file_id with corresponding file_token
    let mut files: HashMap<String, String> = HashMap::new();

    // list of files with their name, id and token
    let mut send_list = db.lock().await;

    // session_id
    let session_id = Uuid::new_v4();
    let session_id = session_id.to_string();

    for file in body.files.values() {
        // generate file token
        let file_token = Uuid::new_v4().to_string();

        files.insert(file.id.clone(), file_token.clone());

        // add files to a list
        // FIX: duplicate files
        send_list.push(ReceivedFiles {
            file_id: file.id.clone(),
            file_name: file.file_name.clone(),
            file_token,
        })
    }

    let json_response = serde_json::json!(Response { session_id, files });

    Ok((StatusCode::OK, Json(json_response)))
}

async fn health_checker_handler(State(db): State<DB>) -> impl IntoResponse {
    const MESSAGE: &str = "Downloading file...";

    // list of files with their name, id and token
    let send_list = db.lock().await;
    println!("{:#?}", send_list);

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

async fn upload_handler(
    opts: Query<QueryOptions>,
    State(db): State<DB>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut vec = db.lock().await;
    // if file_token does not match server file_token send status code 403 with message "Invalid token or IP address" 
    // Any internal problem then send status code 500 with message "Unknown error by receiver"
    // if everything good get the name of the file from the database given the file token
    // write file to disk and send status code 200 if everything goes well and if there are
    // internal problems such as lack of permission or not enough space in the disk then send
    // the status code 500 


    // get session_id, file_id and token from query params
    // if any of the above is ommited then send error message  send status code 400 with message "Missing parameters"
    let Query(opts) = opts;
    println!("{:?}", opts.sessionId);


    // if session_id does match the server session_id then send status code 403 with message "Invalid token or IP address"

    let json_response = serde_json::json!({
        "status": "success",
        "message":  "Received upload request"
    });

    Ok((StatusCode::OK, Json(json_response)))
}
