//! macOS 手柄：gilrs（IOKit HID / GameController）。
//! 输出与 Windows 完全一致的按键名（1Joy1…11、1JoyPOV1…8、JoyLT/RT、JoyX/Y/U/R）。
#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use gilrs::{Axis, Button};

use crate::stats::{push_txt, Shared};

/// 单个手柄读取一帧的按键集合（对应 joy/windows.rs 的 XInput 逻辑）
fn pressed_keys(gilrs: &gilrs::Gilrs, id: gilrs::GamepadId, cid: u32) -> Vec<String> {
    let gp = gilrs.gamepad(id);
    let mut keys = Vec::new();
    // 十字键：与 Windows 一致，只输出单一方向（POV1/3/5/7）
    let dpad = [
        (Button::DPadUp, 1),
        (Button::DPadRight, 3),
        (Button::DPadDown, 5),
        (Button::DPadLeft, 7),
    ];
    for (btn, pov) in dpad {
        if gp.is_pressed(btn) {
            keys.push(format!("{}JoyPOV{}", cid, pov));
        }
    }
    // 与 XInput 顺序对齐：A B X Y LB RB BACK START LSTICK RSTICK GUIDE
    let buttons = [
        (Button::South, 1),
        (Button::East, 2),
        (Button::West, 3),
        (Button::North, 4),
        (Button::LeftTrigger, 5),
        (Button::RightTrigger, 6),
        (Button::Select, 7),
        (Button::Start, 8),
        (Button::LeftThumb, 9),
        (Button::RightThumb, 10),
        (Button::Mode, 11),
    ];
    for (btn, n) in buttons {
        if gp.is_pressed(btn) {
            keys.push(format!("{}Joy{}", cid, n));
        }
    }
    // 模拟扳机（Windows 阈值为 10/255 ≈ 0.04）
    if gp.value(Axis::LeftZ) > 0.04 {
        keys.push(format!("{}JoyLT", cid));
    }
    if gp.value(Axis::RightZ) > 0.04 {
        keys.push(format!("{}JoyRT", cid));
    }
    // 摇杆（Windows 阈值为 2000/32767 ≈ 0.06）
    if gp.value(Axis::LeftStickX).abs() > 0.06 {
        keys.push(format!("{}JoyX", cid));
    }
    if gp.value(Axis::LeftStickY).abs() > 0.06 {
        keys.push(format!("{}JoyY", cid));
    }
    if gp.value(Axis::RightStickX).abs() > 0.06 {
        keys.push(format!("{}JoyU", cid));
    }
    if gp.value(Axis::RightStickY).abs() > 0.06 {
        keys.push(format!("{}JoyR", cid));
    }
    keys
}

pub fn start(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("poll".into())
        .spawn(move || {
            let mut gilrs = gilrs::Gilrs::new().ok();
            let mut joy_holds: HashMap<String, i32> = HashMap::new();
            let mut last_minute_tick = std::time::Instant::now();

            loop {
                if shared.exit.load(Ordering::Relaxed) {
                    break;
                }
                let (record_mouse_move, record_minute) = {
                    let cfg = shared.cfg.read().unwrap();
                    (cfg.record_mouse_move, cfg.record_minute)
                };
                let max_press = 1; // AHK maxJoyPressCount = 1

                // 鼠标移动距离（与 Windows 端一致，直接复用系统光标位置）
                if record_mouse_move {
                    if let Some((x, y)) = crate::winutil::cursor_pos() {
                        crate::stats::add_mouse_distance(&shared, x, y);
                    }
                }

                if let Some(g) = gilrs.as_mut() {
                    // 取完所有事件以刷新内部状态（连接/断开、轴值）
                    while g.next_event().is_some() {}
                    let connected: Vec<_> = g
                        .gamepads()
                        .filter(|(_, gp)| gp.is_connected())
                        .map(|(id, _)| id)
                        .take(4)
                        .collect();
                    let mut all_keys: Vec<String> = Vec::new();
                    for (i, id) in connected.iter().enumerate() {
                        let cid = i as u32 + 1;
                        let keys = pressed_keys(g, *id, cid);
                        for k in &keys {
                            let c = joy_holds.entry(k.clone()).or_insert(0);
                            *c += 1;
                            if *c <= max_press {
                                push_txt(&shared, k, false);
                            }
                        }
                        let prefix = format!("{}Joy", cid);
                        for (k, v) in joy_holds.iter_mut() {
                            if k.starts_with(&prefix) && !keys.contains(k) {
                                *v = 0; // 松开
                            }
                        }
                        all_keys.extend(keys);
                    }
                    let _ = all_keys;
                }

                // 分钟数据定时刷新（GetMinuteDataTimer，1 秒）
                // 锁顺序 stats→cfg，不 clone Config
                if record_minute && last_minute_tick.elapsed().as_millis() >= 1000 {
                    last_minute_tick = std::time::Instant::now();
                    let mut stats = shared.stats.lock().unwrap();
                    let cfg = shared.cfg.read().unwrap();
                    stats.minute_data(&cfg, false, false);
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        })
        .expect("spawn poll thread")
}
