use axum::{
    extract::{State, Multipart},
    response::IntoResponse,
    Json,
};
use log::info;
use std::io::Write;
use crate::AppState;
use crate::models::*;
use crate::db;
use crate::config::AppConfig;

async fn get_db_path(state: &AppState) -> String {
    state.read().await.db_path.clone()
}

pub async fn get_para(
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("POST /getPara");
    
    let db_path = get_db_path(&state).await;
    
    // Read shared state
    let state_guard = state.read().await;
    let desc_ini_path = state_guard.desc_ini_path.clone();
    let user_ini_path = state_guard.user_ini_path.clone();
    let key_list_path = state_guard.key_list_path.clone();
    let network_ip = state_guard.network_ip.clone();
    let info_pc = state_guard.info_pc.clone();
    drop(state_guard);
    
    // Re-read config from files every time (matching Node.js behavior:
    // Object.assign(config, getConfig()))
    let mut config = AppConfig::load_with_override(&desc_ini_path, &user_ini_path);
    
    // Post-processing: same as Node.js getConfig()
    // If shareDbName is empty, use hostname
    if config.common.get("shareDbName").map_or(true, |v| v.is_empty()) {
        if let Ok(hostname) = hostname::get() {
            if let Some(name) = hostname.to_str() {
                config.common.insert("shareDbName".to_string(), name.to_string());
            }
        }
    }
    // Ensure shareDbHour is a valid number
    let hour_valid = config.common.get("shareDbHour")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    config.common.insert("shareDbHour".to_string(), hour_valid.to_string());

    // Build config section (common + dialog as plain HashMap, serialized directly)
    let config_section = ConfigSection {
        common: config.common,
        dialog: config.dialog,
    };
    
    // Read keyList.txt (Node.js format: "key : value" per line)
    // Node.js splits by ':' and ONLY accepts lines with exactly 2 parts (key, value)
    // Lines with extra colons (e.g. "a:b:c") are skipped because arr2.length != 2
    let mut key_list = std::collections::HashMap::new();
    match std::fs::read_to_string(&key_list_path) {
        Ok(content) => {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() { continue; }
                // Match Node.js: split by ':', check exactly 2 parts
                let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    key_list.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
            info!("keyList: loaded {} entries from {:?}", key_list.len(), key_list_path);
        }
        Err(e) => {
            info!("keyList: failed to read {:?}: {}", key_list_path, e);
        }
    }
    
    // Query database settings (with error logging, matching Node.js behavior)
    let data_setting = match db::get_data_setting(&db_path) {
        Ok(ds) => {
            info!("dataSetting: loaded {} entries from db={}", ds.len(), db_path);
            // Print first few keys for debugging
            if ds.is_empty() {
                info!("dataSetting: WARNING - result is EMPTY! DB path: {}", db_path);
            } else {
                let keys: Vec<&String> = ds.keys().take(5).collect();
                info!("dataSetting: first keys = {:?}", keys);
            }
            ds
        }
        Err(e) => {
            info!("dataSetting: query failed path={} err={}", db_path, e);
            std::collections::HashMap::new()
        }
    };
    
    let keymaps = match db::get_keymaps_brief(&db_path) {
        Ok(km) => {
            info!("keymaps: loaded {} entries", km.len());
            km
        }
        Err(e) => {
            info!("keymaps: query failed: {}", e);
            Vec::new()
        }
    };
    
    // InfoPC logging (matching Node.js: infoPC is set by client POST /sendPCInfo)
    let info_pc_is_empty = match &info_pc {
        serde_json::Value::Object(m) => m.is_empty(),
        _ => true,
    };
    if info_pc_is_empty {
        info!("infoPC: no client has reported PC info yet");
    } else {
        info!("infoPC: has data");
    }
    
    // Return in Node.js compatible format
    // Using typed struct ensures serde_json serializes in a single pass,
    // avoiding double-escaping of \" in common/dialog string values.
    Json(ParaResponse {
        config: config_section,
        key_list,
        fonts: crate::font::get_system_fonts(),
        info_pc,
        data_setting,
        keymaps,
        network_ip,
    })
}

pub async fn set_para(
    State(state): State<AppState>,
    Json(req): Json<SetParaRequest>,
) -> impl IntoResponse {
    info!("POST /setPara");

    let state_guard = state.read().await;
    let desc_ini_path = state_guard.desc_ini_path.clone();
    let user_ini_path = state_guard.user_ini_path.clone();
    let key_list_path = state_guard.key_list_path.clone();
    let db_path = state_guard.db_path.clone();
    drop(state_guard);

    // 1. Save keyList to file (key : value format, one per line)
    if let Some(ref key_list) = req.key_list {
        if let Some(obj) = key_list.as_object() {
            let lines: Vec<String> = obj.iter()
                .map(|(k, v)| format!("{} : {}", k, v.as_str().unwrap_or("")))
                .collect();
            let _ = std::fs::write(&key_list_path, lines.join("\n"));
        }
    }

    // 2. Save config to ini file (reload fresh, merge, then save)
    if let Some(ref config_json) = req.config {
        // Re-parse desc.ini + user.ini as base, then apply changes
        let mut config = AppConfig::load_with_override(&desc_ini_path, &user_ini_path);

        // Merge common section from request into config
        if let Some(common) = config_json.get("common").and_then(|c| c.as_object()) {
            for (k, v) in common {
                let val = v.as_str().unwrap_or("").to_string();
                if k == "serverPort" {
                    if let Ok(port) = val.parse() {
                        config.port = port;
                    }
                }
                config.common.insert(k.clone(), val);
            }
        }

        // Merge dialog section from request
        if let Some(dialog) = config_json.get("dialog").and_then(|d| d.as_object()) {
            for (k, v) in dialog {
                config.dialog.insert(k.clone(), v.as_str().unwrap_or("").to_string());
            }
        }

        let _ = config.save(&user_ini_path);
    }

    // 3. Save dataSetting to database
    if let Some(ref data_setting) = req.data_setting {
        let _ = db::set_data_setting(&db_path, data_setting);
    }

    Json(serde_json::json!({
        "code": 200
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
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("POST /sendPCInfo");
    
    // Store infoPC in state (minus flag field, matching Node.js behavior)
    let mut state_guard = state.write().await;
    let mut info = req.clone();
    if let Some(obj) = info.as_object_mut() {
        obj.remove("flag");
    }
    info!("infoPC: {:?}", info);
    state_guard.info_pc = info;
    drop(state_guard);
    
    Json(serde_json::json!({
        "code": 200
    }))
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
    info!("POST /optKeymap - flag: {:?}, mapName: {:?}", req.flag, req.map_name);
    
    let db_path = get_db_path(&_state).await;
    let map_name = req.map_name.as_deref().unwrap_or("");
    
    // Node.js: if mapName.toLowerCase() == 'default', skip
    if map_name.to_lowercase() == "default" {
        return Json(serde_json::json!({ "success": true }));
    }
    
    match req.flag.unwrap_or(0) {
        0 => {
            // Delete by mapName
            match db::delete_keymap(&db_path, map_name) {
                Ok(_) => Json(serde_json::json!({ "success": true })),
                Err(e) => Json(serde_json::json!({ "success": false, "error": e })),
            }
        }
        1 => {
            // Insert
            let detail = req.map_detail.as_deref().unwrap_or("");
            match db::insert_keymap(&db_path, map_name, detail) {
                Ok(_) => Json(serde_json::json!({ "success": true })),
                Err(e) => Json(serde_json::json!({ "success": false, "error": e })),
            }
        }
        2 => {
            // Update by mapName
            let detail = req.map_detail.as_deref().unwrap_or("");
            match db::update_keymap(&db_path, map_name, detail) {
                Ok(_) => Json(serde_json::json!({ "success": true })),
                Err(e) => Json(serde_json::json!({ "success": false, "error": e })),
            }
        }
        _ => {
            // Default: list all keymaps (matching Node.js getKeymaps)
            match db::get_keymaps(&db_path) {
                Ok(keymaps) => Json(serde_json::json!({ "success": true, "data": keymaps })),
                Err(e) => Json(serde_json::json!({ "success": false, "error": e })),
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

pub async fn version(
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("GET /version");
    
    let full_ver = env!("FULL_VERSION");
    let major = state.read().await.info_pc
        .get("majorVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    Json(serde_json::json!({
        "msg": "showKeyBoardServer Version:",
        "ver": full_ver,
        "majorVersion": major
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
