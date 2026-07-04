mod handlers;
mod models;
mod db;
mod websocket;
mod config;
pub mod font;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::net::SocketAddr;
use log::info;

pub type AppState = Arc<RwLock<SharedState>>;

pub struct SharedState {
    pub config: config::AppConfig,
    pub db_path: String,
    pub base_dir: String,
    pub desc_ini_path: String,
    pub user_ini_path: String,
    pub key_list_path: String,
    pub network_ip: Vec<String>,
    pub info_pc: serde_json::Value,
}

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    println!("Starting ShowKeyBoard Rust Server v1.56.0");
    
    // Get the exe directory
    let exe_dir = std::env::current_exe()
        .expect("Failed to get executable path")
        .parent()
        .unwrap()
        .to_path_buf();
    let cwd = std::env::current_dir().unwrap_or_else(|_| exe_dir.clone());
    // base_dir = exe_dir/../.. — used for dbs/ directory resolution (matching Node.js basePath)
    let base_dir = exe_dir.parent().and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| cwd.clone());

    // desc.ini is always relative to the exe directory (bundled with binary)
    let desc_path = exe_dir.join("showKeyBoard.desc.ini");
    // User config: smart path search (exe_dir → cwd → exe_dir/../..)
    let user_ini_path = config::find_config_file(&exe_dir, &cwd, "showKeyBoard.ini");

    println!("Desc config: {:?}", desc_path);
    println!("User config: {:?}", user_ini_path);

    // Load configuration: desc.ini first, then user.ini overrides
    let mut config = config::AppConfig::load_with_override(&desc_path, &user_ini_path);

    // Database: prefer cwd → parent2 → exe_dir (avoid empty db in target/debug/)
    // Node.js: dbsPath = basePath/dbs/, so also check dbs/ subdirectory
    let db_filename = config.db_path.trim_start_matches("./");
    let db_path = {
        // Order matters: cwd first (cargo run from rust/), then base_dir, then exe_dir fallback
        let candidates: Vec<std::path::PathBuf> = vec![
            cwd.join(db_filename),
            cwd.join("dbs").join(db_filename),
        ].into_iter().chain(
            [base_dir.join(db_filename), base_dir.join("dbs").join(db_filename)].into_iter()
        ).chain(
            vec![
                exe_dir.join(db_filename),
                exe_dir.join("dbs").join(db_filename),
            ]
        ).collect();
        
        let found = candidates.iter().find(|p| p.exists()).cloned();
        match found {
            Some(path) => {
                info!("Database found at: {}", path.display());
                path.to_string_lossy().to_string()
            }
            None => {
                let fallback = cwd.join(db_filename);
                info!("Database not found, creating at: {}", fallback.display());
                fallback.to_string_lossy().to_string()
            }
        }
    };

    // Extract values from config before moving into shared state
    // Resolve ui_path relative to exe_dir (matching Node.js: everything is relative to __dirname)
    let ui_path = if std::path::Path::new(&config.ui_path).is_absolute() {
        config.ui_path.clone()
    } else {
        exe_dir.join(&config.ui_path).to_string_lossy().to_string()
    };
    let mut port = config.port;

    println!("Initial port from config: {}", port);
    println!("UI path: {}", ui_path);
    println!("Database path: {}", db_path);
    
    // Initialize database
    if let Err(e) = db::init_database(&db_path) {
        println!("Failed to initialize database: {}", e);
    }
    
    // Try to bind with port retry: config.port → config.port+4 (max 5 attempts)
    const MAX_PORT_RETRIES: u16 = 5;
    let initial_port = port;
    let listener = loop {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("Server bound to port {}", port);
                break listener;
            }
            Err(e) => {
                let attempts = port - initial_port + 1;
                if attempts >= MAX_PORT_RETRIES {
                    panic!(
                        "Failed to bind any port from {} to {} ({} attempts): {}",
                        initial_port, port, attempts, e
                    );
                }
                println!(
                    "Port {} is in use, retrying with port {} (attempt {}/{})...",
                    port,
                    port + 1,
                    attempts + 1,
                    MAX_PORT_RETRIES
                );
                port += 1;
            }
        }
    };
    
    // If port changed, save the new port to showKeyBoard.ini
    if port != initial_port {
        config.port = port;
        config.common.insert("serverPort".to_string(), port.to_string());
        if let Err(e) = config.save(&user_ini_path) {
            println!(
                "Warning: failed to save updated port {} to {:?}: {}",
                port, user_ini_path, e
            );
        } else {
            println!("Saved updated port {} to {:?}", port, user_ini_path);
        }
    }
    
    // Read keyList.txt path (same directory as user ini)
    let key_list_path = config::find_config_file(&exe_dir, &cwd, "keyList.txt")
        .to_string_lossy().to_string();
    
    // Get local network IPs (uses final port after retry)
    let network_ip: Vec<String> = match local_ip_address::list_afinet_netifas() {
        Ok(interfaces) => interfaces
            .iter()
            .filter(|(_, ip)| {
                // Only IPv4, exclude 127.0.0.1/8 loopback
                ip.is_ipv4() && !ip.is_loopback()
            })
            .map(|(_, ip)| format!("{}:{}", ip, port))
            .collect(),
        Err(e) => {
            println!("Failed to list network interfaces: {}", e);
            Vec::new()
        }
    };
    println!("Network IPs: {:?}", network_ip);
    
    let shared_state = Arc::new(RwLock::new(SharedState {
        config,
        db_path,
        base_dir: base_dir.to_string_lossy().to_string(),
        desc_ini_path: desc_path.to_string_lossy().to_string(),
        user_ini_path: user_ini_path.to_string_lossy().to_string(),
        key_list_path,
        network_ip,
        info_pc: serde_json::Value::Object(serde_json::Map::new()),
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
    
    axum::serve(listener, app).await.expect("Server error");
}
