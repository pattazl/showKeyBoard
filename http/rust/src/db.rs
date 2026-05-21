use rusqlite::{Connection, params, Result as SqlResult};
use std::path::Path;
use log::info;
use crate::models::{KeyRecord, AppMinuteRecord, HistoryDate, StatRecord, UserKeymap};

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
    
    // Create keymaps table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS keymaps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            data TEXT,
            date TEXT
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

pub fn insert_keymap(db_path: &str, name: &str, data: &str) -> Result<i64, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    
    conn.execute(
        "INSERT INTO keymaps (name, data, date) VALUES (?1, ?2, ?3)",
        params![name, data, date],
    ).map_err(|e| e.to_string())?;
    
    Ok(conn.last_insert_rowid())
}

pub fn update_keymap(db_path: &str, id: i64, name: &str, data: &str) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    
    conn.execute(
        "UPDATE keymaps SET name = ?1, data = ?2, date = ?3 WHERE id = ?4",
        params![name, data, date, id],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn delete_keymap(db_path: &str, id: i64) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM keymaps WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn get_keymaps(db_path: &str) -> Result<Vec<UserKeymap>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT id, name, data, date FROM keymaps ORDER BY id DESC"
    ).map_err(|e| e.to_string())?;
    
    let keymaps = stmt.query_map([], |row| {
        Ok(UserKeymap {
            id: row.get(0)?,
            name: row.get(1)?,
            data: row.get(2)?,
            date: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?
      .collect::<SqlResult<Vec<_>>>()
      .map_err(|e| e.to_string())?;
    
    Ok(keymaps)
}

// ============== PC Info Operations ==============

pub fn insert_pc_info(db_path: &str, display: &str, resolution: &str, cpu_info: &str) -> Result<i64, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    
    conn.execute(
        "INSERT INTO pc_info (display, resolution, cpu_info, date) VALUES (?1, ?2, ?3, ?4)",
        params![display, resolution, cpu_info, date],
    ).map_err(|e| e.to_string())?;
    
    Ok(conn.last_insert_rowid())
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
