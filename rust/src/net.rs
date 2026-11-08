//! 与后台服务通讯：HTTP(/version /data /sendPCInfo /exit) + WebSocket(ahkClient/IniMonitor)
//! 对应 AHK events.ahk。
#![allow(dead_code)]
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::stats::Shared;

const UA: &str = "ShowKeyBoardRust";

#[link(name = "kernel32")]
unsafe extern "system" {
    fn OutputDebugStringW(lp_output_string: *const u16);
}

/// 输出调试信息到调试器（DebugViewNt 等工具可查看），统一以 "AHK-" 开头
fn debug_out(msg: &str) {
    let text = format!("AHK-{}\0", msg);
    let wide: Vec<u16> = text.encode_utf16().collect();
    unsafe { OutputDebugStringW(wide.as_ptr()) }
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

#[repr(C)]
struct ProcessBasicInformation {
    _reserved1: *mut core::ffi::c_void,
    peb_base_address: *mut core::ffi::c_void,
    _reserved2: [*mut core::ffi::c_void; 2],
    _unique_process_id: *mut core::ffi::c_void,
    _reserved3: *mut core::ffi::c_void,
}

#[repr(C)]
struct UnicodeString {
    length: u16,
    _maximum_length: u16,
    buffer: *mut u16,
}

/// 进程是否存在
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

fn kill_process(pid: u32) {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};
    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
    if !handle.is_null() {
        unsafe { TerminateProcess(handle, 0) };
        unsafe { CloseHandle(handle) };
    }
}

/// 启动 Node 后台服务（对应 AHK startServer）：
/// 启动前读取 kbserver.pid，若记录的进程仍在运行且确认为本服务，则先杀掉并清理 pid 文件，
/// 避免重复启动。命令行读取走 NtQueryInformationProcess+ReadProcessMemory，无需外部进程。
fn start_node_server(shared: &Arc<Shared>) {
    use std::os::windows::process::CommandExt;
    let (http_path, exec, debug) = {
        let cfg = shared.cfg.read().unwrap();
        (cfg.http_path.clone(), cfg.server_exec_name.clone(), cfg.show_http_debug)
    };
    let script = http_path.join("server.js");
    if !script.exists() {
        return;
    }
    let exec_name = exec.trim_end_matches(".exe").to_string();
    let exec = if exec_name.is_empty() { "node".to_string() } else { exec_name };

    let pid_file = http_path.join("kbserver.pid");
    if pid_file.exists() {
        if let Ok(s) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = s.trim().parse::<u32>() {
                if pid > 0 && process_exists(pid) {
                    let image = process_image_name(pid).unwrap_or_default();
                    let cmd = process_command_line(pid).unwrap_or_default();
                    let is_node = image.eq_ignore_ascii_case(&format!("{}.exe", exec))
                        || image.eq_ignore_ascii_case(&exec);
                    let script_lc = script.to_string_lossy().to_lowercase();
                    let http_lc = http_path.to_string_lossy().to_lowercase();
                    let cmd_lc = cmd.to_lowercase();
                    let is_ours =
                        cmd_lc.contains(&script_lc) || cmd_lc.contains(&http_lc);
                    if is_node && is_ours {
                        kill_process(pid);
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&pid_file);
    }

    let mut cmd = std::process::Command::new(&exec);
    cmd.arg(&script)
        .current_dir(&http_path)
        .creation_flags(if debug { 0 } else { 0x08000000 });
    match cmd.spawn() {
        Ok(_) => {
            // 假设启动失败，等 /version 检测成功后才置 1（对应 AHK startServer 中 serverState := 0）
            shared.server_state.store(0, Ordering::Relaxed);
            std::thread::sleep(Duration::from_millis(1000)); // 启动服务需要等待
        }
        Err(e) => debug_out(&format!("start node server failed: {}", e)),
    }
}

/// /version 连接检查（对应 CheckServer / ServerCore）
fn check_server(shared: &Arc<Shared>) -> bool {
    let url = { format!("{}/version", shared.cfg.read().unwrap().server_url()) };
    match post_json(&url, "", 3000) {
        Ok(text) if text.contains("showKeyBoardServer Version:") => {
            shared.server_state.store(1, Ordering::Relaxed);
            true
        }
        _ => {
            shared.server_state.store(0, Ordering::Relaxed);
            false
        }
    }
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
                    if check_server(&shared) {
                        send_pc_info(&shared, 0);
                        check_count = 0;
                        started_server = true;
                    } else {
                        check_count += 1;
                        if never_started && !started_server {
                            started_server = true;
                            start_node_server(&shared);
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
