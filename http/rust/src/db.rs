use rusqlite::{Connection, params, Result as SqlResult};
use std::path::Path;
use std::collections::HashMap;
use log::info;
use crate::models::{HistoryRecord, AppMinuteRecord, UserKeymap, KeymapBrief, MinuteStatRecord, MinuteAppRecord};

pub fn init_database<P: AsRef<Path>>(db_path: P) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    // Create events table — matches Node.js schema for /historyData (today's data)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            keyname TEXT,
            keycount INTEGER,
            date TEXT,
            tick INTEGER
        )",
        [],
    ).map_err(|e| e.to_string())?;

    // Create stat table — matches Node.js schema for /historyData (historical aggregated data)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stat (
            keyname TEXT,
            keycount INTEGER,
            date TEXT
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
    conn.execute("CREATE INDEX IF NOT EXISTS idx_events_date ON events(date)", [])
        .map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_stat_date ON stat(date)", [])
        .map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_app_minutes_date ON app_minutes(date)", [])
        .map_err(|e| e.to_string())?;
    
    // Create statFreq table — matches Node.js schema for /minuteData (non-app)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS statFreq (
            keyTime TEXT, 
            keyCount INTEGER, 
            mouseCount INTEGER, 
            distance INTEGER,  
            freqType INTEGER,
            date TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS statFreq_date_IDX ON statFreq (date)", [])
        .map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS statFreq_type_IDX ON statFreq (freqType)", [])
        .map_err(|e| e.to_string())?;
    
    // Create appFreq table — matches Node.js schema for /minuteData (app)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS appFreq (
            keyTime TEXT, 
            appPath TEXT, 
            keyCount INTEGER, 
            mouseCount INTEGER, 
            freqType INTEGER,
            date TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS appFreq_date_IDX ON appFreq (date)", [])
        .map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS appFreq_type_IDX ON appFreq (freqType)", [])
        .map_err(|e| e.to_string())?;
    
    info!("Database initialized successfully");
    Ok(())
}

// ============== Key Records Operations (Node.js getRecords) ==============

/// Insert into `events` table — matches Node.js insertData: INSERT INTO events (keyname, keycount, tick, date)
pub fn insert_key_record(db_path: &str, keyname: &str, keycount: i64, tick: i64, date: &str) -> Result<i64, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO events (keyname, keycount, date, tick) VALUES (?1, ?2, ?3, ?4)",
        params![keyname, keycount, date, tick],
    ).map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

/// Matches Node.js getRecords(begin, end, newDbName).
/// - Today (begin == today && end == today): query `events` table → keyname, keycount, date, tick
/// - Single day (begin == end, not today): query `stat` table → keyname, keycount, date
/// - Date range (begin != end): query `stat` table with SUM+GROUP BY → keyname, sum(keycount), min(date)
/// Returns Vec<HistoryRecord> in Node.js format.
pub fn get_records(
    db_path: &str,
    begin: &str,
    end: &str,
) -> Result<Vec<HistoryRecord>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let is_today = begin == today.as_str() && end == today.as_str();

    let (sql, has_tick): (&str, bool) = if is_today {
        // Node.js: SELECT keyname, keycount, date, tick FROM events where date between ? and ?
        ("SELECT keyname, keycount, date, tick FROM events WHERE date BETWEEN ?1 AND ?2", true)
    } else if begin == end {
        // Node.js: SELECT keyname, keycount, date FROM stat where date between ? and ?
        ("SELECT keyname, keycount, date FROM stat WHERE date BETWEEN ?1 AND ?2", false)
    } else {
        // Node.js: SELECT keyname, sum(keycount) as keycount, min(date) as date FROM stat where date between ? and ? group by keyname
        ("SELECT keyname, sum(keycount) as keycount, min(date) as date FROM stat WHERE date BETWEEN ?1 AND ?2 GROUP BY keyname", false)
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let results = if has_tick {
        stmt.query_map(params![begin, end], |row| {
            Ok(HistoryRecord {
                keyname: row.get(0)?,
                keycount: row.get(1)?,
                date: row.get(2)?,
                tick: row.get::<_, Option<i64>>(3)?.map(|t| t.to_string()).unwrap_or_default(),
            })
        }).map_err(|e| e.to_string())?
          .collect::<SqlResult<Vec<_>>>()
          .map_err(|e| e.to_string())?
    } else {
        stmt.query_map(params![begin, end], |row| {
            Ok(HistoryRecord {
                keyname: row.get(0)?,
                keycount: row.get(1)?,
                date: row.get(2)?,
                tick: String::new(),
            })
        }).map_err(|e| e.to_string())?
          .collect::<SqlResult<Vec<_>>>()
          .map_err(|e| e.to_string())?
    };

    Ok(results)
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

// ============== Minute/App Frequency Operations (Node.js getMinuteRecords) ==============

/// Query statFreq (non-app) or appFreq (app) matching Node.js getMinuteRecords logic.
pub fn get_minute_records(
    db_path: &str,
    begin_date: Option<&str>,
    end_date: Option<&str>,
    freq_type: i32,
    is_app: bool,
) -> Result<Vec<serde_json::Value>, String> {
    use rusqlite::types::Value as SqlValue;

    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    if is_app {
        // Node.js: if (freqType > 0) sqlFreqType = 'freqType > 0'
        let sql_freq_type = if freq_type > 0 {
            "freqType > 0".to_string()
        } else {
            format!("freqType = {}", freq_type)
        };

        let (sql, params): (String, Vec<SqlValue>) = match (begin_date, end_date) {
            (Some(begin), Some(end)) => (
                format!(
                    "SELECT keyTime, keyCount, mouseCount, appPath, date, freqType \
                     FROM appFreq WHERE {} AND date BETWEEN ?1 AND ?2 ORDER BY date",
                    sql_freq_type
                ),
                vec![SqlValue::Text(begin.to_string()), SqlValue::Text(end.to_string())],
            ),
            _ => (
                format!(
                    "SELECT keyTime, keyCount, mouseCount, appPath, date, freqType \
                     FROM appFreq WHERE {} ORDER BY date",
                    sql_freq_type
                ),
                vec![],
            ),
        };

        let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(serde_json::to_value(MinuteAppRecord {
                minute: row.get(0)?,
                key_count: row.get(1)?,
                mouse_count: row.get(2)?,
                apps: row.get(3)?,
                date: row.get(4)?,
                duration: row.get(5)?,
            }).unwrap_or(serde_json::Value::Null))
        }).map_err(|e| e.to_string())?;

        Ok(rows.collect::<SqlResult<Vec<_>>>().map_err(|e| e.to_string())?)
    } else {
        // Node.js: sqlFreqType = `freqType = ${freqType} `
        let sql_freq_type = format!("freqType = {}", freq_type);

        let (sql, params): (String, Vec<SqlValue>) = match (begin_date, end_date) {
            (Some(begin), Some(end)) => (
                format!(
                    "SELECT keyTime, keyCount, mouseCount, distance, date \
                     FROM statFreq WHERE {} AND date BETWEEN ?1 AND ?2 ORDER BY date",
                    sql_freq_type
                ),
                vec![SqlValue::Text(begin.to_string()), SqlValue::Text(end.to_string())],
            ),
            _ => (
                format!(
                    "SELECT keyTime, keyCount, mouseCount, distance, date \
                     FROM statFreq WHERE {} ORDER BY date",
                    sql_freq_type
                ),
                vec![],
            ),
        };

        let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(serde_json::to_value(MinuteStatRecord {
                minute: row.get(0)?,
                key_count: row.get(1)?,
                mouse_count: row.get(2)?,
                distance: row.get(3)?,
                date: row.get(4)?,
            }).unwrap_or(serde_json::Value::Null))
        }).map_err(|e| e.to_string())?;

        Ok(rows.collect::<SqlResult<Vec<_>>>().map_err(|e| e.to_string())?)
    }
}

// ============== History Date Operations ==============

/// Matches Node.js getHistoryDate: query stat table for all distinct dates, descending.
/// Returns a flat array of date strings like ["2025-01-15", "2025-01-14", ...].
pub fn get_history_dates(db_path: &str) -> Result<Vec<String>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT date FROM stat group by date order by date desc"
    ).map_err(|e| e.to_string())?;

    let dates = stmt.query_map([], |row| {
        row.get::<_, String>(0)
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;

    Ok(dates)
}

// ============== Statistics Operations (Node.js statData) ==============

/// Matches Node.js statData(begin, end, newDbName):
/// 4 parallel queries on `stat` table, returns 4-element array:
///   [0] mouseDistance by date
///   [1] mouse + keyboard grouped by date
///   [2] top N keys
///   [3] top N apps
pub fn stat_data(
    db_path: &str,
    begin: &str,
    end: &str,
    top_n: i32,
    app_top_n: i32,
) -> Result<Vec<Vec<serde_json::Value>>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // pro1: mouseDistance by date
    let sql1 = "SELECT keycount, date FROM stat WHERE date BETWEEN ?1 AND ?2 AND keyname = 'mouseDistance' ORDER BY date";
    let mut stmt1 = conn.prepare(sql1).map_err(|e| e.to_string())?;
    let pro1: Vec<serde_json::Value> = stmt1.query_map(params![begin, end], |row| {
        Ok(serde_json::json!({
            "keycount": row.get::<_, i64>(0)?,
            "date": row.get::<_, String>(1)?,
        }))
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;

    // pro2: mouse + keyboard grouped by date
    let sql2 = "SELECT sum(keycount) as keycount, date, 'mouse' as keyname FROM stat
        WHERE date BETWEEN ?1 AND ?2 AND keyname IN ('LButton','RButton','MButton','WheelDown','WheelUp') GROUP BY date
        UNION
        SELECT sum(keycount) as keycount, date, 'keyboard' FROM stat
        WHERE date BETWEEN ?1 AND ?2 AND keyname NOT IN ('mouseDistance','LButton','RButton','MButton','WheelDown','WheelUp')
        AND keyname NOT LIKE 'App-%'
        GROUP BY date";
    let mut stmt2 = conn.prepare(sql2).map_err(|e| e.to_string())?;
    let pro2: Vec<serde_json::Value> = stmt2.query_map(params![begin, end, begin, end], |row| {
        Ok(serde_json::json!({
            "keycount": row.get::<_, i64>(0)?,
            "date": row.get::<_, String>(1)?,
            "keyname": row.get::<_, String>(2)?,
        }))
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;

    // pro3: top N keys
    let sql3 = format!(
        "WITH temp AS (SELECT * FROM stat WHERE date BETWEEN ?1 AND ?2)
         SELECT keycount, date, keyname FROM temp WHERE keyname IN
         (SELECT keyname FROM temp WHERE keyname NOT IN ('mouseDistance','LButton','RButton','MButton','WheelDown','WheelUp')
          AND keyname NOT LIKE 'App-%'
          GROUP BY keyname ORDER BY sum(keycount) DESC LIMIT {})
         ORDER BY date",
        top_n
    );
    let mut stmt3 = conn.prepare(&sql3).map_err(|e| e.to_string())?;
    let pro3: Vec<serde_json::Value> = stmt3.query_map(params![begin, end], |row| {
        Ok(serde_json::json!({
            "keycount": row.get::<_, i64>(0)?,
            "date": row.get::<_, String>(1)?,
            "keyname": row.get::<_, String>(2)?,
        }))
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;

    // pro4: top N apps
    let sql4 = format!(
        "WITH temp AS (
            SELECT * FROM stat WHERE keyname LIKE 'App-%' AND date BETWEEN ?1 AND ?2
        ),
        temp2 AS (
            SELECT REPLACE(REPLACE(keyname,'App-Mouse-',''),'App-Key-','') as pathname, sum(keycount) as count
            FROM temp GROUP BY pathname ORDER BY count DESC LIMIT {}
        )
        SELECT * FROM temp WHERE keyname IN (SELECT 'App-Mouse-' || pathname FROM temp2)
        UNION
        SELECT * FROM temp WHERE keyname IN (SELECT 'App-Key-' || pathname FROM temp2)",
        app_top_n
    );
    let mut stmt4 = conn.prepare(&sql4).map_err(|e| e.to_string())?;
    let pro4: Vec<serde_json::Value> = stmt4.query_map(params![begin, end], |row| {
        Ok(serde_json::json!({
            "keyname": row.get::<_, String>(0)?,
            "keycount": row.get::<_, i64>(1)?,
            "date": row.get::<_, String>(2)?,
        }))
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;

    Ok(vec![pro1, pro2, pro3, pro4])
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
    
    conn.execute("DELETE FROM events WHERE date = ?1", params![date])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM stat WHERE date = ?1", params![date])
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


