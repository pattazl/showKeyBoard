use rusqlite::{Connection, params, Result as SqlResult};
use std::path::Path;
use std::collections::HashMap;
use log::info;
use crate::models::{KeyRecord, AppMinuteRecord, HistoryDate, StatRecord, UserKeymap, KeymapBrief};

pub fn init_database<P: AsRef<Path>>(db_path: P) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    // Create key_records table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS key_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT,
            app TEXT,
            tick INTEGER,
            date TEXT,
            count INTEGER DEFAULT 1
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    // Create app_minutes table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_minutes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app TEXT,
            date TEXT,
            minutes INTEGER DEFAULT 0
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    // Create keymaps table — matches Node.js schema: mapName/mapDetail
    conn.execute(
        "CREATE TABLE IF NOT EXISTS keymaps (
            mapName TEXT NOT NULL,
            mapDetail TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    // Create pc_info table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pc_info (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            display TEXT,
            resolution TEXT,
            cpu_info TEXT,
            date TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    // Create dataSetting2 table (key-value config store)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dataSetting2 (
            keyname TEXT PRIMARY KEY,
            val TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    // Migration: if dataSetting2 is empty but old dataSetting table has data, migrate it
    // Matches Node.js updateDBStruct() behavior
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM dataSetting2", [], |row| row.get(0)
    ).unwrap_or(0);
    
    if count == 0 {
        let has_old: bool = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='dataSetting'",
            [], |row| row.get::<_, i64>(0)
        ).map(|c| c > 0).unwrap_or(false);
        
        if has_old {
            // Execute each INSERT separately for compatibility
            let migrations = [
                "INSERT OR IGNORE INTO dataSetting2 (keyname, val) SELECT 'keymap', keymap FROM dataSetting WHERE keymap IS NOT NULL",
                "INSERT OR IGNORE INTO dataSetting2 (keyname, val) SELECT 'screenSize', screenSize FROM dataSetting WHERE screenSize IS NOT NULL",
                "INSERT OR IGNORE INTO dataSetting2 (keyname, val) SELECT 'mouseDPI', mouseDPI FROM dataSetting WHERE mouseDPI IS NOT NULL",
                "INSERT OR IGNORE INTO dataSetting2 (keyname, val) SELECT 'topN', topN FROM dataSetting WHERE topN IS NOT NULL",
            ];
            for sql in &migrations {
                conn.execute(sql, []).map_err(|e| e.to_string())?;
            }
            info!("Migrated data from dataSetting to dataSetting2");
        }
    }
    
    // Create indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_key_records_date ON key_records(date)", [])
        .map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_key_records_app ON key_records(app)", [])
        .map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_app_minutes_date ON app_minutes(date)", [])
        .map_err(|e| e.to_string())?;
    
    info!("Database initialized successfully");
    Ok(())
}

// ============== Key Records Operations ==============

pub fn insert_key_record(db_path: &str, record: &KeyRecord) -> Result<i64, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO key_records (key, app, tick, date, count) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            record.key,
            record.app,
            record.tick,
            record.date,
            record.count.unwrap_or(1)
        ],
    ).map_err(|e| e.to_string())?;
    
    Ok(conn.last_insert_rowid())
}

pub fn get_key_records(
    db_path: &str, 
    begin_date: Option<&str>, 
    end_date: Option<&str>
) -> Result<Vec<KeyRecord>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let sql = match (begin_date, end_date) {
        (Some(begin), Some(end)) => format!(
            "SELECT id, key, app, tick, date, count, (SELECT SUM(count) FROM key_records WHERE date = kr.date AND app = kr.app) as totalCount 
             FROM key_records kr 
             WHERE date BETWEEN '{}' AND '{}' 
             ORDER BY date DESC, tick DESC", 
            begin, end
        ),
        (Some(begin), None) => format!(
            "SELECT id, key, app, tick, date, count, (SELECT SUM(count) FROM key_records WHERE date = kr.date AND app = kr.app) as totalCount 
             FROM key_records kr 
             WHERE date >= '{}' 
             ORDER BY date DESC, tick DESC", 
            begin
        ),
        _ => "SELECT id, key, app, tick, date, count, (SELECT SUM(count) FROM key_records WHERE date = kr.date AND app = kr.app) as totalCount 
              FROM key_records kr 
              ORDER BY date DESC, tick DESC".to_string()
    };
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let records = stmt.query_map([], |row| {
        Ok(KeyRecord {
            id: row.get(0)?,
            key: row.get(1)?,
            app: row.get(2)?,
            tick: row.get(3)?,
            date: row.get(4)?,
            count: row.get(5)?,
            total_count: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;
    
    Ok(records)
}

// ============== App Minutes Operations ==============

pub fn get_app_minutes(
    db_path: &str,
    begin_date: Option<&str>,
    end_date: Option<&str>,
    is_app: bool
) -> Result<Vec<AppMinuteRecord>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let group_by = if is_app { "app, date" } else { "date" };
    
    let sql = match (begin_date, end_date) {
        (Some(begin), Some(end)) => format!(
            "SELECT app, date, SUM(minutes) as minutes 
             FROM app_minutes 
             WHERE date BETWEEN '{}' AND '{}' 
             GROUP BY {} 
             ORDER BY date DESC", 
            begin, end, group_by
        ),
        _ => format!(
            "SELECT app, date, SUM(minutes) as minutes 
             FROM app_minutes 
             GROUP BY {} 
             ORDER BY date DESC", 
            group_by
        )
    };
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let records = stmt.query_map([], |row| {
        Ok(AppMinuteRecord {
            app: row.get(0)?,
            date: row.get(1)?,
            minutes: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;
    
    Ok(records)
}

// ============== History Date Operations ==============

pub fn get_history_dates(db_path: &str) -> Result<Vec<HistoryDate>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT DISTINCT date FROM key_records ORDER BY date DESC"
    ).map_err(|e| e.to_string())?;
    
    let dates = stmt.query_map([], |row| {
        Ok(HistoryDate {
            date: row.get(0)?
        })
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;
    
    Ok(dates)
}

// ============== Statistics Operations ==============

pub fn get_statistics(
    db_path: &str,
    begin_date: Option<&str>,
    end_date: Option<&str>
) -> Result<Vec<StatRecord>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let sql = match (begin_date, end_date) {
        (Some(begin), Some(end)) => format!(
            "SELECT date, app, SUM(count) as count 
             FROM key_records 
             WHERE date BETWEEN '{}' AND '{}' 
             GROUP BY date, app 
             ORDER BY date DESC, count DESC", 
            begin, end
        ),
        _ => "SELECT date, app, SUM(count) as count 
              FROM key_records 
              GROUP BY date, app 
              ORDER BY date DESC, count DESC".to_string()
    };
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let records = stmt.query_map([], |row| {
        Ok(StatRecord {
            date: row.get(0)?,
            app: row.get(1)?,
            count: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;
    
    Ok(records)
}

// ============== Keymap Operations ==============
// All match Node.js schema: keymaps(mapName, mapDetail)

pub fn insert_keymap(db_path: &str, map_name: &str, map_detail: &str) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO keymaps(mapName, mapDetail) VALUES(?1, ?2)",
        params![map_name, map_detail],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn update_keymap(db_path: &str, map_name: &str, map_detail: &str) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute(
        "UPDATE keymaps SET mapDetail = ?1 WHERE mapName = ?2",
        params![map_detail, map_name],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn delete_keymap(db_path: &str, map_name: &str) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM keymaps WHERE mapName = ?1", params![map_name])
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn get_keymaps(db_path: &str) -> Result<Vec<UserKeymap>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT mapName, mapDetail FROM keymaps"
    ).map_err(|e| e.to_string())?;
    
    let keymaps = stmt.query_map([], |row| {
        Ok(UserKeymap {
            map_name: row.get(0)?,
            map_detail: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;
    
    Ok(keymaps)
}

// ============== Data Delete Operations ==============

pub fn delete_data_by_date(db_path: &str, date: &str) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM key_records WHERE date = ?1", params![date])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM app_minutes WHERE date = ?1", params![date])
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

// ============== Data Setting Operations (dataSetting2 table) ==============

/// Get all key-value settings for getPara response.
///
/// Matches Node.js getDataSetting():
/// 1. Read key-value pairs from dataSetting2 table
/// 2. UNION join keymaps.mapDetail with dataSetting2 when keyname='keymap'
/// 3. Merge deprecated dataSetting table's 4 columns (keymap, screenSize, mouseDPI, topN)
///    as fallback in case dataSetting2 migration hasn't run
pub fn get_data_setting(db_path: &str) -> Result<HashMap<String, String>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    // 1. Query from dataSetting2 + UNION keymaps join (matching Node.js exactly)
    let sql1 = "SELECT keyname, val FROM dataSetting2
        UNION
        SELECT 'mapDetail', mapDetail FROM keymaps 
        JOIN dataSetting2 ON mapName = val 
        WHERE keyname = 'keymap'";
    
    let mut stmt = conn.prepare(sql1).map_err(|e| {
        format!("get_data_setting: prepare dataSetting2 failed: {}", e)
    })?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
        ))
    }).map_err(|e| format!("get_data_setting: query dataSetting2 failed: {}", e))?;
    
    let mut settings = HashMap::new();
    for row in rows {
        let (key, val) = row.map_err(|e| e.to_string())?;
        settings.insert(key, val);
    }
    
    // 2. If old dataSetting table exists, merge its first row's 4 columns
    //    (keymap, screenSize, mouseDPI, topN) as key-value pairs.
    //    dataSetting2 takes precedence, only fill in missing keys.
    let table_exists: bool = conn
        .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='dataSetting'")
        .and_then(|mut stmt| {
            stmt.query_row([], |row| row.get::<_, i64>(0))
        })
        .map(|count| count > 0)
        .unwrap_or(false);
    
    if table_exists {
        let sql2 = "SELECT keymap, screenSize, mouseDPI, topN FROM dataSetting LIMIT 1";
        if let Ok(mut stmt2) = conn.prepare(sql2) {
            if let Ok(row) = stmt2.query_row([], |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            }) {
                let mut insert_if_present = |key: &str, val: Option<String>| {
                    if let Some(v) = val {
                        if !v.is_empty() {
                            settings.entry(key.to_string()).or_insert(v);
                        }
                    }
                };
                insert_if_present("keymap", row.0);
                insert_if_present("screenSize", row.1);
                insert_if_present("mouseDPI", row.2);
                insert_if_present("topN", row.3);
            }
        }
    }
    
    info!("get_data_setting: loaded {} settings", settings.len());
    Ok(settings)
}

/// Set key-value settings into dataSetting2
pub fn set_data_setting(db_path: &str, hash: &HashMap<String, String>) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    for (key, val) in hash {
        conn.execute(
            "INSERT OR REPLACE INTO dataSetting2 (keyname, val) VALUES (?1, ?2)",
            params![key, val],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// ============== Keymaps (mapName/mapDetail format for getPara) ==============

/// Get keymaps in {mapName, mapDetail} format for getPara compatibility
/// Matches Node.js getKeymaps(): SELECT mapName,mapDetail FROM keymaps
pub fn get_keymaps_brief(db_path: &str) -> Result<Vec<KeymapBrief>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT mapName, mapDetail FROM keymaps"
    ).map_err(|e| format!("get_keymaps_brief: prepare failed: {}", e))?;
    
    let keymaps = stmt.query_map([], |row| {
        Ok(KeymapBrief {
            map_name: row.get(0)?,
            map_detail: row.get(1)?,
        })
    }).map_err(|e| format!("get_keymaps_brief: query failed: {}", e))?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;
    
    info!("get_keymaps_brief: loaded {} keymaps", keymaps.len());
    Ok(keymaps)
}


