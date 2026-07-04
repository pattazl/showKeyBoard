use axum::{
    extract::{State, Multipart},
    response::IntoResponse,
    Json,
};
use log::{info, warn};
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

/// Matches Node.js setParaFun: saves config, keyList, dataSetting.
/// When config or keyList changes, broadcasts "IniMonitor" to ahkClient via WebSocket.
pub async fn set_para(
    State(state): State<AppState>,
    Json(req): Json<SetParaRequest>,
) -> impl IntoResponse {
    info!("POST /setPara");

    let mut is_update = false;

    // Read current state
    let (desc_ini_path, user_ini_path, key_list_path, db_path, ws_tx) = {
        let sg = state.read().await;
        (
            sg.desc_ini_path.clone(),
            sg.user_ini_path.clone(),
            sg.key_list_path.clone(),
            sg.db_path.clone(),
            sg.ws_tx.clone(),
        )
    };

    // Load current config for change detection
    let current_config = AppConfig::load_with_override(&desc_ini_path, &user_ini_path);

    // 1. Save keyList to file (key : value format, one per line)
    if let Some(ref key_list) = req.key_list {
        if let Some(obj) = key_list.as_object() {
            let new_key_list_str = serde_json::to_string(key_list).unwrap_or_default();
            // Compare with current file content
            let old_content = std::fs::read_to_string(&key_list_path).unwrap_or_default();
            if old_content.trim() != new_key_list_str.trim() {
                let lines: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{} : {}", k, v.as_str().unwrap_or("")))
                    .collect();
                let _ = std::fs::write(&key_list_path, lines.join("\n"));
                is_update = true;
                info!("keyList changed, saved to {:?}", key_list_path);
            }
        }
    }

    // 2. Save config to ini file (reload fresh, merge, then save)
    if let Some(ref config_json) = req.config {
        let _new_config_str = serde_json::to_string(config_json).unwrap_or_default();
        // Build merged config to compare
        let mut new_config = AppConfig::load_with_override(&desc_ini_path, &user_ini_path);

        if let Some(common) = config_json.get("common").and_then(|c| c.as_object()) {
            for (k, v) in common {
                let val = v.as_str().unwrap_or("").to_string();
                if k == "serverPort" {
                    if let Ok(port) = val.parse() {
                        new_config.port = port;
                    }
                }
                new_config.common.insert(k.clone(), val);
            }
        }
        if let Some(dialog) = config_json.get("dialog").and_then(|d| d.as_object()) {
            for (k, v) in dialog {
                new_config.dialog.insert(k.clone(), v.as_str().unwrap_or("").to_string());
            }
        }

        // Compare with current config
        let current_config_str = serde_json::to_string(&current_config.common).unwrap_or_default()
            + &serde_json::to_string(&current_config.dialog).unwrap_or_default();
        let new_cfg_str = serde_json::to_string(&new_config.common).unwrap_or_default()
            + &serde_json::to_string(&new_config.dialog).unwrap_or_default();

        if current_config_str != new_cfg_str {
            let _ = new_config.save(&user_ini_path);
            is_update = true;
            info!("config changed, saved to {:?}", user_ini_path);
        }
    }

    // 3. Save dataSetting to database
    if let Some(ref data_setting) = req.data_setting {
        let _ = db::set_data_setting(&db_path, data_setting);
    }

    // 4. Broadcast "IniMonitor" to ahkClient clients (matches Node.js WebSocket notification)
    if is_update {
        info!("Config/keyList changed, broadcasting IniMonitor to ahkClient");
        let _ = ws_tx.send(("ahkClient".to_string(), "IniMonitor".to_string()));
    }

    Json(serde_json::json!({
        "code": 200
    }))
}

/// Matches Node.js exitFun: saves last data (saveLastData → insertDataFun),
/// then triggers graceful shutdown (server.close + process.exit equivalent).
pub async fn exit_server(State(state): State<AppState>) -> impl IntoResponse {
    info!("POST /exit - Server shutdown requested");

    // 1. Save last data before exit (matches Node.js saveLastData → insertDataFun)
    {
        let sg = state.read().await;
        let pre_data = sg.pre_data.clone();
        let db_path = sg.db_path.clone();
        let base_dir = sg.base_dir.clone();

        // Check if there is pending data to save (tick > 0 means new data came in)
        let tick = pre_data.get("tick").and_then(|v| v.as_i64()).unwrap_or(0);
        if tick > 0 {
            let keyname = pre_data
                .get("data")
                .and_then(|d| d.get("key"))
                .and_then(|k| k.as_str())
                .unwrap_or("");
            let date = pre_data
                .get("date")
                .and_then(|d| d.as_str())
                .unwrap_or("");

            if let Err(e) = db::insert_key_record(&db_path, keyname, 1, tick, date) {
                warn!("Failed to save last data before exit: {}", e);
            } else {
                info!("Saved last data before exit: key={}, tick={}, date={}", keyname, tick, date);
            }

            // Handle updateTime (matches Node.js: write to updateTime.txt)
            if let Some(update_time) = pre_data.get("updateTime").and_then(|v| v.as_str()) {
                let update_time_path = std::path::PathBuf::from(&base_dir).join("updateTime.txt");
                if let Err(e) = std::fs::write(&update_time_path, update_time) {
                    warn!("Failed to write updateTime.txt: {}", e);
                }
            }
        }

        // Trigger graceful shutdown (matches Node.js: wss.close() + server.close() + process.exit)
        sg.shutdown_notify.notify_one();
    }

    info!("Shutdown signal sent, server is shutting down...");
    Json(serde_json::json!({
        "success": true,
        "message": "Server exiting"
    }))
}

/// Matches Node.js dataFun: inserts into events table, stores preData, broadcasts to web clients.
pub async fn upload_data(
    State(state): State<AppState>,
    Json(req): Json<DataRequest>,
) -> impl IntoResponse {
    info!("POST /data - app: {:?}, type: {:?}", req.app_name, req.data_type);
    
    let db_path = get_db_path(&state).await;
    
    let keyname = req.data.as_ref()
        .and_then(|d| d.get("key"))
        .and_then(|k| k.as_str())
        .unwrap_or("");
    let tick = req.tick.unwrap_or(0);
    let date = req.date.as_deref().unwrap_or("");
    
    let result = db::insert_key_record(&db_path, keyname, 1, tick, date);
    
    // Store preData and broadcast to web clients (matches Node.js dataFun)
    {
        let mut sg = state.write().await;
        sg.pre_data = serde_json::to_value(&req).unwrap_or(serde_json::Value::Null);
        let pre_data_str = serde_json::to_string(&sg.pre_data).unwrap_or_default();
        let _ = sg.ws_tx.send(("".to_string(), pre_data_str));
    }
    
    match result {
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

/// Matches Node.js getRecords(begin, end, newDbName).
/// Queries `events` table for today, `stat` table for historical data.
/// Returns a flat JSON array: [{keyname, keycount, date, tick}, ...].
pub async fn history_data(
    State(state): State<AppState>,
    body: Option<Json<serde_json::Value>>,
) -> impl IntoResponse {
    // Extract params with optional chaining like Node.js req.body?.beginDate
    let begin = body.as_ref()
        .and_then(|b| b.get("beginDate"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let end = body.as_ref()
        .and_then(|b| b.get("endDate"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let db_name = body.as_ref()
        .and_then(|b| b.get("db"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    info!("POST /historyData - begin: {:?}, end: {:?}, db: {:?}", begin, end, db_name);

    // Resolve DB path (Node.js getDbObj)
    let state_guard = state.read().await;
    let default_db_path = state_guard.db_path.clone();
    let base_dir = state_guard.base_dir.clone();
    drop(state_guard);

    let db_path = resolve_minute_db(&db_name, &default_db_path, &base_dir);

    // Node.js: if (null == db) return []
    if db_path.is_empty() {
        return Json(serde_json::json!([]));
    }

    // Node.js: if (begin > end) return [] (string comparison)
    if !begin.is_empty() && !end.is_empty() && begin > end {
        return Json(serde_json::json!([]));
    }

    match db::get_records(&db_path, begin, end) {
        Ok(records) => Json(serde_json::Value::Array(
            records.into_iter().map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)).collect(),
        )),
        Err(_e) => {
            info!("historyData query error: {}", _e);
            Json(serde_json::json!([]))
        }
    }
}

pub async fn minute_data(
    State(state): State<AppState>,
    Json(req): Json<MinuteDataRequest>,
) -> impl IntoResponse {
    let freq_type = match req.freq_type {
        Some(serde_json::Value::Number(ref n)) => n.as_i64().unwrap_or(0) as i32,
        Some(serde_json::Value::String(ref s)) => s.parse::<i32>().unwrap_or(0),
        _ => 0,
    };
    let is_app = req.is_app.unwrap_or(false);
    info!("POST /minuteData - freqType: {}, isApp: {}, db: {:?}", freq_type, is_app, req.db);

    // Resolve database path: if req.db is provided, look in <base_dir>/dbs/<db>.db
    let state_guard = state.read().await;
    let default_db_path = state_guard.db_path.clone();
    let base_dir = state_guard.base_dir.clone();
    drop(state_guard);

    let db_path = resolve_minute_db(&req.db, &default_db_path, &base_dir);

    // Node.js: if (null == db) return []
    if db_path.is_empty() {
        return Json(serde_json::json!([]));
    }

    match db::get_minute_records(
        &db_path,
        req.begin_date.as_deref(),
        req.end_date.as_deref(),
        freq_type,
        is_app,
    ) {
        Ok(records) => Json(serde_json::Value::Array(records)),
        Err(_e) => {
            info!("minute_data query error: {}", _e);
            Json(serde_json::json!([]))
        }
    }
}

/// Resolve database path for minute_data matching Node.js getDbObj().
fn resolve_minute_db(db_name: &Option<String>, default_db: &str, base_dir: &str) -> String {
    let dbn = match db_name {
        Some(name) => name.trim().to_string(),
        None => String::new(),
    };

    if dbn.is_empty() {
        // Use default database
        return default_db.to_string();
    }

    // Node.js: dbpath = path.join(dbsPath, newDbName + '.db')
    let dbpath = std::path::Path::new(base_dir).join("dbs").join(format!("{}.db", dbn));
    if dbpath.exists() {
        info!("minute_data: using external db: {}", dbpath.display());
        dbpath.to_string_lossy().to_string()
    } else {
        info!("minute_data: external db not found: {}", dbpath.display());
        String::new()
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

/// Matches Node.js getHistoryDate: query stat table for all distinct dates.
/// Returns a flat JSON array like ["2025-01-15","2025-01-14",...].
pub async fn get_history_date(
    State(state): State<AppState>,
    body: Option<Json<serde_json::Value>>,
) -> impl IntoResponse {
    info!("POST /getHistoryDate");

    let state_guard = state.read().await;
    let default_db_path = state_guard.db_path.clone();
    let base_dir = state_guard.base_dir.clone();
    drop(state_guard);

    // Node.js: req.body?.db — optional db name, look in dbs/<name>.db if provided
    let db_name = body
        .as_ref()
        .and_then(|b| b.get("db"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let db_path = resolve_minute_db(&db_name, &default_db_path, &base_dir);

    // Node.js: if (null == db) return []
    if db_path.is_empty() {
        return Json(serde_json::json!([]));
    }

    match db::get_history_dates(&db_path) {
        Ok(dates) => Json(serde_json::Value::Array(
            dates.into_iter().map(serde_json::Value::String).collect(),
        )),
        Err(_e) => {
            info!("getHistoryDate query error: {}", _e);
            Json(serde_json::json!([]))
        }
    }
}

/// Matches Node.js statData(begin, end, newDbName):
/// 4 parallel queries on `stat` table, returns 4-element array.
pub async fn stat_data(
    State(state): State<AppState>,
    body: Option<Json<serde_json::Value>>,
) -> impl IntoResponse {
    let begin = body.as_ref()
        .and_then(|b| b.get("beginDate"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let end = body.as_ref()
        .and_then(|b| b.get("endDate"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let db_name = body.as_ref()
        .and_then(|b| b.get("db"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    info!("POST /statData - begin: {:?}, end: {:?}, db: {:?}", begin, end, db_name);

    let state_guard = state.read().await;
    let default_db_path = state_guard.db_path.clone();
    let base_dir = state_guard.base_dir.clone();
    drop(state_guard);

    let db_path = resolve_minute_db(&db_name, &default_db_path, &base_dir);
    if db_path.is_empty() {
        return Json(serde_json::json!([]));
    }

    // Default topN = 10, matching Node.js: globalTopN = dataSetting?.topN || 10
    let top_n = 10i32;
    let app_top_n = 10i32;

    match db::stat_data(&db_path, begin, end, top_n, app_top_n) {
        Ok(results) => Json(serde_json::Value::Array(
            results.into_iter().map(|v| serde_json::Value::Array(v)).collect(),
        )),
        Err(_e) => {
            info!("statData query error: {}", _e);
            Json(serde_json::json!([]))
        }
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

/// Matches Node.js getDbsFun: scan <base_dir>/dbs/ for .db files, return names without extension.
pub async fn get_dbs(
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("POST /getDbs");

    let state_guard = state.read().await;
    let base_dir = state_guard.base_dir.clone();
    drop(state_guard);

    let dbs_dir = std::path::Path::new(&base_dir).join("dbs");

    match std::fs::read_dir(&dbs_dir) {
        Ok(entries) => {
            let mut db_files: Vec<String> = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.to_ascii_lowercase() == "db" {
                            // basename without .db extension
                            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                                db_files.push(name.to_string());
                            }
                        }
                    }
                }
            }
            info!("getDbs: found {} db files in {:?}", db_files.len(), dbs_dir);
            Json(serde_json::json!({ "code": 200, "dbs": db_files }))
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                info!("getDbs: directory not found: {:?}", dbs_dir);
                Json(serde_json::json!({
                    "code": 10,
                    "msg": format!("目录不存在: {}", dbs_dir.display())
                }))
            } else {
                info!("getDbs: error reading directory {:?}: {}", dbs_dir, e);
                Json(serde_json::json!({
                    "code": 10,
                    "msg": format!("获取文件时出错: {}", e)
                }))
            }
        }
    }
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
