mod handlers;
mod models;
mod db;
mod websocket;
mod config;
mod tasks;
pub mod font;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, Notify};
use std::net::SocketAddr;
use log::info;

pub type AppState = Arc<RwLock<SharedState>>;

pub struct SharedState {
    pub config: config::AppConfig,
    pub db_path: String,
    pub base_dir: String,
    /// 用户数据目录（%USERPROFILE%\Documents\ShowKeyBoard），存放 dbs/ backup/ share/ uploads/ updateTime.txt
    pub data_dir: String,
    pub desc_ini_path: String,
    pub user_ini_path: String,
    pub key_list_path: String,
    pub ui_path: String,
    pub network_ip: Vec<String>,
    pub info_pc: serde_json::Value,
    /// WebSocket broadcast: (target_type, message)
    /// target_type: "" = web, "ahkClient", "ahkKeyShow"
    pub ws_tx: broadcast::Sender<(String, String)>,
    /// Latest key data (for preData sent to web clients) — matches Node.js preData
    pub pre_data: serde_json::Value,
    /// Last minute record kept for merging (matches Node.js lastMinuteRecords)
    pub last_minute_record: serde_json::Value,
    /// Pending last record with LastFlag=1 (matches Node.js lastFlagRecord)
    pub last_flag_record: serde_json::Value,
    /// Graceful shutdown signal (matches Node.js exitFun)
    pub shutdown_notify: Arc<Notify>,
    /// 触发 doCleanData 数据整理（Node: 插入数据后 setTimeout(doCleanData, 3000)）
    pub clean_notify: Arc<Notify>,
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
    // base_dir = exe_dir/../..（安装根目录）：仅用于旧数据迁移、数据库回退查找、shareDbExec 的 cwd
    let base_dir = exe_dir.parent().and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| cwd.clone());

    // 用户数据目录：dbs / backup / share / uploads / updateTime.txt 均存放于此，不再写入安装目录
    let data_dir = data_dir_root(&exe_dir);
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        println!("Warning: failed to create data dir {:?}: {}", data_dir, e);
    }
    println!("Data dir: {:?}", data_dir);
    // 旧版本数据（安装目录下的 dbs/backup/share）首次运行时自动迁移
    migrate_legacy_dirs(&base_dir, &data_dir);
    // uploads 旧位置在服务进程工作目录（安装目录 httpdist/dist）
    migrate_dir_if_absent(&exe_dir.join("uploads"), &data_dir.join("uploads"));

    // desc.ini is always relative to the exe directory (bundled with binary)
    let desc_path = exe_dir.join("showKeyBoard.desc.ini");
    // 用户配置 showKeyBoard.ini 位于用户数据目录；首次运行时从安装目录迁移旧配置
    let user_ini_path = data_dir.join("showKeyBoard.ini");
    if !user_ini_path.exists() {
        let legacy = config::find_config_file(&exe_dir, &cwd, "showKeyBoard.ini");
        if legacy.exists() && legacy != user_ini_path {
            match std::fs::copy(&legacy, &user_ini_path) {
                Ok(_) => println!("Migrated config {:?} -> {:?}", legacy, user_ini_path),
                Err(e) => println!("Warning: failed to migrate config: {}", e),
            }
        }
    }

    println!("Desc config: {:?}", desc_path);
    println!("User config: {:?}", user_ini_path);

    // Load configuration: desc.ini first, then user.ini overrides
    let mut config = config::AppConfig::load_with_override(&desc_path, &user_ini_path);

    // Database: prefer user data dir → cwd → parent2 → exe_dir (avoid empty db in target/debug/)
    // 外部数据库目录 dbs/ 现在位于用户数据目录下
    let db_filename = config.db_path.trim_start_matches("./");
    let db_path = {
        let candidates: Vec<std::path::PathBuf> = vec![
            data_dir.join("dbs").join(db_filename),
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
    // keyList.txt 位于用户数据目录；首次运行时从安装目录迁移旧文件
    let key_list_path = data_dir.join("keyList.txt");
    if !key_list_path.exists() {
        let legacy = config::find_config_file(&exe_dir, &cwd, "keyList.txt");
        if legacy.exists() && legacy != key_list_path {
            let _ = std::fs::copy(&legacy, &key_list_path);
        }
    }
    let key_list_path = key_list_path.to_string_lossy().to_string();
    
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
    
    // Create WebSocket broadcast channel (capacity 1024)
    let (ws_tx, _) = broadcast::channel::<(String, String)>(1024);
    // Graceful shutdown signal
    let shutdown_notify = Arc::new(Notify::new());
    
    let shared_state = Arc::new(RwLock::new(SharedState {
        config,
        db_path,
        base_dir: base_dir.to_string_lossy().to_string(),
        data_dir: data_dir.to_string_lossy().to_string(),
        desc_ini_path: desc_path.to_string_lossy().to_string(),
        user_ini_path: user_ini_path.to_string_lossy().to_string(),
        key_list_path,
        ui_path: ui_path.clone(),
        network_ip,
        info_pc: serde_json::Value::Object(serde_json::Map::new()),
        ws_tx,
        pre_data: serde_json::Value::Object(serde_json::Map::new()),
        last_minute_record: serde_json::Value::Null,
        last_flag_record: serde_json::Value::Null,
        shutdown_notify: shutdown_notify.clone(),
        clean_notify: Arc::new(Notify::new()),
    }));
    
    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Build router
    let app = Router::new()
        // Root: WebSocket upgrade or serve index.html
        .route("/", get(websocket::root_handler))
        // Static files (sub-paths)
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
        // WebSocket endpoint (explicit path)
        .route("/ws", get(websocket::ws_handler))
        // Fallback: serve static files (JS, CSS, images, etc.) for all unmatched paths
        .fallback_service(ServeDir::new(&ui_path))
        .layer(cors)
        .with_state(shared_state.clone());
    
    println!("⚡ WebSocket server ready at ws://127.0.0.1:{}/", port);
    println!("🚀 Server listening on http://127.0.0.1:{}", port);

    // 启动后台定时任务：doCleanData 数据整理 / autoSave2Db 落库 / autoShare 共享库
    tasks::start(shared_state.clone());

    axum::serve(listener, app)
        .with_graceful_shutdown(async move { shutdown_notify.notified().await })
        .await
        .expect("Server error");
}

/// 用户数据根目录：`%USERPROFILE%\Documents\ShowKeyBoard`
/// （macOS/Linux 为 `$HOME/Documents/ShowKeyBoard`；无法确定时退回 exe 所在目录）
fn data_dir_root(fallback: &std::path::Path) -> std::path::PathBuf {
    let home = if cfg!(windows) {
        std::env::var_os("USERPROFILE")
    } else {
        std::env::var_os("HOME")
    };
    match home {
        Some(h) => std::path::PathBuf::from(h)
            .join("Documents")
            .join("ShowKeyBoard"),
        None => fallback.join("ShowKeyBoard"),
    }
}

/// 首次运行时把旧版本留在程序目录下的 dbs/backup/share 迁移到用户数据目录。
/// 仅当目标目录尚不存在时迁移（复制，不删除旧文件），避免每次启动重复回填已删除的数据。
fn migrate_legacy_dirs(legacy_root: &std::path::Path, data_root: &std::path::Path) {
    for sub in ["dbs", "backup", "share"] {
        migrate_dir_if_absent(&legacy_root.join(sub), &data_root.join(sub));
    }
}

/// 目标目录不存在时递归复制整个目录
fn migrate_dir_if_absent(src: &std::path::Path, dst: &std::path::Path) {
    if !src.is_dir() || dst.exists() {
        return;
    }
    println!("Migrating {:?} -> {:?}", src, dst);
    copy_dir_missing(src, dst);
}

/// 递归复制目录，已存在的目标文件不覆盖
fn copy_dir_missing(src: &std::path::Path, dst: &std::path::Path) {
    if let Err(e) = std::fs::create_dir_all(dst) {
        println!("Warning: failed to create {:?}: {}", dst, e);
        return;
    }
    let entries = match std::fs::read_dir(src) {
        Ok(e) => e,
        Err(e) => {
            println!("Warning: failed to read {:?}: {}", src, e);
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_missing(&path, &target);
        } else if path.is_file() && !target.exists() {
            if let Err(e) = std::fs::copy(&path, &target) {
                println!("Warning: failed to copy {:?}: {}", path, e);
            }
        }
    }
}
