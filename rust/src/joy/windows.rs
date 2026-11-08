//! 游戏手柄轮询（对应 showKeyBoard.ahk 的两套方案）
//! - joyMethod == 1: winmm!joyGetPosEx（AHK 方案1，按键通过 dwButtons 边沿检测）
//! - joyMethod == 2: XInput!XInputGetState（AHK 方案2，getJoyInfo2 逻辑）
#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::Arc;

use crate::stats::{push_txt, Shared};

// XInput 按键常量
const DPAD_UP: u16 = 0x0001;
const DPAD_DOWN: u16 = 0x0002;
const DPAD_LEFT: u16 = 0x0004;
const DPAD_RIGHT: u16 = 0x0008;
const START: u16 = 0x0010;
const BACK: u16 = 0x0020;
const LEFT_THUMB: u16 = 0x0040;
const RIGHT_THUMB: u16 = 0x0080;
const LEFT_SHOULDER: u16 = 0x0100;
const RIGHT_SHOULDER: u16 = 0x0200;
const GUIDE: u16 = 0x0400;
const A: u16 = 0x1000;
const B: u16 = 0x2000;
const X: u16 = 0x4000;
const Y: u16 = 0x8000;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct XInputState {
    packet_number: u32,
    w_buttons: u16,
    left_trigger: u8,
    right_trigger: u8,
    thumb_lx: i16,
    thumb_ly: i16,
    thumb_rx: i16,
    thumb_ry: i16,
}

struct XInputDll {
    get_state: unsafe extern "system" fn(u32, *mut XInputState) -> i32,
}

unsafe fn load_xinput() -> Option<XInputDll> {
    type GetState = unsafe extern "system" fn(u32, *mut XInputState) -> i32;
    for name in ["xinput1_4.dll", "xinput1_3.dll", "xinput9_1_0.dll"] {
        if let Ok(lib) = unsafe { libloading::Library::new(name) } {
            // 先尝试序号 100（xinput1_4 加密导出序号），失败用导出名
            let sym: Result<libloading::Symbol<GetState>, _> =
                unsafe { lib.get(b"\x64\x00\x00\x00") };
            let f: GetState = match sym {
                Ok(s) => *s,
                Err(_) => match unsafe { lib.get(b"XInputGetState") } {
                    Ok(s) => *s,
                    Err(_) => continue,
                },
            };
            std::mem::forget(lib); // 进程生命周期内保持加载
            return Some(XInputDll { get_state: f });
        }
    }
    None
}

/// XInput 方案（对应 getJoyInfo2）
fn check_joy_state_xinput(dll: &XInputDll, shared: &Arc<Shared>, joy_holds: &mut HashMap<String, i32>, max_press: i32) {
    const POV_ARR: [u16; 8] = [
        DPAD_UP, DPAD_UP | DPAD_RIGHT, DPAD_RIGHT, DPAD_RIGHT | DPAD_DOWN,
        DPAD_DOWN, DPAD_DOWN | DPAD_LEFT, DPAD_LEFT, DPAD_LEFT | DPAD_UP,
    ];
    const BUTTON_ARR: [u16; 11] = [
        A, B, X, Y, LEFT_SHOULDER, RIGHT_SHOULDER, BACK, START, LEFT_THUMB, RIGHT_THUMB, GUIDE,
    ];
    let trigger_holds = 10i32;
    let thumb_holds = 2000i32;
    for cid in 1u32..=4 {
        let mut state = XInputState::default();
        if unsafe { (dll.get_state)(cid - 1, &mut state) } != 0 {
            continue;
        }
        let mut keys: Vec<String> = Vec::new();
        let low4 = state.w_buttons & 0x000F;
        for (i, po) in POV_ARR.iter().enumerate() {
            if low4 == *po {
                keys.push(format!("{}JoyPOV{}", cid, i + 1));
                break;
            }
        }
        for (i, mask) in BUTTON_ARR.iter().enumerate() {
            if state.w_buttons & mask != 0 {
                keys.push(format!("{}Joy{}", cid, i + 1));
            }
        }
        if state.left_trigger as i32 > trigger_holds {
            keys.push(format!("{}JoyLT", cid));
        }
        if state.right_trigger as i32 > trigger_holds {
            keys.push(format!("{}JoyRT", cid));
        }
        if (state.thumb_lx as i32).abs() > thumb_holds {
            keys.push(format!("{}JoyX", cid));
        }
        if (state.thumb_ly as i32).abs() > thumb_holds {
            keys.push(format!("{}JoyY", cid));
        }
        if (state.thumb_rx as i32).abs() > thumb_holds {
            keys.push(format!("{}JoyU", cid));
        }
        if (state.thumb_ry as i32).abs() > thumb_holds {
            keys.push(format!("{}JoyR", cid));
        }
        // 按下的计数 +1（不超过 max），松开的复位
        for k in &keys {
            let c = joy_holds.entry(k.clone()).or_insert(0);
            *c += 1;
            if *c <= max_press {
                push_txt(shared, k, false);
            }
        }
        let prefix = format!("{}Joy", cid);
        let held: Vec<String> = joy_holds
            .keys()
            .filter(|k| k.starts_with(&prefix) && keys.contains(k))
            .cloned()
            .collect();
        for (k, v) in joy_holds.iter_mut() {
            if k.starts_with(&prefix) && !held.contains(k) {
                *v = 0; // 松开
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct JoyInfoEx {
    size: u32,
    flags: u32,
    xpos: u32,
    ypos: u32,
    zpos: u32,
    rpos: u32,
    upos: u32,
    vpos: u32,
    buttons: u32,
    button_number: u32,
    pov: u32,
    reserved1: u32,
    reserved2: u32,
}

struct Winmm {
    joy_get_pos_ex: unsafe extern "system" fn(u32, *mut JoyInfoEx) -> i32,
}

unsafe fn load_winmm() -> Option<Winmm> {
    unsafe {
        if let Ok(lib) = libloading::Library::new("winmm.dll") {
            if let Ok(f) = lib.get(b"joyGetPosEx") {
                let f = *f;
                std::mem::forget(lib);
                return Some(Winmm { joy_get_pos_ex: f });
            }
        }
    }
    None
}

/// joyGetPosEx 方案（对应 AHK 方案1 的 getJoyInfo 逻辑）
fn check_joy_state_winmm(dll: &Winmm, shared: &Arc<Shared>, joy_holds: &mut HashMap<String, i32>, max_press: i32) {
    const JOY_RETURNALL: u32 = 0xFF;
    for cid in 1u32..=4 {
        let mut ji = JoyInfoEx::default();
        ji.size = std::mem::size_of::<JoyInfoEx>() as u32;
        ji.flags = JOY_RETURNALL;
        if unsafe { (dll.joy_get_pos_ex)(cid - 1, &mut ji) } != 0 {
            continue;
        }
        let mut keys: Vec<String> = Vec::new();
        // Z: <50 RT, >50 LT（AHK: val<50 -> RT, val>50 -> LT）
        let z = (ji.zpos as f32 / 65535.0 * 100.0).round() as i32;
        if z < 50 {
            keys.push(format!("{}JoyRT", cid));
        } else if z > 50 {
            keys.push(format!("{}JoyLT", cid));
        }
        // POV: 8 方向
        if ji.pov != 0xFFFF {
            let idx = (ji.pov / 4500) as usize;
            if idx < 8 {
                keys.push(format!("{}JoyPOV{}", cid, idx + 1));
            }
        }
        // 按钮（dwButtons 位图）
        for bit in 0..32u32 {
            if ji.buttons & (1 << bit) != 0 {
                keys.push(format!("{}Joy{}", cid, bit + 1));
            }
        }
        // 摇杆轴（!=50 为触发）
        for (axis, name) in [
            (ji.xpos, "JoyX"), (ji.ypos, "JoyY"), (ji.rpos, "JoyR"), (ji.upos, "JoyU"), (ji.vpos, "JoyV"),
        ] {
            let val = (axis as f32 / 65535.0 * 100.0).round() as i32;
            if val != 50 {
                keys.push(format!("{}{}", cid, name));
            }
        }
        for k in &keys {
            let c = joy_holds.entry(k.clone()).or_insert(0);
            *c += 1;
            if *c <= max_press {
                push_txt(shared, k, false);
            }
        }
        let prefix = format!("{}Joy", cid);
        let held: Vec<String> = joy_holds
            .keys()
            .filter(|k| k.starts_with(&prefix) && keys.contains(k))
            .cloned()
            .collect();
        for (k, v) in joy_holds.iter_mut() {
            if k.starts_with(&prefix) && !held.contains(k) {
                *v = 0;
            }
        }
    }
}

/// 轮询线程：鼠标距离(100ms) + 手柄(100ms) + 分钟数据刷新(1s)
pub fn start(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("poll".into())
        .spawn(move || {
            let xinput = unsafe { load_xinput() };
            let winmm = unsafe { load_winmm() };
            let mut joy_holds: HashMap<String, i32> = HashMap::new();
            let mut last_minute_tick = std::time::Instant::now();

            loop {
                if shared.exit.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let (joy_method, record_mouse_move, record_minute) = {
                    let cfg = shared.cfg.read().unwrap();
                    (cfg.joy_method, cfg.record_mouse_move, cfg.record_minute)
                };
                let max_press = 1; // AHK maxJoyPressCount = 1

                // 鼠标移动距离
                if record_mouse_move {
                    if let Some((x, y)) = crate::winutil::cursor_pos() {
                        crate::stats::add_mouse_distance(&shared, x, y);
                    }
                }

                // 手柄
                match joy_method {
                    1 => {
                        if let Some(dll) = &winmm {
                            check_joy_state_winmm(dll, &shared, &mut joy_holds, max_press);
                        }
                    }
                    _ => {
                        if let Some(dll) = &xinput {
                            check_joy_state_xinput(dll, &shared, &mut joy_holds, max_press);
                        }
                    }
                }

                // 分钟数据定时刷新（GetMinuteDataTimer，1 秒）
                if record_minute && last_minute_tick.elapsed().as_millis() >= 1000 {
                    last_minute_tick = std::time::Instant::now();
                    let cfg = shared.cfg.read().unwrap().clone();
                    let mut stats = shared.stats.lock().unwrap();
                    stats.minute_data(&cfg, false, false);
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        })
        .expect("spawn poll thread")
}
