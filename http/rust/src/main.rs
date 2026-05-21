mod handlers;
mod models;
mod db;
mod websocket;
mod config;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::net::SocketAddr;
use log::{info, error};

pub type AppState = Arc<RwLock<SharedState>>;

pub struct SharedState {
    pub config: config::AppConfig,
    pub db_path: String,
}

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    println!("Starting ShowKeyBoard Rust Server v1.56.0");
    
    // Get the exe directory, then go up 2 levels for showKeyBoard.ini only
    let exe_dir = std::env::current_exe()
        .expect("Failed to get executable path")
        .parent()
        .unwrap()
        .to_path_buf();
    let base_dir = exe_dir.parent().unwrap().parent().unwrap().to_path_buf();
    let desc_path = exe_dir.join("showKeyBoard.desc.ini");
    let user_ini_path = base_dir.join("showKeyBoard.ini");

    println!("Desc config: {:?}", desc_path);
    println!("User config: {:?}", user_ini_path);

    // Load configuration: desc.ini first, then user.ini overrides
    let config = config::AppConfig::load_with_override(&desc_path, &user_ini_path);

    // Database and runtime files are relative to exe directory
    let db_path = if config.db_path.starts_with("./") {
        exe_dir.join(config.db_path.trim_start_matches("./")).to_string_lossy().to_string()
    } else {
        config.get_db_path()
    };

    // Extract values from config before moving into shared state
    let port = config.port;
    let ui_path = config.ui_path.clone();

    println!("Port: {}", port);
    println!("UI path: {}", ui_path);
    println!("Database path: {}", db_path);
    
    // Initialize database
    if let Err(e) = db::init_database(&db_path) {
        println!("Failed to initialize database: {}", e);
    }
    
    let shared_state = Arc::new(RwLock::new(SharedState {
        config,
        db_path,
    }));
    
    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Build router
    let app = Router::new()
        // Static files
        .nest_service("/", ServeDir::new(&ui_path))
        .nest_service("/Setting", ServeDir::new(&ui_path))
        .nest_service("/Today", ServeDir::new(&ui_path))
        .nest_service("/History", ServeDir::new(&ui_path))
        .nest_service("/Statistics", ServeDir::new(&ui_path))
        .nest_service("/Export", ServeDir::new(&ui_path))
        // API routes
        .route("/getPara", post(handlers::get_para))
        .route("/setPara", post(handlers::set_para))
        .route("/exit", post(handlers::exit_server))
        .route("/data", post(handlers::upload_data))
        .route("/sendPCInfo", post(handlers::send_pc_info))
        .route("/historyData", post(handlers::history_data))
        .route("/minuteData", post(handlers::minute_data))
        .route("/optKeymap", post(handlers::opt_keymap))
        .route("/getHistoryDate", post(handlers::get_history_date))
        .route("/statData", post(handlers::stat_data))
        .route("/deleteData", post(handlers::delete_data))
        .route("/zipDownload", get(handlers::zip_download))
        .route("/zipUpload", post(handlers::zip_upload))
        .route("/version", get(handlers::version).post(handlers::version))
        .route("/getDbs", post(handlers::get_dbs))
        .route("/getAppMinute", post(handlers::get_app_minute))
        // WebSocket endpoint
        .route("/ws", get(websocket::ws_handler))
        .layer(cors)
        .with_state(shared_state);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind port");
    axum::serve(listener, app).await.expect("Server error");
}
