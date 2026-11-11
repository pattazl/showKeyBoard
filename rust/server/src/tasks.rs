//! 后台定时任务，对齐 Node.js 侧：
//! 1. doCleanData（records.js）—— /data 插入后 3 秒整理每日数据到历史表（events→stat、
//!    statFreq/appFreq 保留天数清理、appStat 汇总、24 小时统计生成）
//! 2. autoSaveFun / saveLastData（common.js）—— 每 autoSave2Db 秒把 preData 落库
//! 3. autoShare（shareDb.js）—— 每 60 秒检查：按 shareDbHour 周期打包 records.db 生成
//!    共享 zip（<shareDbName>_<时间戳>.db），并解压其它机器的共享库到 dbs/ 目录
#![allow(dead_code)]
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Local};
use futures_util::FutureExt;
use log::{info, warn};

use crate::config::AppConfig;
use crate::db;
use crate::handlers;
use crate::AppState;

/// 启动全部后台任务
pub fn start(state: AppState) {
    tokio::spawn(clean_task(state.clone()));
    tokio::spawn(auto_save_task(state.clone()));
    tokio::spawn(auto_share_task(state));
}

/// 重新从磁盘读取配置（对齐 Node：setPara 更新的是全局 config 对象，
/// getPara/setPara 均以磁盘 ini 为准；此处同样每次读取最新值）
async fn read_config(state: &AppState) -> AppConfig {
    let (desc, user) = {
        let sg = state.read().await;
        (sg.desc_ini_path.clone(), sg.user_ini_path.clone())
    };
    AppConfig::load_with_override(&desc, &user)
}

/// sqlite date() 的时区修正串（Node: -(new Date().getTimezoneOffset()/60) + ' hours'）
/// 例如 UTC+8 → "8 hours"，UTC-5.5 → "-5.5 hours"
fn tz_string() -> String {
    let secs = Local::now().offset().local_minus_utc(); // 东时区为正秒
    let hours = secs as f64 / 3600.0;
    if hours.fract() == 0.0 {
        format!("{} hours", hours as i64)
    } else {
        format!("{} hours", hours)
    }
}

// ============== 1. doCleanData ==============

async fn clean_task(state: AppState) {
    // 启动后先执行一次（Node 启动时 patchLastData 也会触发 mergeAppStat/cleanErrStat）
    tokio::time::sleep(Duration::from_secs(3)).await;
    do_clean_data(&state).await;

    let notify = state.read().await.clean_notify.clone();
    loop {
        notify.notified().await;
        // Node.js: 插入数据后 setTimeout(doCleanData, 3000) —— 延迟 3 秒再整理，
        // 期间若再次触发则合并为一次
        tokio::time::sleep(Duration::from_secs(3)).await;
        while notify.notified().now_or_never().is_some() {}
        do_clean_data(&state).await;
    }
}

async fn do_clean_data(state: &AppState) {
    let db_path = {
        let sg = state.read().await;
        sg.db_path.clone()
    };
    // Node: let before = dataSetting.minuteKeepDays ?? 7
    let keep_days: i64 = db::get_data_setting(&db_path)
        .ok()
        .and_then(|m| m.get("minuteKeepDays").cloned())
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(7);
    let str_now = Local::now().format("%Y-%m-%d").to_string();
    let before_days = (Local::now() - ChronoDuration::days(keep_days))
        .format("%Y-%m-%d")
        .to_string();
    let yesterday = (Local::now() - ChronoDuration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let tz = tz_string();

    let db_path_c = db_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        db::clean_daily_data(&db_path_c, &str_now, &before_days, &yesterday, &tz)
    })
    .await;

    match result {
        Ok(Ok(report)) => {
            if !report.is_empty() {
                info!("{}", report);
            }
        }
        Ok(Err(e)) => warn!("doCleanData error: {}", e),
        Err(e) => warn!("doCleanData join error: {}", e),
    }
}

// ============== 2. autoSave2Db / saveLastData ==============

async fn auto_save_task(state: AppState) {
    let cfg = read_config(&state).await;
    // Node: interval = config.common?.autoSave2Db ?? 180（秒）
    let interval: i64 = cfg
        .common
        .get("autoSave2Db")
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(180);
    if interval <= 0 {
        info!("autoSave2Db = {}, auto save disabled", interval);
        return;
    }
    info!("autoSave task started, interval = {}s", interval);
    let dur = Duration::from_secs(interval as u64);
    loop {
        tokio::time::sleep(dur).await;
        // Node: saveLastData() —— preData.tick > 0 时 insertDataFun(preData) 并置 tick=0
        handlers::save_last_data(&state).await;
    }
}

// ============== 3. autoShare ==============

async fn auto_share_task(state: AppState) {
    // Node: autoShare() 启动时立即执行一次 autoShareCore，再每 60 秒检查
    let mut first = true;
    loop {
        if !first {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
        first = false;
        let cfg = read_config(&state).await;
        // Node: let hour = config.common.shareDbHour; if (hour > 0)
        let hour: f64 = cfg
            .common
            .get("shareDbHour")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0.0);
        if hour > 0.0 {
            if let Err(e) = auto_share_core(&state, &cfg, hour).await {
                warn!("autoShare error: {}", e);
            }
        }
    }
}

/// 把 ini 中可能来自 Windows 的反斜杠路径转换为当前平台分隔符
#[cfg(windows)]
fn portable_path(p: &str) -> String {
    p.to_string()
}

#[cfg(not(windows))]
fn portable_path(p: &str) -> String {
    p.replace('\\', "/")
}

/// Node: path.resolve(basePath, xxx) —— 绝对路径直接用，否则基于用户数据目录
fn resolve_path(base: &str, p: &str) -> PathBuf {
    let p = portable_path(p);
    let path = Path::new(&p);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(base).join(&p)
    }
}

/// path 是否位于 root 目录之内（Windows 下大小写不敏感；不存在的路径按字面比较）
fn is_inside(path: &Path, root: &Path) -> bool {
    let norm = |p: &Path| -> String {
        let p = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        let s = p.to_string_lossy().replace('/', "\\");
        if cfg!(windows) { s.to_lowercase() } else { s }
    };
    let p = norm(path);
    let r = norm(root);
    if p == r {
        return true;
    }
    let prefix = if r.ends_with('\\') { r } else { format!("{}\\", r) };
    p.starts_with(&prefix)
}

async fn auto_share_core(state: &AppState, cfg: &AppConfig, hour: f64) -> Result<(), String> {
    let (data_dir, base_dir, db_path) = {
        let sg = state.read().await;
        (sg.data_dir.clone(), sg.base_dir.clone(), sg.db_path.clone())
    };

    // shareDbName 为空时用主机名（与 get_para 的后处理一致）
    let mut share_db_name = cfg
        .common
        .get("shareDbName")
        .cloned()
        .unwrap_or_default();
    if share_db_name.trim().is_empty() {
        share_db_name = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "noname".to_string());
    }
    let share_db_path = cfg.common.get("shareDbPath").cloned().unwrap_or_default();
    let share_db_exec = cfg.common.get("shareDbExec").cloned().unwrap_or_default();

    // 递归创建共享目录（相对路径基于用户数据目录，默认 share/）
    let share_db_path = if share_db_path.trim().is_empty() {
        "share".to_string()
    } else {
        share_db_path
    };
    let mut share_file_dir = resolve_path(&data_dir, &share_db_path);
    // shareDbPath 不允许位于安装目录内：命中则回退到 <data_dir>/share
    if is_inside(&share_file_dir, Path::new(&base_dir))
        && !is_inside(Path::new(&data_dir), Path::new(&base_dir))
    {
        let fallback = Path::new(&data_dir).join("share");
        info!(
            "autoShare: shareDbPath {:?} 位于安装目录内，改用 {:?}",
            share_file_dir, fallback
        );
        share_file_dir = fallback;
    }
    std::fs::create_dir_all(&share_file_dir)
        .map_err(|e| format!("目录创建失败: {}", e))?;
    let share_file_name = share_file_dir.join(format!("{}.zip", share_db_name));

    // 判断是否需要重新打包：不存在，或修改时间超过 shareDbHour 小时
    let mut need_share = false;
    if share_file_name.exists() {
        if let Ok(meta) = std::fs::metadata(&share_file_name) {
            if let Ok(mtime) = meta.modified() {
                let diff = std::time::SystemTime::now()
                    .duration_since(mtime)
                    .map(|d| d.as_millis() as f64)
                    .unwrap_or(0.0);
                if diff > hour * 3600.0 * 1000.0 {
                    need_share = true; // 文件旧，需要重新备份
                }
            }
        }
    } else {
        need_share = true; // 不存在，需要备份新文件
    }

    if need_share {
        // 将当前 records.db 压缩为 <shareDbName>_<unix秒>.db（Node zipCore(isShare=true)）
        let zip_path = share_file_name.to_string_lossy().to_string();
        let db_path_c = db_path.clone();
        let entry_name = format!("{}_{}.db", share_db_name, chrono::Utc::now().timestamp());
        let entry_name_c = entry_name.clone();
        let zip_res = tokio::task::spawn_blocking(move || {
            zip_db_to(&db_path_c, &entry_name_c, &zip_path)
        })
        .await
        .map_err(|e| e.to_string())?;
        zip_res.map_err(|e| format!("压缩当前数据失败: {}", e))?;
        info!("autoShare: packaged {} into {:?}", entry_name, share_file_name);

        // 执行额外的同步命令（Node doExec，cwd = basePath）
        let cmd_line = share_db_exec.trim().to_string();
        if !cmd_line.is_empty() {
            info!("autoShare exec: {}", cmd_line);
            spawn_command(&cmd_line, Path::new(&base_dir));
        }
    }

    // 解压共享文件到 dbs/ 目录（用户数据目录下）
    unzip_shared(&data_dir, &share_file_dir, &share_db_name).await
}

/// 压缩 records.db 到 zip（zip 内文件名为 entry_name，DEFLATE level 5）
fn zip_db_to(db_path: &str, entry_name: &str, zip_path: &str) -> Result<(), String> {
    use std::io::Write;
    let data = std::fs::read(db_path).map_err(|e| e.to_string())?;
    let file = std::fs::File::create(zip_path).map_err(|e| e.to_string())?;
    let mut zipw = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(5)); // Node.js JSZip level 5
    zipw.start_file(entry_name, opts)
        .and_then(|_| zipw.write_all(&data).map_err(zip::result::ZipError::Io))
        .map_err(|e| e.to_string())?;
    zipw.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// 从 db 文件名（形如 `<设备名>_1731111404.db`）提取 `(设备名, unix秒时间戳)`。
/// 设备名本身可含 `_`（如 `DESKTOP-ABC_1731111404.db`），所以从右往左找**最后一个** `_`。
/// 时间戳部分必须能 parse 为 i64，否则视为格式不对返回 None。
fn parse_db_filename(name: &str) -> Option<(String, i64)> {
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
    let (device, ts_str) = stem.rsplit_once('_')?;
    let ts = ts_str.parse::<i64>().ok()?;
    if device.is_empty() {
        return None;
    }
    Some((device.to_string(), ts))
}

/// 打开 zip 包，提取内部 db 文件名。
/// 成功返回 `(完整文件名, 设备名, unix时间戳)`；失败或无 db 条目返回 Err。
fn peek_zip_db_entry(zip_path: &Path) -> Result<(String, String, i64), String> {
    let f = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| e.to_string())?;
        if file.is_dir() {
            continue;
        }
        let name = file.name().to_string();
        if name.to_lowercase().ends_with(".db") {
            match parse_db_filename(&name) {
                Some((device, ts)) => return Ok((name, device, ts)),
                None => return Err(format!("内部 db 文件名格式不对: {}", name)),
            }
        }
    }
    Err("zip 内未找到 .db 文件".into())
}

/// 扫描 dbs 目录，返回 `设备名 -> (完整文件名, unix时间戳)` 映射。
fn collect_dbs_info(dbs_path: &Path) -> std::collections::HashMap<String, (String, i64)> {
    let mut info = std::collections::HashMap::new();
    let entries = match std::fs::read_dir(dbs_path) {
        Ok(e) => e,
        Err(_) => return info,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.to_lowercase().ends_with(".db") {
            continue;
        }
        if let Some((device, ts)) = parse_db_filename(&name) {
            // 同设备名有多份时保留时间戳较大的（新的）
            info.entry(device)
                .and_modify(|existing| {
                    if ts > existing.1 {
                        *existing = (name.clone(), ts);
                    }
                })
                .or_insert((name, ts));
        }
    }
    info
}

/// Node: unzipShared —— 扫描共享目录全部 *.zip（跳过自己的），
/// 打开 zip 看内部 db 文件名的时间戳，与 dbs/ 目录下同设备名的 db 比较，
/// 较新则删除旧库并解压覆盖，较旧则跳过。
async fn unzip_shared(
    data_dir: &str,
    share_file_dir: &Path,
    own_name: &str,
) -> Result<(), String> {
    let dbs_path = Path::new(data_dir).join("dbs/");
    std::fs::create_dir_all(&dbs_path).map_err(|e| format!("目录创建失败: {}", e))?;

    // dbs 目录现有 db 信息：设备名 -> (完整文件名, unix时间戳)
    let dbs_info = collect_dbs_info(&dbs_path);

    // share 目录所有 *.zip
    let zip_paths: Vec<PathBuf> = std::fs::read_dir(share_file_dir)
        .map_err(|e| format!("目录读取失败: {}", e))?
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.is_file() && p.to_string_lossy().to_lowercase().ends_with(".zip") {
                Some(p)
            } else {
                None
            }
        })
        .collect();

    for zip_path in zip_paths {
        // 跳过自己机器的 zip（<shareDbName>.zip）
        let zip_stem = zip_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if zip_stem == own_name {
            continue;
        }

        // 打开 zip 看内部 db 文件名（主线程 peek，zip::ZipArchive 打开很快）
        let (entry_name, device, entry_ts) = match peek_zip_db_entry(&zip_path) {
            Ok(info) => info,
            Err(e) => {
                warn!("unzip_shared: 无法解析 {:?}: {}", zip_path, e);
                continue;
            }
        };

        // 和 dbs 里同设备名的现有 db 比较时间戳
        let should_extract = match dbs_info.get(&device) {
            Some((existing_name, existing_ts)) => {
                if entry_ts > *existing_ts {
                    info!(
                        "unzip_shared: {} 比 dbs 里的 {} 新 ({} > {})，覆盖",
                        entry_name, existing_name, entry_ts, existing_ts
                    );
                    true
                } else {
                    info!(
                        "unzip_shared: {} 不比 dbs 里的 {} 新 ({} <= {})，跳过",
                        entry_name, existing_name, entry_ts, existing_ts
                    );
                    false
                }
            }
            None => {
                info!(
                    "unzip_shared: dbs 里没有设备 {}，解压 {}",
                    device, entry_name
                );
                true
            }
        };

        if !should_extract {
            continue;
        }

        // 删掉 dbs 里同设备名的旧 db（如果存在）
        let old_db = dbs_info.get(&device).map(|(n, _)| dbs_path.join(n));
        let zip_file = zip_path.clone();
        let dbs_dir = dbs_path.clone();
        tokio::task::spawn_blocking(move || {
            if let Some(old) = old_db {
                if old.exists() {
                    let _ = std::fs::remove_file(&old);
                }
            }
            unzip_to_dbs(&zip_file, &dbs_dir)
        })
        .await
        .map_err(|e| e.to_string())??;
    }
    Ok(())
}

/// 解压 zip 内全部文件到 dbs 目录（对应 Node unZipCore + fun(filename)）。
/// 调用方已在入口侧按时间戳比较、删旧文件，这里无条件写入覆盖。
fn unzip_to_dbs(zip_path: &Path, dbs_dir: &Path) -> Result<(), String> {
    use std::io::Read;
    let f = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        if file.is_dir() {
            continue;
        }
        let name = file.name().to_string();
        let dest = dbs_dir.join(&name);
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(|e| e.to_string())?;
        std::fs::write(&dest, &data).map_err(|e| e.to_string())?;
        info!("unzip_to_dbs: 写入 {:?}", dest);
    }
    Ok(())
}

/// 执行共享后的额外命令（Node: exec(command, { cwd: basePath })）
/// Windows 下经 cmd /C（CREATE_NO_WINDOW 避免弹窗）；macOS/Linux 下经 /bin/sh -c。
#[cfg(windows)]
fn spawn_command(command: &str, cwd: &Path) {
    use std::os::windows::process::CommandExt;
    let _ = std::process::Command::new("cmd")
        .args(["/C", command])
        .current_dir(cwd)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn();
}

#[cfg(not(windows))]
fn spawn_command(command: &str, cwd: &Path) {
    let _ = std::process::Command::new("/bin/sh")
        .args(["-c", command])
        .current_dir(cwd)
        .spawn();
}

// ============== 4. autoBackup（Node.js common.js autoBackup） ==============

/// 供 handlers::send_pc_info 在客户端启动时触发备份（Node.js: sendPCInfo flag==0 → autoBackup()）
pub async fn auto_backup(state: AppState) {
    // 提前 clone 路径，避免在 spawn_blocking 中持有 RwLock
    let (data_dir, db_path, user_ini_path, key_list_path) = {
        let sg = state.read().await;
        (
            sg.data_dir.clone(),
            sg.db_path.clone(),
            sg.user_ini_path.clone(),
            sg.key_list_path.clone(),
        )
    };
    let result = tokio::task::spawn_blocking(move || {
        backup_core(&data_dir, &db_path, &user_ini_path, &key_list_path)
    })
    .await;
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => warn!("autoBackup error: {}", e),
        Err(e) => warn!("autoBackup join error: {}", e),
    }
}

/// 同步备份核心（Node.js common.js autoBackup）：
/// 1. 从 dataSetting.autoBackupDays 读取保留天数，默认 3
/// 2. 创建 <data_dir>/backup/ 目录
/// 3. 清理 backup/ 下早于 (now - days) 的 skb_*.zip 文件
/// 4. 取最新备份文件名，若 (now - 3min) 晚于最新备份 → 生成新的 skb_YYYY-MM-DD-HH-mm-ss.zip
/// 5. zip 内容：showKeyBoard.ini + keyList.txt + records.db
fn backup_core(
    data_dir: &str,
    db_path: &str,
    user_ini_path: &str,
    key_list_path: &str,
) -> Result<(), String> {

    // 1. 读取保留天数（Node: parseInt(dataSetting.autoBackupDays ?? 3, 10)）
    let days: i64 = db::get_data_setting(&db_path)
        .ok()
        .and_then(|m| m.get("autoBackupDays").cloned())
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(3);

    // 日期格式化
    let fmt_full = "%Y-%m-%d-%H-%M-%S";
    let now = Local::now();
    let current_file = format!("skb_{}.zip", now.format(fmt_full)); // skb_YYYY-MM-DD-HH-mm-ss.zip

    let backup_dir = Path::new(&data_dir).join("backup");
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("backup目录创建失败: {}", e))?;

    if days <= 0 {
        return Ok(()); // 不保留任何备份，无需后续操作
    }

    // 早于 (now - days) 的阈值文件名（按字符串比较）
    let before_str = format!(
        "skb_{}.zip",
        (now - ChronoDuration::days(days)).format(fmt_full)
    );
    // 3 分钟前的时间点，用于防重复备份
    let min_backup = format!(
        "skb_{}.zip",
        (now - ChronoDuration::minutes(3)).format(fmt_full)
    );

    // 2. 遍历 backup 目录
    let mut max_file = String::new();
    let entries = std::fs::read_dir(&backup_dir)
        .map_err(|e| format!("backup目录读取失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // 正则匹配：skb_YYYY-MM-DD-HH-mm-ss.zip
        if !is_backup_filename(&name) {
            continue;
        }

        // 清理过期文件（Node: file < beforeStr → unlink）
        if name < before_str {
            match std::fs::remove_file(&path) {
                Ok(_) => info!("autoBackup: 删除过期文件 {}", name),
                Err(e) => warn!("autoBackup: 删除文件 {} 失败: {}", name, e),
            }
            continue;
        }

        // 取最新备份文件（Node: file < currentFile 前提下取 max）
        if name.as_str() < current_file.as_str() && name.as_str() > max_file.as_str() {
            max_file = name;
        }
    }

    // 3. 只有 minBackUp > maxFile（即距上次备份超过 3 分钟）才生成新备份
    if min_backup.as_str() > max_file.as_str() {
        let zip_path = backup_dir.join(&current_file);
        match zip_backup_files(&user_ini_path, &key_list_path, &db_path, &zip_path) {
            Ok(_) => info!("autoBackup: 成功生成 {:?}", zip_path),
            Err(e) => warn!("autoBackup: 生成 {} 失败: {}", current_file, e),
        }
    } else {
        info!(
            "autoBackup: 距上次备份 {} 小于 3 分钟，跳过",
            max_file
        );
    }

    Ok(())
}

/// 判断文件名是否符合 skb_YYYY-MM-DD-HH-mm-ss.zip 格式
fn is_backup_filename(name: &str) -> bool {
    // "skb_" + 19 chars "YYYY-MM-DD-HH-mm-ss" + ".zip"
    // total length 4 + 19 + 4 = 27
    if name.len() != 27 {
        return false;
    }
    let bytes = name.as_bytes();
    if &name[0..4] != "skb_" || &name[23..] != ".zip" {
        return false;
    }
    // 检查中间部分 "YYYY-MM-DD-HH-mm-ss"
    // 索引：4-12 "YYYY-MM-DD"，13-18 "HH-mm-ss"
    // 分隔符：'-' 位置 8、11、14、17、20
    for &pos in &[8usize, 11, 14, 17, 20] {
        if bytes[pos] != b'-' {
            return false;
        }
    }
    // 数字检查
    for &pos in &[4usize, 5, 6, 7, 9, 10, 12, 13, 15, 16, 18, 19] {
        if !bytes[pos].is_ascii_digit() {
            return false;
        }
    }
    true
}

/// 将 showKeyBoard.ini + keyList.txt + records.db 压缩写入 zip_path
/// 对应 Node.js zipCore(cbFun, false)
fn zip_backup_files(
    ini_path: &str,
    key_path: &str,
    db_path: &str,
    zip_path: &Path,
) -> Result<(), String> {
    use std::io::Write;

    let file = std::fs::File::create(zip_path).map_err(|e| e.to_string())?;
    let mut zipw = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(5)); // Node.js JSZip level 5

    let files: Vec<&str> = vec![ini_path, key_path, db_path];
    for path in files {
        let p = Path::new(path);
        let name = p.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        match std::fs::read(p) {
            Ok(data) => {
                zipw.start_file(&name, opts)
                    .and_then(|_| zipw.write_all(&data).map_err(zip::result::ZipError::Io))
                    .map_err(|e| format!("压缩 {} 失败: {}", name, e))?;
            }
            Err(e) => warn!("autoBackup: 跳过缺失文件 {}: {}", path, e),
        }
    }
    zipw.finish().map_err(|e| e.to_string())?;
    Ok(())
}
