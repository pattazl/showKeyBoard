//! 与后台服务通讯：HTTP(/version /data /sendPCInfo /exit) + WebSocket(ahkClient/IniMonitor)
//! 对应 AHK events.ahk。
#![allow(dead_code)]
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::stats::Shared;

const UA: &str = "ShowKeyBoardRust";

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn OutputDebugStringW(lp_output_string: *const u16);
}

/// 输出调试信息到调试器（DebugViewNt 等工具可查看），统一以 "AHK-" 开头
#[cfg(windows)]
fn debug_out(_msg: &str) {
    //let text = format!("AHK-{}\0", msg);
    //let wide: Vec<u16> = text.encode_utf16().collect();
    //unsafe { OutputDebugStringW(wide.as_ptr()) }
}

/// macOS 没有 DebugView，退化输出到 stderr（从 LaunchAgents 启动时会进系统日志）
#[cfg(target_os = "macos")]
fn debug_out(msg: &str) {
    eprintln!("AHK-{}", msg);
}

/// 界面状态提示前缀：`\u{1}` 常驻（连接中/异常），`\u{2}` 短暂显示后自动渐隐。
/// 由 UI 线程的浮窗（popup）显示，不使用系统对话框——避免抢焦点、影响输入。
pub const STATUS_STICKY: char = '\u{1}';
pub const STATUS_TEMP: char = '\u{2}';

/// 通过 UI 文本管道推送状态文本（复用浮窗 popup 展示）
fn push_status(shared: &Arc<Shared>, kind: char, text: impl AsRef<str>) {
    let _ = shared.ui_tx.send(format!("{}{}", kind, text.as_ref()));
}

fn post_json(url: &str, body: &str, timeout: u64) -> Result<String, String> {
    if !body.is_empty() {
        debug_out(&format!("POST {} {}", url, body));
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_millis(timeout))
        .user_agent(UA)
        .build();
    let resp = agent
        .post(url)
        .set("Content-Type", "application/json")
        .send_string(body)
        .map_err(|e| e.to_string())?;
    resp.into_string().map_err(|e| e.to_string())
}

/// SendPCInfo：屏幕信息上报，同时更新鼠标距离校验边界
fn send_pc_info(shared: &Arc<Shared>, flag: i32) {
    let monitors = shared.monitors.read().unwrap().clone();
    if monitors.is_empty() {
        return;
    }
    let mut arr = Vec::new();
    let (mut min_l, mut min_t, mut max_r, mut max_b) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for (l, t, r, b) in monitors {
        min_l = min_l.min(l);
        min_t = min_t.min(t);
        max_r = max_r.max(r);
        max_b = max_b.max(b);
        arr.push(json!({"Left": l, "Top": t, "Right": r, "Bottom": b}));
    }
    *shared.screen_bounds.lock().unwrap() = (min_l, min_t, max_r, max_b);
    let major = shared.major_version.clone();
    let body = json!({"screen": arr, "flag": flag, "majorVersion": major}).to_string();
    let url = { shared.cfg.read().unwrap().server_url() };
    let url = format!("{}/sendPCInfo", url);
    let _ = post_json(&url, &body, 8000);
}

/// 上报数据中的单个字母统一转小写（字母前后不能紧邻其他字母/数字，
/// 避免把 F5、LControl、App-Key-xxx 这类名称里的字母也改掉）
fn lower_single_letters(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    for (i, &c) in chars.iter().enumerate() {
        let prev_alnum = i > 0 && chars[i - 1].is_ascii_alphanumeric();
        let next_alnum = chars.get(i + 1).is_some_and(|n| n.is_ascii_alphanumeric());
        if c.is_ascii_uppercase() && !prev_alnum && !next_alnum {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// 序列化 AllKeyRecord（与 AHK JSON.stringify(AllKeyRecord) 字段一致）
pub fn build_data_body(shared: &Arc<Shared>) -> Value {
    let stats = shared.stats.lock().unwrap();
    let mut obj = serde_json::Map::new();
    // 按键计数（BTreeMap 按名称排序）；单字母按键统一小写上报
    for (k, v) in &stats.keys {
        obj.insert(lower_single_letters(k), json!(v));
    }
    obj.insert("tick".into(), json!(stats.tick));
    obj.insert("updateTime".into(), json!(stats.update_time));
    let cfg = shared.cfg.read().unwrap();
    if cfg.record_minute {
        obj.insert("MinuteRecords".into(), serde_json::to_value(&stats.minute_recs).unwrap_or(json!([])));
    }
    if cfg.record_history_max > 0 {
        let history: Vec<String> = stats.record_history.iter().map(|s| lower_single_letters(s)).collect();
        obj.insert("recordHistory".into(), json!(history));
    }
    Value::Object(obj)
}

// 进程信息查询（用于判断 kbserver.pid 记录的进程是否为我们自己的 node 服务）
#[cfg(windows)]
#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtQueryInformationProcess(
        process_handle: windows_sys::Win32::Foundation::HANDLE,
        process_information_class: u32,
        process_information: *mut core::ffi::c_void,
        process_information_length: u32,
        return_length: *mut u32,
    ) -> i32;
}

#[cfg(windows)]
#[repr(C)]
struct ProcessBasicInformation {
    _reserved1: *mut core::ffi::c_void,
    peb_base_address: *mut core::ffi::c_void,
    _reserved2: [*mut core::ffi::c_void; 2],
    _unique_process_id: *mut core::ffi::c_void,
    _reserved3: *mut core::ffi::c_void,
}

#[cfg(windows)]
#[repr(C)]
struct UnicodeString {
    length: u16,
    _maximum_length: u16,
    buffer: *mut u16,
}

/// 进程是否存在
#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return false;
    }
    unsafe { CloseHandle(handle) };
    true
}

/// 进程镜像名（不含路径）
#[cfg(windows)]
fn process_image_name(pid: u32) -> Option<String> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return None;
    }
    let mut buf = [0u16; 1024];
    let mut len = buf.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(handle as HANDLE, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len) };
    unsafe { CloseHandle(handle as HANDLE) };
    if ok == 0 {
        return None;
    }
    let s = String::from_utf16_lossy(&buf[..len as usize]);
    std::path::Path::new(&s)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}

/// 进程命令行（x64：PEB.ProcessParameters @ 0x20，CommandLine @ 0x88）
#[cfg(windows)]
fn process_command_line(pid: u32) -> Option<String> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

    let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid) };
    if handle.is_null() {
        return None;
    }
    let mut cmd: Option<String> = None;
    let mut pbi: ProcessBasicInformation = unsafe { core::mem::zeroed() };
    let rc = unsafe {
        NtQueryInformationProcess(
            handle as HANDLE,
            0, // ProcessBasicInformation
            &mut pbi as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<ProcessBasicInformation>() as u32,
            core::ptr::null_mut(),
        )
    };
    if rc == 0 && !pbi.peb_base_address.is_null() {
        let mut params_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        let mut read: usize = 0;
        let ok = unsafe {
            ReadProcessMemory(
                handle as HANDLE,
                (pbi.peb_base_address as usize + 0x20) as *const core::ffi::c_void,
                &mut params_ptr as *mut _ as *mut core::ffi::c_void,
                core::mem::size_of::<*mut core::ffi::c_void>(),
                &mut read,
            )
        };
        if ok != 0 && !params_ptr.is_null() {
            let mut us: UnicodeString = unsafe { core::mem::zeroed() };
            let ok = unsafe {
                ReadProcessMemory(
                    handle as HANDLE,
                    (params_ptr as usize + 0x88) as *const core::ffi::c_void,
                    &mut us as *mut _ as *mut core::ffi::c_void,
                    core::mem::size_of::<UnicodeString>(),
                    &mut read,
                )
            };
            if ok != 0 && !us.buffer.is_null() && us.length > 0 {
                let mut buf = vec![0u16; (us.length / 2) as usize];
                let ok = unsafe {
                    ReadProcessMemory(
                        handle as HANDLE,
                        us.buffer as *const core::ffi::c_void,
                        buf.as_mut_ptr() as *mut core::ffi::c_void,
                        us.length as usize,
                        &mut read,
                    )
                };
                if ok != 0 {
                    cmd = Some(String::from_utf16_lossy(&buf));
                }
            }
        }
    }
    unsafe { CloseHandle(handle as HANDLE) };
    cmd
}

#[cfg(windows)]
fn kill_process(pid: u32) {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};
    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
    if !handle.is_null() {
        unsafe { TerminateProcess(handle, 0) };
        unsafe { CloseHandle(handle) };
    }
}

// ---------- macOS 进程工具（libproc / libc，无外部命令依赖） ----------

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn proc_pidpath(pid: i32, buffer: *mut core::ffi::c_void, buffersize: u32) -> i32;
}

#[cfg(target_os = "macos")]
fn process_exists(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(target_os = "macos")]
fn process_image_name(pid: u32) -> Option<String> {
    let mut buf = [0u8; 4096];
    let n = unsafe { proc_pidpath(pid as i32, buf.as_mut_ptr() as *mut _, buf.len() as u32) };
    if n <= 0 {
        return None;
    }
    let path = std::str::from_utf8(&buf[..n as usize]).ok()?;
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}

#[cfg(target_os = "macos")]
fn process_command_line(pid: u32) -> Option<String> {
    // macOS 没有等价的可读 PEB，用 ps 读取完整命令行（仅启动服务时调用一次，开销可接受）
    let out = std::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(target_os = "macos")]
fn kill_process(pid: u32) {
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
}

/// 启动 Node 后台服务（对应 AHK startServer）：
/// 启动前读取 kbserver.pid，若记录的进程仍在运行且确认为本服务，则先杀掉并清理 pid 文件，
/// 避免重复启动。命令行读取走 NtQueryInformationProcess+ReadProcessMemory，无需外部进程。
/// 失败时返回可直接展示给用户的原因（由调用方推到界面浮窗）。
fn start_node_server(shared: &Arc<Shared>) -> Result<(), String> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;
    let (http_path, exec, _debug) = {
        let cfg = shared.cfg.read().unwrap();
        (cfg.http_path.clone(), cfg.server_exec_name.clone(), cfg.show_http_debug)
    };
    if !http_path.exists() {
        return Err(format!("未找到后台服务目录 {}", http_path.display()));
    }
    let script = http_path.join("server.js");
    let exec_raw = exec.trim();
    let default_exec = if cfg!(windows) { "node.exe" } else { "node" };
    let exec_raw = if exec_raw.is_empty() { default_exec } else { exec_raw };

    // 程序名解析：CreateProcess 解析 exe 时**不使用** Command 的 current_dir
    // （那只设置子进程的工作目录），否则 showkeyboard_server.exe 这类放在 httpPath 下的
    // 独立服务只会去客户端目录/PATH 里找，找不到就静默失败——node.exe 在 PATH 里所以正常。
    let mut exec_path = std::path::PathBuf::from(exec_raw);
    if !exec_path.is_absolute() && !exec_raw.contains(['\\', '/']) {
        let local = http_path.join(exec_raw);
        if local.exists() {
            exec_path = local;
        }
    }
    debug_out(&format!(
        "start server: http_path={} exec_raw={} resolved={} exists={}",
        http_path.display(),
        exec_raw,
        exec_path.display(),
        exec_path.exists()
    ));
    let exec_name = exec_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| exec_raw.to_string());
    let is_node = exec_name.to_lowercase().starts_with("node");
    if is_node && !script.exists() {
        return Err(format!("未找到后台服务脚本 {}", script.display()));
    }

    let pid_file = http_path.join("kbserver.pid");
    if pid_file.exists() {
        if let Ok(s) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = s.trim().parse::<u32>() {
                if pid > 0 && process_exists(pid) {
                    let image = process_image_name(pid).unwrap_or_default();
                    let cmd = process_command_line(pid).unwrap_or_default();
                    // 按文件名（去扩展名）比较，node / node.exe / showkeyboard_server.exe 都适用
                    let same_exe = std::path::Path::new(&image)
                        .file_stem()
                        .zip(std::path::Path::new(&exec_name).file_stem())
                        .map(|(a, b)| a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy()))
                        .unwrap_or(false);
                    let script_lc = script.to_string_lossy().to_lowercase();
                    let http_lc = http_path.to_string_lossy().to_lowercase();
                    let cmd_lc = cmd.to_lowercase();
                    let is_ours = cmd_lc.contains(&script_lc)
                        || cmd_lc.contains(&http_lc)
                        || cmd.is_empty();
                    if same_exe && is_ours {
                        kill_process(pid);
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&pid_file);
    }

    let mut cmd = std::process::Command::new(&exec_path);
    // 存在 server.js 时按 AHK 的方式作为参数传入；独立打包服务没有脚本文件也能启动
    if script.exists() {
        cmd.arg(&script);
    }
    cmd.current_dir(&http_path);
    #[cfg(windows)]
    cmd.creation_flags(if _debug { 0 } else { 0x08000000 });
    match cmd.spawn() {
        Ok(_) => {
            // 假设启动失败，等 /version 检测成功后才置 1（对应 AHK startServer 中 serverState := 0）
            shared.server_state.store(0, Ordering::Relaxed);
            std::thread::sleep(Duration::from_millis(1000)); // 启动服务需要等待
            Ok(())
        }
        Err(e) => {
            debug_out(&format!("start node server failed: {}", e));
            Err(format!("启动 {} 失败：{}", exec_name, e))
        }
    }
}

/// /version 连接检查（对应 CheckServer / ServerCore）。
/// 失败时返回可直接展示给用户的原因，便于在界面浮窗反馈异常。
fn check_server(shared: &Arc<Shared>) -> Result<(), String> {
    let url = { format!("{}/version", shared.cfg.read().unwrap().server_url()) };
    match post_json(&url, "", 3000) {
        Ok(text) if text.contains("showKeyBoardServer Version:") => {
            shared.server_state.store(1, Ordering::Relaxed);
            Ok(())
        }
        Ok(body) => {
            shared.server_state.store(0, Ordering::Relaxed);
            let brief: String = body.trim().chars().take(60).collect();
            Err(format!(
                "服务返回异常响应：{}",
                if brief.is_empty() { "<空>" } else { &brief }
            ))
        }
        Err(e) => {
            shared.server_state.store(0, Ordering::Relaxed);
            Err(format!("无法连接 {}：{}", url, e))
        }
    }
}

/// 拉起服务后立即重试连接（每 500ms 一次），期间在浮窗中显示进度；
/// 失败返回最后一次错误原因。
fn wait_server(shared: &Arc<Shared>, attempts: u32) -> Result<(), String> {
    let mut last = String::new();
    for i in 0..attempts {
        if shared.exit.load(Ordering::Relaxed) {
            return Err(last);
        }
        match check_server(shared) {
            Ok(()) => return Ok(()),
            Err(e) => last = e,
        }
        if i + 1 < attempts {
            push_status(
                shared,
                STATUS_STICKY,
                format!("正在连接后台服务…（第 {} 次）", i + 2),
            );
            std::thread::sleep(Duration::from_millis(500));
        }
    }
    Err(last)
}

fn is_server_ok(shared: &Arc<Shared>) -> bool {
    shared.server_state.load(Ordering::Relaxed) == 1
}

/// 网络线程
pub fn start(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("net".into())
        .spawn(move || {
            // 初始屏幕边界
            {
                let mons = crate::winutil::monitors();
                let (mut min_l, mut min_t, mut max_r, mut max_b) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
                for (l, t, r, b) in &mons {
                    min_l = min_l.min(*l);
                    min_t = min_t.min(*t);
                    max_r = max_r.max(*r);
                    max_b = max_b.max(*b);
                }
                *shared.screen_bounds.lock().unwrap() = (min_l, min_t, max_r, max_b);
                *shared.monitors.write().unwrap() = mons;
            }

            let mut check_count = 0;
            let mut last_send = Instant::now() - Duration::from_secs(10);
            let mut fail_count = 0;
            let mut started_server = false;

            loop {
                if shared.exit.load(Ordering::Relaxed) {
                    break;
                }

                // IniMonitor 热重载
                if shared.ws_reload.swap(false, Ordering::Relaxed) {
                    reload_config(&shared);
                }

                // PCInfo 请求
                let pc_flag = shared.pcinfo_dirty.swap(0, Ordering::Relaxed);
                if pc_flag != 0 && is_server_ok(&shared) {
                    send_pc_info(&shared, pc_flag);
                }

                if !is_server_ok(&shared) {
                    // server_state 初始为 -1（从未启动过），首次 /version 失败时立即拉起服务，
                    // 之后再按 check_server_max 重试（对应 AHK serverState=-1 时 startServer）
                    let never_started = shared.server_state.load(Ordering::Relaxed) == -1;
                    // 重启/完全重启过程中不要自动拉起后端（完全重启是先关服务再重启自己）
                    let restarting = shared.restart_mode.load(Ordering::Relaxed) != 0;
                    if check_server(&shared).is_ok() {
                        push_status(&shared, STATUS_TEMP, "后台服务已连接");
                        send_pc_info(&shared, 0);
                        check_count = 0;
                        started_server = true;
                    } else {
                        check_count += 1;
                        if never_started && !started_server && !restarting {
                            started_server = true;
                            push_status(&shared, STATUS_STICKY, "正在启动后台服务…");
                            match start_node_server(&shared) {
                                Ok(()) => {
                                    // 启动后立即重试连接，界面持续反馈进度/异常
                                    push_status(&shared, STATUS_STICKY, "后台服务已启动，正在连接…");
                                    match wait_server(&shared, 20) {
                                        Ok(()) => {
                                            push_status(&shared, STATUS_TEMP, "后台服务已连接");
                                            send_pc_info(&shared, 0);
                                            check_count = 0;
                                        }
                                        Err(e) => push_status(
                                            &shared,
                                            STATUS_STICKY,
                                            format!("后台服务连接失败：{}", e),
                                        ),
                                    }
                                }
                                Err(e) => push_status(
                                    &shared,
                                    STATUS_STICKY,
                                    format!("后台服务启动失败：{}", e),
                                ),
                            }
                        }
                        // 超过最大尝试次数后退避重试，不阻塞主循环
                        let max = shared.cfg.read().unwrap().check_server_max;
                        if check_count > max {
                            check_count = max;
                        }
                    }
                    std::thread::sleep(Duration::from_secs(3));
                    continue;
                }

                // 已连接：按 1 秒限频发送 /data
                let dirty = shared.stats.lock().unwrap().dirty;
                if dirty && last_send.elapsed().as_millis() >= 1000 {
                    let body = build_data_body(&shared).to_string();
                    let url = format!("{}/data", shared.cfg.read().unwrap().server_url());
                    shared.stats.lock().unwrap().dirty = false;
                    last_send = Instant::now();
                    if post_json(&url, &body, 8000).is_ok() {
                        fail_count = 0;
                    } else {
                        fail_count += 1;
                        // maxCountOfConnectFail：连续失败超过该次数才判定掉线
                        let max_fail = shared.cfg.read().unwrap().max_count_of_connect_fail;
                        if fail_count >= max_fail {
                            shared.server_state.store(-1, Ordering::Relaxed);
                            push_status(&shared, STATUS_STICKY, "后台服务连接中断，正在重连…");
                            check_count = 0;
                            started_server = false;
                        }
                    }
                }

                std::thread::sleep(Duration::from_millis(200));
            }
        })
        .expect("spawn net thread")
}

/// 配置热重载（IniMonitor(1)）：重新读取 ini，并同步钩子参数
fn reload_config(shared: &Arc<Shared>) {
    let mut cfg = shared.cfg.write().unwrap();
    let new_cfg = crate::config::Config::load();
    // needShowKey 的运行时开关不丢失
    let need_show = cfg.need_show_key;
    let mut new_cfg = new_cfg;
    new_cfg.need_show_key = need_show;
    new_cfg.generation = cfg.generation + 1;
    *cfg = new_cfg;
    drop(cfg);
    crate::input::refresh_hook_params();
}

/// WebSocket 线程：ahkClient 身份，接收 IniMonitor，并发送实时按键文本
/// （对应 AHK events.ahk BindWebSocket + dialog.ahk 中的 handleWS.sendText(wsText)）
pub fn start_ws(
    shared: Arc<Shared>,
    ws_rx: crossbeam_channel::Receiver<String>,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("ws".into())
        .spawn(move || loop {
            if shared.exit.load(Ordering::Relaxed) {
                break;
            }
            if shared.server_state.load(Ordering::Relaxed) != 1 {
                shared.ws_ready.store(false, Ordering::Relaxed);
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            let url = { shared.cfg.read().unwrap().server_ws_url() };
            match tungstenite::connect(&url) {
                Ok((mut ws, _)) => {
                    use tungstenite::stream::MaybeTlsStream;
                    use tungstenite::Message;
                    let _ = ws.send(Message::Text("ahkClient".into()));
                    debug_out("WS send ahkClient");
                    if let MaybeTlsStream::Plain(tcp) = ws.get_mut() {
                        // 短读超时：既及时响应 IniMonitor，也让发送队列尽快投递
                        tcp.set_read_timeout(Some(Duration::from_millis(50))).ok();
                    }
                    // 丢弃未连接期间积压的按键文本（AHK 未连接时 sendText 直接失败，不补发）
                    while ws_rx.try_recv().is_ok() {}
                    shared.ws_ready.store(true, Ordering::Relaxed);
                    let mut link_ok = true;
                    while link_ok {
                        if shared.exit.load(Ordering::Relaxed) {
                            break;
                        }
                        // 实时按键文本：按到达顺序发送（对应 handleWS.sendText(wsText)）
                        while let Ok(text) = ws_rx.try_recv() {
                            if text.is_empty() {
                                continue;
                            }
                            debug_out(&format!("WS send {}", text));
                            if ws.send(Message::Text(text)).is_err() {
                                link_ok = false;
                                break;
                            }
                        }
                        if !link_ok {
                            break;
                        }
                        match ws.read() {
                            Ok(Message::Text(text)) => {
                                if text.trim() == "IniMonitor" {
                                    shared.ws_reload.store(true, Ordering::Relaxed);
                                }
                            }
                            Ok(Message::Ping(_)) => {}
                            Ok(_) => {}
                            Err(tungstenite::Error::Io(ref e))
                                if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut =>
                            {
                                continue;
                            }
                            Err(_) => break,
                        }
                    }
                    shared.ws_ready.store(false, Ordering::Relaxed);
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_secs(3));
                }
            }
            std::thread::sleep(Duration::from_millis(300));
        })
        .expect("spawn ws thread")
}

/// 退出时发送 /exit（ExitServer）
pub fn send_exit(shared: &Arc<Shared>) {
    if is_server_ok(shared) {
        let url = format!("{}/exit", shared.cfg.read().unwrap().server_url());
        let _ = post_json(&url, "", 3000);
    }
}

/// 后台服务是否仍在响应 /version（不修改 server_state）
fn server_reachable(shared: &Arc<Shared>) -> bool {
    let url = { format!("{}/version", shared.cfg.read().unwrap().server_url()) };
    matches!(post_json(&url, "", 800), Ok(text) if text.contains("showKeyBoardServer Version:"))
}

/// 完全重启前关闭后台服务：
/// 1) 先 POST /exit 让服务端落盘并自行退出；
/// 2) 若 /exit 未生效（旧版服务/端口残留），按 kbserver.pid 强制结束；
/// 3) 等待端口真正下线，保证新实例启动时检测不到服务，从而重新拉起后端。
pub fn shutdown_server(shared: &Arc<Shared>) {
    if is_server_ok(shared) || server_reachable(shared) {
        let url = { format!("{}/exit", shared.cfg.read().unwrap().server_url()) };
        let _ = post_json(&url, "", 1500);
    }

    let (http_path, exec) = {
        let cfg = shared.cfg.read().unwrap();
        (cfg.http_path.clone(), cfg.server_exec_name.clone())
    };
    let pid_file = http_path.join("kbserver.pid");
    let exec_raw = if exec.trim().is_empty() { "node" } else { exec.trim() };
    let exec_stem = std::path::Path::new(exec_raw)
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if let Ok(s) = std::fs::read_to_string(&pid_file) {
        if let Ok(pid) = s.trim().parse::<u32>() {
            if pid > 0 && process_exists(pid) {
                let image = process_image_name(pid).unwrap_or_default();
                let stem = std::path::Path::new(&image)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                // 防止 pid 被复用后误杀无关进程
                if stem == exec_stem || stem.starts_with("node") || stem.contains("showkeyboard_server") {
                    kill_process(pid);
                }
            }
        }
    }

    // 最多等待约 2 秒，确保端口释放
    for _ in 0..8 {
        if !server_reachable(shared) {
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}
