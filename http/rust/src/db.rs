use rusqlite::{Connection, params, Result as SqlResult};
use std::path::Path;
use std::collections::HashMap;
use log::info;
use crate::models::{HistoryRecord, UserKeymap, KeymapBrief, MinuteStatRecord, MinuteAppRecord};

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

    // Create appStat table — matches Node.js updateDBStruct() (application usage minutes)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS appStat (
            appPath TEXT,
            minuteCount REAL,
            date TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS appStat_date_IDX ON appStat (date)", [])
        .map_err(|e| e.to_string())?;
    
    info!("Database initialized successfully");
    Ok(())
}

// ============== Key Records Operations (Node.js getRecords) ==============

/// Matches Node.js insertData(records):
/// - tick null/0 → skip
/// - DELETE FROM events WHERE tick = ? (dedupe)
/// - every numeric field (except tick) → one row (keyname, keycount, tick, date)
/// - date = local date from tick timestamp (dayjs(new Date(tick)).format('YYYY-MM-DD'))
/// - skip if stat table already has this date (already summarized)
pub fn insert_key_records(db_path: &str, records: &serde_json::Value) -> Result<i64, String> {
    let tick = records.get("tick").and_then(|v| v.as_i64()).unwrap_or(0);
    if tick == 0 {
        return Ok(0);
    }

    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // Node.js: delete FROM events where tick = ?
    conn.execute("DELETE FROM events WHERE tick = ?1", params![tick])
        .map_err(|e| e.to_string())?;

    // Collect numeric fields (typeof records[k] == 'number', k != 'tick')
    let mut arr: Vec<(String, f64)> = Vec::new();
    if let Some(obj) = records.as_object() {
        for (k, v) in obj {
            if k != "tick" {
                if let Some(n) = v.as_f64() {
                    arr.push((k.clone(), n));
                }
            }
        }
    }
    if arr.is_empty() {
        return Ok(0);
    }

    // date from tick (local timezone, matching new Date(tick))
    use chrono::TimeZone;
    let tick_date = chrono::Local
        .timestamp_millis_opt(tick)
        .single()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    // Node.js: skip if stat already summarized this date
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stat WHERE date = ?1 LIMIT 1",
            params![tick_date],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if exists > 0 {
        info!("insertData: Exist rows in {}", tick_date);
        return Ok(0);
    }

    let mut changes: i64 = 0;
    for (name, count) in &arr {
        changes += conn
            .execute(
                "INSERT INTO events (keyname, keycount, tick, date) VALUES (?1, ?2, ?3, ?4)",
                params![name, count, tick, tick_date],
            )
            .map_err(|e| e.to_string())? as i64;
    }
    info!("insertData: {} rows inserted (tick={}, date={})", changes, tick, tick_date);
    Ok(changes)
}

/// 'YYYYMMDDHHmm' → 'YYYY-MM-DD' (matches Node.js convert2Date)
fn minute_str_to_date(minute: &str) -> String {
    if minute.len() >= 8 {
        format!("{}-{}-{}", &minute[0..4], &minute[4..6], &minute[6..8])
    } else {
        String::new()
    }
}

/// Matches Node.js insertMiniute(records):
/// - statFreq: (keyTime=Minute, keyCount, mouseCount, distance, freqType=0, date)
/// - appFreq: (keyTime, appPath, keyCount=App.Key, mouseCount=App.Mouse, freqType=0, date)
///   for each entry in Apps (skip empty-string keys)
pub fn insert_minute_records(db_path: &str, records: &[serde_json::Value]) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    for r in records {
        let minute = r.get("Minute").and_then(|v| v.as_str()).unwrap_or("");
        if minute.is_empty() {
            continue;
        }
        let date = minute_str_to_date(minute);
        let key_count = r.get("KeyCount").and_then(|v| v.as_i64()).unwrap_or(0);
        let mouse_count = r.get("MouseCount").and_then(|v| v.as_i64()).unwrap_or(0);
        let distance = r.get("Distance").and_then(|v| v.as_i64()).unwrap_or(0);

        conn.execute(
            "INSERT INTO statFreq (keyTime, keyCount, mouseCount, distance, freqType, date) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
            params![minute, key_count, mouse_count, distance, date],
        ).map_err(|e| e.to_string())?;

        // Apps: { appPath: { Key, Mouse } }
        if let Some(apps) = r.get("Apps").and_then(|v| v.as_object()) {
            for (app_path, app) in apps {
                if app_path.is_empty() {
                    continue; // 为空值时跳过
                }
                let app_key = app.get("Key").and_then(|v| v.as_i64()).unwrap_or(0);
                let app_mouse = app.get("Mouse").and_then(|v| v.as_i64()).unwrap_or(0);
                conn.execute(
                    "INSERT INTO appFreq (keyTime, appPath, keyCount, mouseCount, freqType, date) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
                    params![minute, app_path, app_key, app_mouse, date],
                ).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
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
                tick: row.get::<_, Option<i64>>(3)?,
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
                tick: None,
            })
        }).map_err(|e| e.to_string())?
          .collect::<SqlResult<Vec<_>>>()
          .map_err(|e| e.to_string())?
    };

    Ok(results)
}

// ============== App Minutes Operations (Node.js getAppMinute) ==============

/// Format a REAL/INTEGER SQLite number the way Node.js would: integral values
/// serialize without a fractional part (161.0 → 161, 161.32 → 161.32).
fn js_num(v: f64) -> serde_json::Value {
    if v.fract() == 0.0 && v.abs() < 9.007_199_254_740_992e15 {
        serde_json::json!(v as i64)
    } else {
        serde_json::Value::from(v)
    }
}

/// Matches Node.js getAppMinute(beginDate, endDate, isTotal, newDbName):
/// - is_total truthy → per-date totals from `appStat`:
///     `{ "Date": date, "Minutes": sum(minuteCount), "AppCount": count(1) }`
/// - else → per-date top `app_top_n` apps by minutes:
///     `{ "Apps": appPath, "Date": date, "Minutes": minuteCount }`
/// NOTE: `app_top_n` corresponds to Node.js `~~globalAppTopN` (dataSetting2.appTopN, default 10).
pub fn get_app_minute(
    db_path: &str,
    begin_date: Option<&str>,
    end_date: Option<&str>,
    is_total: bool,
    app_top_n: i64,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // Node.js: beginDate ?? '' / endDate ?? ''
    let begin = begin_date.unwrap_or("");
    let end = end_date.unwrap_or("");

    let mut out: Vec<serde_json::Value> = Vec::new();

    if is_total {
        // select sum(minuteCount) as minuteCount,date,count(1) as appCount from appStat
        //   where date between ? and ? group by date
        let sql = "SELECT sum(minuteCount) AS minuteCount, date, count(1) AS appCount \
                   FROM appStat WHERE date BETWEEN ? AND ? GROUP BY date";
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![begin, end], |row| {
                let minutes: Option<f64> = row.get(0)?;
                Ok(serde_json::json!({
                    "Date": row.get::<_, Option<String>>(1)?,
                    "Minutes": js_num(minutes.unwrap_or(0.0)),
                    "AppCount": row.get::<_, i64>(2)?,
                }))
            })
            .map_err(|e| e.to_string())?;
        for r in rows {
            out.push(r.map_err(|e| e.to_string())?);
        }
    } else {
        // WITH ranked_data AS (
        //   SELECT date, appPath, minuteCount,
        //          ROW_NUMBER() OVER (PARTITION BY date ORDER BY minuteCount DESC) AS rank_num
        //   FROM appStat where date between ? and ?)
        // SELECT date, appPath, minuteCount FROM ranked_data WHERE rank_num <= ? ORDER BY date
        let sql = "WITH ranked_data AS ( \
                       SELECT date, appPath, minuteCount, \
                              ROW_NUMBER() OVER (PARTITION BY date ORDER BY minuteCount DESC) AS rank_num \
                       FROM appStat WHERE date BETWEEN ? AND ? \
                   ) \
                   SELECT date, appPath, minuteCount FROM ranked_data \
                   WHERE rank_num <= ? ORDER BY date";
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![begin, end, app_top_n], |row| {
                let minutes: Option<f64> = row.get(2)?;
                Ok(serde_json::json!({
                    "Apps": row.get::<_, Option<String>>(1)?,
                    "Date": row.get::<_, Option<String>>(0)?,
                    "Minutes": js_num(minutes.unwrap_or(0.0)),
                }))
            })
            .map_err(|e| e.to_string())?;
        for r in rows {
            out.push(r.map_err(|e| e.to_string())?);
        }
    }

    Ok(out)
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
/// Empty / NULL dates are filtered out.
pub fn get_history_dates(db_path: &str) -> Result<Vec<String>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT date FROM stat group by date order by date desc"
    ).map_err(|e| e.to_string())?;

    let dates: Vec<String> = stmt.query_map([], |row| {
        row.get::<_, Option<String>>(0)
    }).map_err(|e| e.to_string())?
      .filter_map(|r| r.ok())
      .flatten()
      .map(|d| d.trim().to_string())
      .filter(|d| !d.is_empty())
      .collect();

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
    // NOTE: use plain `?` placeholders (not ?1/?2) so the bind order matches
    // Node.js sqlite3, which binds parameters strictly left-to-right.
    let sql1 = "SELECT keycount, date FROM stat WHERE date BETWEEN ? AND ? AND keyname = 'mouseDistance' ORDER BY date";
    let mut stmt1 = conn.prepare(sql1).map_err(|e| e.to_string())?;
    let pro1: Vec<serde_json::Value> = stmt1.query_map(params![begin, end], |row| {
        Ok(serde_json::json!({
            "keycount": row.get::<_, i64>(0)?,
            "date": row.get::<_, String>(1)?,
        }))
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;

    // pro2: mouse + keyboard grouped by date (4 placeholders, Node.js binds begin,end,begin,end)
    let sql2 = "SELECT sum(keycount) as keycount, date, 'mouse' as keyname FROM stat
        WHERE date BETWEEN ? AND ? AND keyname IN ('LButton','RButton','MButton','WheelDown','WheelUp') GROUP BY date
        UNION
        SELECT sum(keycount) as keycount, date, 'keyboard' FROM stat
        WHERE date BETWEEN ? AND ? AND keyname NOT IN ('mouseDistance','LButton','RButton','MButton','WheelDown','WheelUp')
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
        "WITH temp AS (SELECT * FROM stat WHERE date BETWEEN ? AND ?)
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
            SELECT * FROM stat WHERE keyname LIKE 'App-%' AND date BETWEEN ? AND ?
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

/// Matches Node.js deleteData(date, flag) in records.js:
/// - flag == 0: DELETE FROM events WHERE tick IN (dates)   — dates are tick values
/// - flag == 1: DELETE FROM stat / statFreq / appFreq WHERE date IN (dates)
/// Only operates on the main db (records.db), matches Node.js `new sqlite3.Database(dbName)`.
pub fn delete_data(db_path: &str, dates: &[String], flag: i64) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let placeholders = dates.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    match flag {
        0 => {
            let sql = format!("DELETE FROM events WHERE tick IN ({})", placeholders);
            conn.execute(&sql, rusqlite::params_from_iter(dates.iter()))
                .map_err(|e| e.to_string())?;
        }
        1 => {
            for table in ["stat", "statFreq", "appFreq"] {
                let sql = format!("DELETE FROM {} WHERE date IN ({})", table, placeholders);
                conn.execute(&sql, rusqlite::params_from_iter(dates.iter()))
                    .map_err(|e| e.to_string())?;
            }
        }
        _ => return Err(format!("Unknown flag: {}", flag)),
    }

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


