// build.rs
// 1. Slint UI 编译
// 2. release 编译自动版本号： <Cargo.toml版本>.<YYMMDD>.<当日序号>  例如 1.61.0.261105.3
//    - 同一天多次 cargo build --release 序号递增（状态文件 build_no.txt）
//    - 跨天日期自动更新，序号归零
//    - debug 编译只读取，不消耗序号
//    - 可用环境变量 BUILD_COUNT 强制指定序号

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    slint_build::compile("ui/app.slint").expect("Slint compile failed");
    gen_version();
}

/// days since 1970-01-01 -> (y, m, d)，civil_from_days 算法
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn local_ymd() -> (i32, u32, u32) {
    // 使用系统本地时区偏移（Windows: GetTimeZoneInformation）
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days: i64 = local_days(secs);

    fn local_days(secs: i64) -> i64 {
        #[cfg(windows)]
        {
            #[repr(C)]
            struct Tzi {
                bias: i32,
                standard_name: [u16; 32],
                standard_date: [u32; 8],
                standard_bias: i32,
                daylight_name: [u16; 32],
                daylight_date: [u32; 8],
                daylight_bias: i32,
            }
            unsafe {
                let mut tzi: Tzi = std::mem::zeroed();
                #[link(name = "kernel32")]
                extern "system" {
                    fn GetTimeZoneInformation(lp: *mut Tzi) -> u32;
                }
                let r = GetTimeZoneInformation(&mut tzi);
                let mut bias = tzi.bias;
                if r == 2 {
                    bias += tzi.daylight_bias;
                }
                // 本地时间 = UTC - bias(分钟)
                (secs - (bias as i64) * 60).div_euclid(86_400)
            }
        }
        #[cfg(not(windows))]
        {
            secs.div_euclid(86_400)
        }
    }
    let (y, m, d) = civil_from_days(days);
    (y as i32, m, d)
}

fn gen_version() {
    let profile = env::var("PROFILE").unwrap_or_default();
    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let (y, m, d) = local_ymd();
    let yymmdd = format!("{:02}{:02}{:02}", y % 100, m, d);

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let state_file = manifest_dir.join("build_no.txt");

    let forced = env::var("BUILD_COUNT").ok().and_then(|v| v.parse::<i64>().ok());

    let mut stored_date = String::new();
    let mut stored_count: i64 = 0;
    if let Ok(txt) = fs::read_to_string(&state_file) {
        let mut it = txt.split_whitespace();
        if let Some(dt) = it.next() {
            stored_date = dt.to_string();
        }
        if let Some(c) = it.next().and_then(|v| v.parse().ok()) {
            stored_count = c;
        }
    }

    let is_release = profile == "release";
    let count = if let Some(c) = forced {
        c
    } else if stored_date == yymmdd {
        if is_release {
            stored_count + 1
        } else {
            stored_count
        }
    } else {
        0
    };

    // release 构建改写状态文件，保证下一次构建必然重跑（序号递增）
    if is_release && forced.is_none() {
        let _ = fs::write(&state_file, format!("{} {}", yymmdd, count));
    }

    let full_version = format!("{}.{}.{}", pkg_version, yymmdd, count);
    println!("cargo:rustc-env=FULL_VERSION={}", full_version);
    println!("cargo:rustc-env=BUILD_YYMMDD={}", yymmdd);
    println!("cargo:rustc-env=BUILD_COUNT={}", count);
    println!("cargo:rerun-if-changed=ui/");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build_no.txt");
    println!("cargo:rerun-if-env-changed=BUILD_COUNT");
    println!(
        "cargo:warning=showkeyboard_client version: {} (profile={})",
        full_version, profile
    );
}
