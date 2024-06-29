use std::collections::HashMap;

use axum::{
    extract::{Json, State},
    http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    http::{HeaderValue, Method},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, Router},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::send_files::{OpenFiles, Settings, Response};

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


type DB = Arc<Mutex<Vec<ReceivedFiles>>>;

// fn todo_db() -> DB {
//     Arc::new(Mutex::new(Vec::new()))
// }

pub async fn start_server() {
    let server_address = "127.0.0.1:53117".to_string();
    let db = Arc::new(Mutex::new(Vec::new()));

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let listener = TcpListener::bind(server_address).await.unwrap();

    let app = Router::new()
        .route("/health", get(health_checker_handler))
        .route("/upload", post(upload_handler))
        .route("/api/localsend/v2/prepare-upload", post(pre_upload))
        .layer(cors)
        .with_state(db);

    println!("🚀 Server started successfully on port :53117");
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
        let file_token = Uuid::new_v4();
        let file_token = file_token.to_string();

        files.insert(file.id.clone(), file_token.to_string());

        // add files to a list
        // FIX: duplicate files
        send_list.push(ReceivedFiles {
            file_id: file.id.clone(),
            file_name: file.file_name.clone(),
            file_token,
        })
    }

    let json_response = serde_json::json!(Response{
        session_id,
        files,
    });

    Ok((StatusCode::OK, Json(json_response)))
}

async fn health_checker_handler( State(db): State<DB>) -> impl IntoResponse {
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

async fn upload_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Build Simple CRUD API in Rust using Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}
