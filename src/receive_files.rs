use std::collections::HashMap;

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Json, Query, State},
    http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    http::StatusCode,
    http::{HeaderValue, Method},
    response::IntoResponse,
    routing::{get, post, Router},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{fs::File, io::Write};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use uuid::Uuid;

use crate::send_files::{OpenFiles, Response, Settings};

#[derive(Debug, Serialize, Deserialize)]
pub struct PreUpload {
    pub info: Settings,
    pub files: HashMap<String, OpenFiles>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReceivedFiles {
    sessionId: String,
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

type DB = Arc<Mutex<HashMap<String, ReceivedFiles>>>;

pub async fn start_server() {
    let server_address = "192.168.2.100:53317".to_string();
    let db = Arc::new(Mutex::new(HashMap::new()));

    let cors = CorsLayer::new()
        // .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let listener = TcpListener::bind(server_address).await.unwrap();

    let app = Router::new()
        .route("/health", get(health_checker_handler))
        .route("/api/localsend/v2/register", post(register))
        .route("/api/localsend/v2/prepare-upload", post(pre_upload))
        .route("/api/localsend/v2/upload", post(upload_handler))
        .layer(cors)
        // disable limit of files
        .layer(DefaultBodyLimit::disable())
        // SET UP THE MAXIMUM SIZE OF FILE ACCEPTED
        // .layer(RequestBodyLimitLayer::new(
        //     250 * 1024 * 1024, /* 250mb */
        // ))
        .with_state(db);

    println!("🚀 Server started successfully on port :53317");
    axum::serve(listener, app).await.unwrap();
}

async fn register(
    Json(body): Json<Settings>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("client sent this {body:#?}");

    let json_response = serde_json::json!({
        "message": "Client info received",
    });
    Ok((StatusCode::OK, Json(json_response)))
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
        send_list.insert(
            file_token.clone(),
            ReceivedFiles {
                sessionId: session_id.clone(),
                file_id: file.id.clone(),
                file_name: file.file_name.clone(),
                file_token,
            },
        );
    }

    let json_response = serde_json::json!(Response { session_id, files });

    Ok((StatusCode::OK, Json(json_response)))
}

async fn upload_handler(
    opts: Query<QueryOptions>,
    State(db): State<DB>,
    bytes: Bytes,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let received_files_database = db.lock().await;

    if received_files_database.len() == 0 {
        let json_response = serde_json::json!({
            "message": "Invalid token or IP address",
            "message2" : "Send Preupload first",
        });
        return Ok((StatusCode::FORBIDDEN, Json(json_response)));
    }

    let Query(opts) = opts;

    if opts.sessionId != received_files_database.get(&opts.token).unwrap().sessionId {
        let json_response = serde_json::json!({
            "message": "Invalid token or IP address",
            "message2" : "Invalid Session id",
        });
        return Ok((StatusCode::FORBIDDEN, Json(json_response)));
    }

    if let Some(received_file) = received_files_database.get(&opts.token) {
        println!("Downloading file: {}", received_file.file_name);
        let file_path = format!("/tmp/rs_send_uploads/{}", received_file.file_name);
        let mut file = File::create(file_path).unwrap();

        task::spawn_blocking(move || {
            file.write_all(&bytes).expect("Failed to write data");
        })
        .await
        .unwrap();
    } else {
        let json_response = serde_json::json!({
            "message": "Invalid token or IP address",
            "message2" : "Wrong file token",
        });
        return Ok((StatusCode::FORBIDDEN, Json(json_response)));
    }

    let json_response = serde_json::json!({
        "status": "success",
        "message":  "Received upload request"
    });

    Ok((StatusCode::OK, Json(json_response)))
}

async fn health_checker_handler(State(db): State<DB>) -> impl IntoResponse {
    const MESSAGE: &str = "Downloading file...";

    let send_list = db.lock().await;
    println!("{:#?}", send_list);

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}
