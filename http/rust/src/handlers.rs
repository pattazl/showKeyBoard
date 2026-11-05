use axum::{
    extract::{State, Multipart},
    response::IntoResponse,
    Json,
};
use log::{info, warn};
use std::io::Write;
use crate::AppState;
use crate::websocket::{AHK_CLIENT_FLAG, WEB_CLIENT_FLAG};
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
        let _ = ws_tx.send((AHK_CLIENT_FLAG.to_string(), "IniMonitor".to_string()));
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
        let (pre_data, db_path) = {
            let sg = state.read().await;
            (sg.pre_data.clone(), sg.db_path.clone())
        };

        // Node.js: if (preData['tick'] > 0) { await insertDataFun(preData); preData['tick'] = 0 }
        let tick = pre_data.get("tick").and_then(|v| v.as_i64()).unwrap_or(0);
        if tick > 0 {
            info!("insertDataFun");
            let mut pre = pre_data.clone();
            insert_data_fun(&state, &db_path, &mut pre).await;
            let mut sg = state.write().await;
            sg.pre_data["tick"] = serde_json::json!(0);
        }

        // Trigger graceful shutdown (matches Node.js: wss.close() + server.close() + process.exit)
        state.read().await.shutdown_notify.notify_one();
    }

    info!("Shutdown signal sent, server is shutting down...");
    Json(serde_json::json!({
        "success": true,
        "message": "Server exiting"
    }))
}

/// Matches Node.js insertMiniuteFun(records) in common.js:
/// processes records.MinuteRecords → statFreq/appFreq (with lastMinuteRecords /
/// lastFlagRecord merge), then deletes the MinuteRecords field from the record.
async fn insert_miniute_fun(state: &AppState, db_path: &str, records: &mut serde_json::Value) {
    let has_mr = records.get("MinuteRecords")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    if !has_mr {
        return;
    }

    let mut arr: Vec<serde_json::Value> =
        records["MinuteRecords"].as_array().unwrap().clone();
    let last = arr.pop().unwrap(); // Node.js: MinuteRecords.pop() — 去掉最后一个

    let last_time = {
        let sg = state.read().await;
        sg.last_minute_record.get("Minute")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };

    let last_minute = last.get("Minute").and_then(|v| v.as_str()).unwrap_or("");
    if last_minute > last_time.as_str() {
        // 有新数据进来，可以插入数据；过滤已插入的（Minute > lastTime）
        let mut new_arr: Vec<serde_json::Value> = arr.into_iter().filter(|item| {
            item.get("Minute").and_then(|v| v.as_str()).unwrap_or("") > last_time.as_str()
        }).collect();

        // 合并上次的 lastFlagRecord 遗留数据（鼠标距离不能合并）
        let lfr = { state.read().await.last_flag_record.clone() };
        if !lfr.is_null() && !new_arr.is_empty() {
            let lfr_minute = lfr.get("Minute").and_then(|v| v.as_str()).unwrap_or("");
            let first_minute = new_arr[0].get("Minute").and_then(|v| v.as_str()).unwrap_or("");
            if first_minute == lfr_minute {
                if let Some(first_obj) = new_arr[0].as_object_mut() {
                    let lfr_key = lfr.get("KeyCount").and_then(|v| v.as_i64()).unwrap_or(0);
                    let lfr_mouse = lfr.get("MouseCount").and_then(|v| v.as_i64()).unwrap_or(0);
                    *first_obj.entry("KeyCount").or_insert(serde_json::json!(0)) =
                        serde_json::json!(first_obj.get("KeyCount").and_then(|v| v.as_i64()).unwrap_or(0) + lfr_key);
                    *first_obj.entry("MouseCount").or_insert(serde_json::json!(0)) =
                        serde_json::json!(first_obj.get("MouseCount").and_then(|v| v.as_i64()).unwrap_or(0) + lfr_mouse);
                }
                info!("merge lastFlagRecord");
            } else {
                new_arr.insert(0, lfr.clone()); // 插入到开头
            }
            state.write().await.last_flag_record = serde_json::Value::Null;
        }

        if !new_arr.is_empty() {
            if let Err(e) = db::insert_minute_records(db_path, &new_arr) {
                warn!("insertMinute error: {}", e);
            }
            let new_last = new_arr.pop().unwrap(); // 用保存的数据作为上次的最后一个
            state.write().await.last_minute_record = new_last;
        }
    }

    // 重新启动时（LastFlag == 1），更新的数据需要累计
    if last.get("LastFlag").and_then(|v| v.as_i64()) == Some(1) {
        info!("lastFlagRecord saved for merge");
        state.write().await.last_flag_record = last;
    }

    // delete records.MinuteRecords — 广播给 Web 的数据不含该字段
    if let Some(obj) = records.as_object_mut() {
        obj.remove("MinuteRecords");
    }
}

/// Matches Node.js insertDataFun(records):
/// 1. write updateTime → updateTime.txt, delete the field
/// 2. insertMiniuteFun(records)
/// 3. insertData(records) → events table
async fn insert_data_fun(state: &AppState, db_path: &str, records: &mut serde_json::Value) {
    if let Some(update_time) = records.get("updateTime").and_then(|v| v.as_str()).map(String::from) {
        let base_dir = {
            let sg = state.read().await;
            sg.base_dir.clone()
        };
        let _ = std::fs::write(std::path::Path::new(&base_dir).join("updateTime.txt"), &update_time);
        if let Some(obj) = records.as_object_mut() {
            obj.remove("updateTime");
        }
    }
    insert_miniute_fun(state, db_path, records).await;
    if let Err(e) = db::insert_key_records(db_path, records) {
        warn!("insertData error: {}", e);
    }
}

/// Matches Node.js dataFun: 
/// 1. insertMiniuteFun(data) — process MinuteRecords into statFreq/appFreq
/// 2. if preData.tick > 0 && data.tick > 0 && tick differs → insertDataFun(preData)
/// 3. preData = data
/// 4. broadcast JSON.stringify(data) to web clients
/// Response: plain text "OK".
pub async fn upload_data(
    State(state): State<AppState>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("POST /data - tick: {:?}", data.get("tick").and_then(|v| v.as_i64()));

    let db_path = get_db_path(&state).await;
    let mut data = data;

    // 1. Node.js: await insertMiniuteFun(data)
    insert_miniute_fun(&state, &db_path, &mut data).await;

    // 2. tick 变化时保存上一份 preData
    let mut to_save: Option<serde_json::Value> = None;
    {
        let mut sg = state.write().await;
        let pre_tick = sg.pre_data.get("tick").and_then(|v| v.as_i64()).unwrap_or(0);
        let cur_tick = data.get("tick").and_then(|v| v.as_i64()).unwrap_or(0);
        if pre_tick > 0 && cur_tick > 0 && cur_tick != pre_tick {
            info!("preTick,tick: {}, {}", pre_tick, cur_tick);
            to_save = Some(sg.pre_data.clone());
        }
        // 3. preData = data
        sg.pre_data = data.clone();
    }
    if let Some(mut pre) = to_save {
        insert_data_fun(&state, &db_path, &mut pre).await;
    }

    // 4. broadcast to web clients (matches Node.js: JSON.stringify(data))
    let payload = serde_json::to_string(&data).unwrap_or_default();
    {
        let sg = state.read().await;
        let _ = sg.ws_tx.send((WEB_CLIENT_FLAG.to_string(), payload));
    }

    // Node.js: res.send('OK')
    "OK"
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

    // Matches Node.js: globalTopN = dataSetting.topN ?? 10, globalAppTopN = dataSetting.appTopN ?? 10
    let settings = db::get_data_setting(&db_path).unwrap_or_default();
    let top_n = settings
        .get("topN")
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(10);
    let app_top_n = settings
        .get("appTopN")
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(10);
    info!("statData: begin={}, end={}, topN={}, appTopN={}", begin, end, top_n, app_top_n);

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

/// Matches Node.js deleteDataFun: body { flag: number, date: [array] }.
/// flag == 0: delete events by tick; flag == 1: delete stat/statFreq/appFreq by date.
/// Response: { code: 200 } on success, { code: 10 } on invalid params or error.
pub async fn delete_data(
    State(state): State<AppState>,
    Json(req): Json<DeleteDataRequest>,
) -> impl IntoResponse {
    info!("POST /deleteData - flag: {:?}, date: {:?}", req.flag, req.date);

    let db_path = get_db_path(&state).await;

    // Node.js: typeof data.flag == 'number' && data.date instanceof Array
    match (req.flag, req.date) {
        (Some(flag), Some(dates)) if !dates.is_empty() => {
            match db::delete_data(&db_path, &dates, flag) {
                Ok(_) => Json(serde_json::json!({ "code": 200 })),
                Err(e) => {
                    info!("deleteData error: {}", e);
                    Json(serde_json::json!({ "code": 10 }))
                }
            }
        }
        _ => Json(serde_json::json!({ "code": 10 })),
    }
}

/// Convert a number to base36 string (matches Node.js Number.prototype.toString(36)).
fn to_base36(mut n: u64) -> String {
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if n == 0 {
        return "0".to_string();
    }
    let mut buf = Vec::new();
    while n > 0 {
        buf.push(CHARS[(n % 36) as usize]);
        n /= 36;
    }
    buf.reverse();
    String::from_utf8(buf).unwrap_or_default()
}

fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Matches Node.js zipDownload: packs showKeyBoard.ini + keyList.txt + records.db
/// into a zip (DEFLATE level 5), returns as octet-stream attachment.
pub async fn zip_download(
    State(state): State<AppState>,
) -> axum::response::Response {
    info!("GET /zipDownload");

    let (user_ini_path, key_list_path, db_path) = {
        let sg = state.read().await;
        (sg.user_ini_path.clone(), sg.key_list_path.clone(), sg.db_path.clone())
    };

    // Collect files: zip entry name = file basename (matches Node.js zip.file)
    let files: Vec<String> = vec![user_ini_path, key_list_path, db_path];

    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut zipw = zip::ZipWriter::new(&mut buf);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(5)); // Node.js JSZip compressionOptions level 5

        for path in &files {
            let p = std::path::Path::new(path);
            let name = p.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            match std::fs::read(p) {
                Ok(data) => {
                    if let Err(e) = zipw.start_file(&name, opts)
                        .and_then(|_| zipw.write_all(&data).map_err(zip::result::ZipError::Io))
                    {
                        warn!("zipDownload: failed to add {}: {}", name, e);
                    }
                }
                Err(e) => warn!("zipDownload: skip missing file {}: {}", path, e),
            }
        }
        if let Err(e) = zipw.finish() {
            warn!("zipDownload: finish failed: {}", e);
        }
    }
    let content = buf.into_inner();

    // Node.js: fileName = new Date().getTime().toString(36)
    let file_name = format!("showkeyboard-{}.zip", to_base36(now_millis() as u64));

    (
        [
            (axum::http::header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", file_name),
            ),
        ],
        content,
    )
        .into_response()
}

/// Matches Node.js clearUpload: deletes files in uploads/ older than 3 minutes
/// (timestamp-named files, string comparison like Node.js).
fn clear_uploads(dir: &std::path::Path) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    let min_name = format!("{}.zip", now_millis() - 3 * 60 * 1000);
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name < min_name {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Unzip and overwrite target files via mapping (matches Node.js unZipCore + configHash).
/// Known entries: showKeyBoard.ini / keyList.txt / records.db. Unknown entries are skipped.
fn unzip_apply(zip_path: &str, mapping: &std::collections::HashMap<String, String>, msg: &mut Vec<String>) -> Result<(), String> {
    use std::io::Read;

    let f = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        if file.is_dir() {
            continue; // Node.js: filter out directories
        }
        let name = file.name().to_string();
        match mapping.get(&name) {
            Some(dest) => {
                let mut data = Vec::new();
                file.read_to_end(&mut data).map_err(|e| e.to_string())?;
                std::fs::write(dest, &data).map_err(|e| e.to_string())?;
                msg.push(format!("{} Ok", name));
                info!("zipUpload: extracted {} -> {}", name, dest);
            }
            None => {
                msg.push(format!("{} Skip", name));
                warn!("zipUpload: unknown entry {}, skipped", name);
            }
        }
    }
    Ok(())
}

/// Matches Node.js zipUpload: saves uploaded zip to uploads/<timestamp>.<ext>,
/// clears uploads older than 3 minutes, then unzips and overwrites
/// showKeyBoard.ini / keyList.txt / records.db.
/// Response: { oriName, path, msg: [...], code } — code 200 on success, 1 on failure.
pub async fn zip_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    info!("POST /zipUpload");

    let (user_ini_path, key_list_path, db_path) = {
        let sg = state.read().await;
        (sg.user_ini_path.clone(), sg.key_list_path.clone(), sg.db_path.clone())
    };

    let mut ori_name = String::new();
    let mut saved_path = String::new();
    let mut msg: Vec<String> = vec!["Upload Success".to_string()];

    // Node.js multer storage: uploads/<timestamp><ext>, single file field
    if let Some(field) = multipart.next_field().await.ok().flatten() {
        ori_name = field.file_name().map(String::from).unwrap_or_default();
        match field.bytes().await {
            Ok(data) => {
                let uploads_dir = std::path::Path::new("uploads");
                let _ = std::fs::create_dir_all(uploads_dir);

                let ext = std::path::Path::new(&ori_name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("zip");
                let filepath = uploads_dir.join(format!("{}.{}", now_millis(), ext));

                match std::fs::File::create(&filepath).and_then(|mut f| f.write_all(&data)) {
                    Ok(_) => {
                        saved_path = filepath.to_string_lossy().to_string();
                        info!("zipUpload: saved to {}", saved_path);
                    }
                    Err(e) => warn!("zipUpload: failed to save file: {}", e),
                }
            }
            Err(e) => warn!("zipUpload: failed to read field: {}", e),
        }
    }

    // Node.js: clearUpload(hash) — clean old uploads first
    clear_uploads(std::path::Path::new("uploads"));

    // Node.js configHash: entry name -> target file path
    let code = if !saved_path.is_empty() {
        let mapping = std::collections::HashMap::from([
            ("showKeyBoard.ini".to_string(), user_ini_path),
            ("keyList.txt".to_string(), key_list_path),
            ("records.db".to_string(), db_path),
        ]);
        match unzip_apply(&saved_path, &mapping, &mut msg) {
            Ok(_) => 200i64,
            Err(e) => {
                msg.push(format!("解压缩失败: {}", e));
                1i64
            }
        }
    } else {
        1i64
    };

    Json(serde_json::json!({
        "oriName": ori_name,
        "path": saved_path,
        "msg": msg,
        "code": code
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
