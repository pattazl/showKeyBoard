use axum::{
    extract::{State, Multipart},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::info;
use std::io::Write;
use crate::{AppState, SharedState};
use crate::models::*;
use crate::db;

async fn get_db_path(state: &AppState) -> String {
    state.read().await.db_path.clone()
}

pub async fn get_para(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    info!("GET /getPara");
    Json(serde_json::json!({
        "success": true,
        "data": {}
    }))
}

pub async fn set_para(
    State(_state): State<AppState>,
    Json(req): Json<SetParaRequest>,
) -> impl IntoResponse {
    info!("POST /setPara - key: {:?}, value: {:?}", req.key, req.value);
    Json(serde_json::json!({
        "success": true
    }))
}

pub async fn exit_server() -> impl IntoResponse {
    info!("POST /exit - Server shutdown requested");
    Json(serde_json::json!({
        "success": true,
        "message": "Server exiting"
    }))
}

pub async fn upload_data(
    State(_state): State<AppState>,
    Json(req): Json<DataRequest>,
) -> impl IntoResponse {
    info!("POST /data - app: {:?}, type: {:?}", req.app_name, req.data_type);
    
    let db_path = get_db_path(&_state).await;
    
    let record = KeyRecord {
        id: None,
        key: req.data.as_ref().and_then(|d| d.get("key").and_then(|k| k.as_str().map(String::from))),
        app: req.app_name,
        tick: req.tick,
        date: req.date,
        count: Some(1),
        total_count: None,
    };
    
    match db::insert_key_record(&db_path, &record) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "id": id
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        }))
    }
}

pub async fn send_pc_info(
    State(_state): State<AppState>,
    Json(req): Json<PcInfoRequest>,
) -> impl IntoResponse {
    info!("POST /sendPCInfo - display: {:?}", req.display);
    
    let db_path = get_db_path(&_state).await;
    
    match db::insert_pc_info(
        &db_path,
        req.display.as_deref().unwrap_or(""),
        req.resolution.as_deref().unwrap_or(""),
        req.cpu_info.as_deref().unwrap_or(""),
    ) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "id": id
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        }))
    }
}

pub async fn history_data(
    State(_state): State<AppState>,
    Json(req): Json<HistoryDataRequest>,
) -> impl IntoResponse {
    info!("POST /historyData - begin: {:?}, end: {:?}", req.begin_date, req.end_date);
    
    let db_path = get_db_path(&_state).await;
    
    match db::get_key_records(&db_path, req.begin_date.as_deref(), req.end_date.as_deref()) {
        Ok(records) => Json(serde_json::json!({
            "success": true,
            "data": records
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        }))
    }
}

pub async fn minute_data(
    State(_state): State<AppState>,
    Json(req): Json<MinuteDataRequest>,
) -> impl IntoResponse {
    info!("POST /minuteData - freqType: {:?}, isApp: {:?}", req.freq_type, req.is_app);
    
    let db_path = get_db_path(&_state).await;
    let is_app = req.is_app.unwrap_or(true);
    
    match db::get_app_minutes(&db_path, req.begin_date.as_deref(), req.end_date.as_deref(), is_app) {
        Ok(records) => Json(serde_json::json!({
            "success": true,
            "data": records
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        }))
    }
}

pub async fn opt_keymap(
    State(_state): State<AppState>,
    Json(req): Json<KeymapRequest>,
) -> impl IntoResponse {
    info!("POST /optKeymap - optType: {:?}, id: {:?}", req.opt_type, req.id);
    
    let db_path = get_db_path(&_state).await;
    let opt_type = req.opt_type.as_deref().unwrap_or("");
    
    match opt_type {
        "add" => {
            let name = req.name.as_deref().unwrap_or("Unnamed");
            let data = req.data.as_ref().map(|d| d.to_string()).unwrap_or_default();
            match db::insert_keymap(&db_path, name, &data) {
                Ok(id) => Json(serde_json::json!({
                    "success": true,
                    "id": id
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": e
                }))
            }
        }
        "update" => {
            let id = req.id.unwrap_or(0);
            let name = req.name.as_deref().unwrap_or("");
            let data = req.data.as_ref().map(|d| d.to_string()).unwrap_or_default();
            match db::update_keymap(&db_path, id, name, &data) {
                Ok(_) => Json(serde_json::json!({
                    "success": true
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": e
                }))
            }
        }
        "delete" => {
            let id = req.id.unwrap_or(0);
            match db::delete_keymap(&db_path, id) {
                Ok(_) => Json(serde_json::json!({
                    "success": true
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": e
                }))
            }
        }
        _ => {
            match db::get_keymaps(&db_path) {
                Ok(keymaps) => Json(serde_json::json!({
                    "success": true,
                    "data": keymaps
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": e
                }))
            }
        }
    }
}

pub async fn get_history_date(
    State(_state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("POST /getHistoryDate");
    
    let db_path = get_db_path(&_state).await;
    let req_db = req.get("db").and_then(|d| d.as_str()).unwrap_or(&db_path);
    
    match db::get_history_dates(req_db) {
        Ok(dates) => Json(serde_json::json!({
            "success": true,
            "data": dates
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        }))
    }
}

pub async fn stat_data(
    State(_state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("POST /statData");
    
    let db_path = get_db_path(&_state).await;
    let begin_date = req.get("beginDate").and_then(|d| d.as_str());
    let end_date = req.get("endDate").and_then(|d| d.as_str());
    let req_db = req.get("db").and_then(|d| d.as_str()).unwrap_or(&db_path);
    
    match db::get_statistics(req_db, begin_date, end_date) {
        Ok(stats) => Json(serde_json::json!({
            "success": true,
            "data": stats
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        }))
    }
}

pub async fn delete_data(
    State(_state): State<AppState>,
    Json(req): Json<DeleteDataRequest>,
) -> impl IntoResponse {
    info!("POST /deleteData - optType: {:?}, date: {:?}", req.opt_type, req.date);
    
    let db_path = get_db_path(&_state).await;
    let req_db = req.db.as_deref().unwrap_or(&db_path);
    let opt_type = req.opt_type.as_deref().unwrap_or("");
    
    match opt_type {
        "deleteByDate" => {
            let date = req.date.as_deref().unwrap_or("");
            match db::delete_data_by_date(req_db, date) {
                Ok(_) => Json(serde_json::json!({
                    "success": true
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": e
                }))
            }
        }
        _ => Json(serde_json::json!({
            "success": false,
            "error": "Unknown operation type"
        }))
    }
}

pub async fn zip_download() -> impl IntoResponse {
    info!("GET /zipDownload");
    
    Json(serde_json::json!({
        "success": false,
        "error": "Not implemented"
    }))
}

pub async fn zip_upload(
    State(_state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    info!("POST /zipUpload");
    
    while let Some(field) = multipart.next_field().await.ok().flatten() {
        let filename = field.file_name().map(String::from);
        info!("Received file: {:?}", filename);
        
        if let Ok(data) = field.bytes().await {
            info!("File size: {} bytes", data.len());
            
            if let Some(name) = filename {
                let uploads_dir = std::path::Path::new("uploads");
                if !uploads_dir.exists() {
                    std::fs::create_dir_all(uploads_dir).ok();
                }
                
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let ext = std::path::Path::new(&name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("zip");
                let filepath = uploads_dir.join(format!("{}.{}", timestamp, ext));
                
                if let Ok(mut file) = std::fs::File::create(&filepath) {
                    file.write_all(&data).ok();
                    info!("File saved to {:?}", filepath);
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "success": true
    }))
}

pub async fn version() -> impl IntoResponse {
    info!("GET /version");
    
    Json(serde_json::json!({
        "msg": "showKeyBoardServer Version:",
        "ver": "1.56.0",
        "majorVersion": "1"
    }))
}

pub async fn get_dbs(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("POST /getDbs");
    
    let db_path = get_db_path(&_state).await;
    
    let dbs = vec![
        DbInfo {
            name: "main".to_string(),
            path: db_path,
        }
    ];
    
    Json(serde_json::json!({
        "success": true,
        "data": dbs
    }))
}

pub async fn get_app_minute(
    State(_state): State<AppState>,
    Json(req): Json<AppMinuteRequest>,
) -> impl IntoResponse {
    info!("POST /getAppMinute - isTotal: {:?}", req.is_total);
    
    let db_path = get_db_path(&_state).await;
    let req_db = req.db.as_deref().unwrap_or(&db_path);
    let is_app = req.is_total.unwrap_or(false);
    
    match db::get_app_minutes(req_db, req.begin_date.as_deref(), req.end_date.as_deref(), is_app) {
        Ok(records) => Json(serde_json::json!({
            "success": true,
            "data": records
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        }))
    }
}
