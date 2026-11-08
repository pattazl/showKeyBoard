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
    let (db_path, base_dir) = {
        let sg = state.read().await;
        (sg.db_path.clone(), sg.base_dir.clone())
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
    let _ = &base_dir;

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

/// Node: path.resolve(basePath, xxx) —— 绝对路径直接用，否则基于 base_dir
fn resolve_path(base: &str, p: &str) -> PathBuf {
    let p = portable_path(p);
    let path = Path::new(&p);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(base).join(&p)
    }
}

async fn auto_share_core(state: &AppState, cfg: &AppConfig, hour: f64) -> Result<(), String> {
    let (base_dir, db_path) = {
        let sg = state.read().await;
        (sg.base_dir.clone(), sg.db_path.clone())
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

    // 递归创建共享目录
    let share_file_dir = resolve_path(&base_dir, &share_db_path);
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

    // 解压共享文件到 dbs/ 目录
    unzip_shared(&base_dir, &share_file_dir, &share_db_name).await
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

/// Node: unzipShared —— 扫描共享目录全部 *.zip（跳过自己的），
/// 与 dbs/ 目录下 <用户名>_<时间戳>.db 对比，较新则删除旧库并解压覆盖
async fn unzip_shared(
    base_dir: &str,
    share_file_dir: &Path,
    own_name: &str,
) -> Result<(), String> {
    let dbs_path = Path::new(base_dir).join("dbs/");
    std::fs::create_dir_all(&dbs_path).map_err(|e| format!("目录创建失败: {}", e))?;

    // 共享目录中的 zip：key = 去掉 .zip 的文件名
    let zip_info = collect_files(share_file_dir, "zip", |name| {
        name.rsplit_once('.').map(|x| x.0.to_string()).unwrap_or_else(|| name.to_string())
    });
    // dbs 目录中的 db：key = 文件名按 '_' 去掉最后一段
    let db_info = collect_files(&dbs_path, "db", |name| {
        name.split('_')
            .next_back()
            .map(|_| {
                let parts: Vec<&str> = name.split('_').collect();
                parts[..parts.len() - 1].join("_")
            })
            .unwrap_or_else(|| name.to_string())
    });

    for (file_id, zinfo) in &zip_info {
        // 需要不是自己的文件
        if file_id == own_name {
            continue;
        }
        // zip 修改时间减去 1 分钟（Node: newTime = ctime - 60*1000）
        let zip_mtime = zinfo.mtime_secs.saturating_sub(60);
        let need_unzip = match db_info.get(file_id) {
            None => true, // 新文件需要解压
            Some(dinfo) => {
                // dbs 中的旧文件名末段是解压/备份时间戳（秒）
                let dbs_ts = dinfo
                    .name
                    .split('_')
                    .next_back()
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);
                if zip_mtime > dbs_ts {
                    // 文件更新需要删除原先的 db 文件
                    info!("delete shared db: {:?}", dinfo.path);
                    let _ = std::fs::remove_file(&dinfo.path);
                    true
                } else {
                    false
                }
            }
        };
        if need_unzip {
            info!("unzip: {:?}", zinfo.path);
            let zip_file = zinfo.path.clone();
            let dbs_dir = dbs_path.clone();
            tokio::task::spawn_blocking(move || unzip_to_dbs(&zip_file, &dbs_dir))
                .await
                .map_err(|e| e.to_string())??;
        }
    }
    Ok(())
}

struct FileInfo {
    name: String,
    path: PathBuf,
    mtime_secs: i64,
}

/// Node getFilesInfo：收集目录下指定扩展名的文件信息
fn collect_files<F: Fn(&str) -> String>(
    directory: &Path,
    ext: &str,
    key_fun: F,
) -> std::collections::HashMap<String, FileInfo> {
    let mut files_info = std::collections::HashMap::new();
    let entries = match std::fs::read_dir(directory) {
        Ok(e) => e,
        Err(_) => return files_info,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.to_lowercase().ends_with(&format!(".{}", ext)) {
            continue;
        }
        let mtime_secs = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        files_info.insert(
            key_fun(&name),
            FileInfo { name, path, mtime_secs },
        );
    }
    files_info
}

/// 解压 zip 内全部文件到 dbs 目录（对应 Node unZipCore + fun(filename)）
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
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(|e| e.to_string())?;
        let dest = dbs_dir.join(name);
        std::fs::write(dest, &data).map_err(|e| e.to_string())?;
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
