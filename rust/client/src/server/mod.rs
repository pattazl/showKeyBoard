//! 内嵌 HTTP/WS 服务端（原 showkeyboard_server 功能迁入客户端进程）。
//!
//! 与客户端在同一进程内运行：客户端主线程跑 Slint UI/输入钩子/托盘，
//! 服务端在独立线程的 tokio runtime 上跑 axum + 后台任务。
//! 客户端通过本模块暴露的同步 API 直接进程内调用，不再走 HTTP/WS 外发。

pub mod config;
pub mod db;
pub mod font;
pub mod handlers;
pub mod models;
pub mod tasks;
pub mod websocket;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::{broadcast, Notify, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use log::{info, warn};

pub type AppState = Arc<RwLock<SharedState>>;

pub struct SharedState {
    pub config: config::AppConfig,
    pub db_path: String,
    pub base_dir: String,
    /// 用户数据目录（%USERPROFILE%\Documents\ShowKeyBoard）
    pub data_dir: String,
    pub desc_ini_path: String,
    pub user_ini_path: String,
    pub key_list_path: String,
    pub ui_path: String,
    pub network_ip: Vec<String>,
    pub info_pc: serde_json::Value,
    /// WebSocket broadcast: (target_type, message)
    /// target_type: "" = web, "ahkKeyShow" = key display
    pub ws_tx: broadcast::Sender<(String, String)>,
    pub pre_data: serde_json::Value,
    pub last_minute_record: serde_json::Value,
    pub last_flag_record: serde_json::Value,
    pub shutdown_notify: Arc<Notify>,
    pub clean_notify: Arc<Notify>,
    /// 进程内配置推送：服务端 set_para 变更配置后，把 {common,dialog} 推给客户端。
    pub client_config_tx: Option<crossbeam_channel::Sender<serde_json::Value>>,
}

/// 服务端句柄：客户端其它线程据此进程内调用服务端逻辑。
pub struct ServerHandle {
    pub state: AppState,
    pub handle: tokio::runtime::Handle,
    pub ws_tx: broadcast::Sender<(String, String)>,
}

static HANDLE: std::sync::OnceLock<ServerHandle> = std::sync::OnceLock::new();

fn handle() -> Option<&'static ServerHandle> {
    HANDLE.get()
}

/// 启动内嵌服务端：同步完成 配置加载 / DB 初始化 / 端口绑定（含重试），
/// 随后在后台线程运行 tokio runtime（axum + 后台任务）。
/// 返回最终绑定的端口。`client_config_tx` 用于服务端→客户端的配置推送。
pub fn start(
    client_config_tx: crossbeam_channel::Sender<serde_json::Value>,
) -> Result<u16, String> {
    if HANDLE.get().is_some() {
        return Err("server already started".to_string());
    }
    // 与原 server main 一致：GUI 应用下 stdout/stderr 已分离，日志走 env_logger（被丢弃亦可）
    let _ = env_logger::try_init_from_env(
        env_logger::Env::default().default_filter_or("info"),
    );

    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or_else(|| "no exe dir".to_string())?
        .to_path_buf();
    let cwd = exe_dir.clone();
    // base_dir：旧数据迁移根（安装根），与旧 server 的 base_dir 语义一致
    let base_dir = exe_dir.clone();
    // 服务端资源目录：desc.ini / ui / 旧 records.db 所在。
    // 旧独立 server 的 exe 就在 httpdist/dist（其 exe_dir 即资源目录）；
    // 合并后 client exe 在安装根，资源下移到 <exe>/httpdist/dist。
    let resource_dir = {
        let cand = exe_dir.join("httpdist").join("dist");
        if cand.join("showKeyBoard.desc.ini").exists() {
            cand
        } else {
            exe_dir.clone()
        }
    };

    let data_dir = data_dir_root(&exe_dir);
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        warn!("failed to create data dir {:?}: {}", data_dir, e);
    }
    // 旧版本 dbs/backup/share 可能留在安装根或资源目录，一并迁移到用户数据目录
    for legacy_root in [&base_dir, &resource_dir] {
        migrate_legacy_dirs(legacy_root, &data_dir);
    }
    migrate_dir_if_absent(&resource_dir.join("uploads"), &data_dir.join("uploads"));

    let desc_path = resource_dir.join("showKeyBoard.desc.ini");
    let user_ini_path = data_dir.join("showKeyBoard.ini");
    if !user_ini_path.exists() {
        let legacy = config::find_config_file(&resource_dir, &cwd, "showKeyBoard.ini");
        if legacy.exists() && legacy != user_ini_path {
            let _ = std::fs::copy(&legacy, &user_ini_path);
        }
    }

    let mut config = config::AppConfig::load_with_override(&desc_path, &user_ini_path);

    let db_filename = config.db_path.trim_start_matches("./");
    let db_path = {
        let user_db = data_dir.join(db_filename);
        if user_db.exists() {
            user_db.to_string_lossy().to_string()
        } else {
            let candidates: Vec<std::path::PathBuf> = vec![
                cwd.join(db_filename),
                base_dir.join(db_filename),
                exe_dir.join(db_filename),
                resource_dir.join(db_filename),
            ];
            if let Some(legacy) = candidates.iter().find(|p| p.exists()) {
                match std::fs::copy(legacy, &user_db) {
                    Ok(_) => user_db.to_string_lossy().to_string(),
                    Err(e) => {
                        warn!("Failed to migrate records.db: {}", e);
                        legacy.to_string_lossy().to_string()
                    }
                }
            } else {
                user_db.to_string_lossy().to_string()
            }
        }
    };

    let ui_path = if std::path::Path::new(&config.ui_path).is_absolute() {
        config.ui_path.clone()
    } else {
        resource_dir.join(&config.ui_path).to_string_lossy().to_string()
    };

    db::init_database(&db_path).map_err(|e| format!("init db failed: {}", e))?;

    // 建 runtime：多线程 + 全部能力（网络/时间/阻塞任务）
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    // 端口绑定（含重试）在调用线程 block_on 完成，保证 start 返回时服务已就绪
    let (listener, port) = runtime.block_on(async {
        const MAX_PORT_RETRIES: u16 = 5;
        let initial_port = config.port;
        let mut port = initial_port;
        let listener = loop {
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
            match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => break l,
                Err(e) => {
                    let attempts = port - initial_port + 1;
                    if attempts >= MAX_PORT_RETRIES {
                        return Err(format!(
                            "bind port {}..{} failed: {}",
                            initial_port, port, e
                        ));
                    }
                    port += 1;
                }
            }
        };
        Ok::<_, String>((listener, port))
    })?;

    if port != config.port {
        config.port = port;
        config
            .common
            .insert("serverPort".to_string(), port.to_string());
        let _ = config.save(&user_ini_path);
    }

    let key_list_path = data_dir.join("KeyList.txt");
    if !key_list_path.exists() {
        for name in ["KeyList.txt", "keyList.txt"] {
            let legacy = config::find_config_file(&resource_dir, &cwd, name);
            if legacy.exists() && legacy != key_list_path {
                if std::fs::copy(&legacy, &key_list_path).is_ok() {
                    break;
                }
            }
        }
    }
    let key_list_path = key_list_path.to_string_lossy().to_string();

    let pid_file = data_dir.join("kbserver.pid");
    let _ = std::fs::write(&pid_file, std::process::id().to_string());

    let network_ip: Vec<String> = match local_ip_address::list_afinet_netifas() {
        Ok(interfaces) => interfaces
            .iter()
            .filter(|(_, ip)| ip.is_ipv4() && !ip.is_loopback())
            .map(|(_, ip)| format!("{}:{}", ip, port))
            .collect(),
        Err(_) => Vec::new(),
    };

    let (ws_tx, _) = broadcast::channel::<(String, String)>(1024);
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
        ws_tx: ws_tx.clone(),
        pre_data: serde_json::Value::Object(serde_json::Map::new()),
        last_minute_record: serde_json::Value::Null,
        last_flag_record: serde_json::Value::Null,
        shutdown_notify: shutdown_notify.clone(),
        clean_notify: Arc::new(Notify::new()),
        client_config_tx: Some(client_config_tx),
    }));

    let server_handle = ServerHandle {
        state: shared_state.clone(),
        handle: runtime.handle().clone(),
        ws_tx,
    };
    HANDLE
        .set(server_handle)
        .map_err(|_| "server already started".to_string())?;

    // 后台线程持有 runtime，跑 axum + 后台任务
    let serve_state = shared_state.clone();
    std::thread::Builder::new()
        .name("skb-server".into())
        .spawn(move || {
            runtime.block_on(async move {
                tasks::start(serve_state.clone());
                let app = build_router(serve_state.clone());
                let shutdown = serve_state
                    .read()
                    .await
                    .shutdown_notify
                    .clone();
                let _ = axum::serve(listener, app)
                    .with_graceful_shutdown(async move {
                        shutdown.notified().await;
                    })
                    .await;
            });
        })
        .map_err(|e| e.to_string())?;

    info!("ShowKeyBoard embedded server listening on http://127.0.0.1:{}", port);
    Ok(port)
}

fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let ui_path = state
        .try_read()
        .map(|s| s.ui_path.clone())
        .unwrap_or_default();

    Router::new()
        .route("/", get(websocket::root_handler))
        .nest_service("/Setting", ServeDir::new(&ui_path))
        .nest_service("/Today", ServeDir::new(&ui_path))
        .nest_service("/History", ServeDir::new(&ui_path))
        .nest_service("/Statistics", ServeDir::new(&ui_path))
        .nest_service("/Export", ServeDir::new(&ui_path))
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
        .route("/ws", get(websocket::ws_handler))
        .fallback_service(ServeDir::new(ui_path))
        .layer(cors)
        .with_state(state)
}

// ============== 对客户端暴露的同步 API（进程内调用，替代原 HTTP/WS） ==============

fn block_on<F: std::future::Future>(f: F) -> Option<F::Output> {
    let h = handle()?;
    Some(h.handle.block_on(f))
}

/// 上报按键统计数据（原 POST /data）
pub fn upload_data(data: serde_json::Value) {
    let Some(h) = handle() else { return };
    let _ = h
        .handle
        .block_on(handlers::upload_data_core(&h.state, data));
}

/// 上报屏幕/PC 信息（原 POST /sendPCInfo）
pub fn send_pc_info(req: serde_json::Value) {
    let Some(h) = handle() else { return };
    let _ = h
        .handle
        .block_on(handlers::send_pc_info_core(&h.state, req));
}

/// 客户端本地配置变更同步到服务端（原 WS ConfigUpdate）
pub fn apply_client_config(value: serde_json::Value) {
    let Some(h) = handle() else { return };
    let _ = h
        .handle
        .block_on(handlers::apply_client_config(&h.state, &value));
}

/// 实时按键文本广播给 ahkKeyShow 显示端（原 WS 发送文本帧）
pub fn broadcast_key_show(text: String) {
    let Some(h) = handle() else { return };
    let _ = h
        .ws_tx
        .send((websocket::AHK_KEY_SHOW_FLAG.to_string(), text));
}

/// 退出前把内存中 preData 落库（原 /exit 的 saveLastData 部分）
pub fn save_last_data() {
    let Some(h) = handle() else { return };
    let _ = h.handle.block_on(handlers::save_last_data(&h.state));
}

// ============== 路径 / 迁移辅助（原 server main.rs） ==============

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

fn migrate_legacy_dirs(legacy_root: &std::path::Path, data_root: &std::path::Path) {
    for sub in ["dbs", "backup", "share"] {
        migrate_dir_if_absent(&legacy_root.join(sub), &data_root.join(sub));
    }
}

fn migrate_dir_if_absent(src: &std::path::Path, dst: &std::path::Path) {
    if !src.is_dir() || dst.exists() {
        return;
    }
    copy_dir_missing(src, dst);
}

fn copy_dir_missing(src: &std::path::Path, dst: &std::path::Path) {
    if let Err(e) = std::fs::create_dir_all(dst) {
        warn!("failed to create {:?}: {}", dst, e);
        return;
    }
    let entries = match std::fs::read_dir(src) {
        Ok(e) => e,
        Err(e) => {
            warn!("failed to read {:?}: {}", src, e);
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
                warn!("failed to copy {:?}: {}", path, e);
            }
        }
    }
}
