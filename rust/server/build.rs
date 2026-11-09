//! 版本号生成脚本
//!
//! 版本格式（与 Node.js `src/version.js` 一致）：
//!     `主版本号.YYMMDD.当日构建序号`
//! 例如：`1.56.0.261105.0`
//!
//! 规则：
//! - 主版本号来自 Cargo.toml 的 `package.version`
//! - `YYMMDD` 为编译当天日期（每次 release 编译都会更新）
//! - 当日构建序号从 0 开始，同一天每编译一次 +1
//! - 序号持久化在 crate 根目录的 `build_no.txt`（内容：`<YYMMDD> <序号>`）
//! - 只有 `--release` 编译才会累加序号；debug 编译只读取当前值（不消耗序号）
//! - 可通过环境变量 `BUILD_COUNT` 强制指定序号（不写状态文件，用于 CI）

use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let state_path = Path::new(&manifest_dir).join("build_no.txt");

    let now = chrono::Local::now();
    let yymmdd = now.format("%y%m%d").to_string();

    // PROFILE = "release" / "debug"（自定义 profile 使用其名字）
    let is_release = std::env::var("PROFILE")
        .map(|p| p == "release")
        .unwrap_or(false);

    // 读取状态文件: "<YYMMDD> <count>"
    let (stored_date, stored_count) = fs::read_to_string(&state_path)
        .ok()
        .and_then(|s| {
            let mut it = s.split_whitespace();
            let d = it.next()?.to_string();
            let c = it.next().and_then(|x| x.parse::<u32>().ok()).unwrap_or(0);
            Some((d, c))
        })
        .unwrap_or_default();

    let build_count = if let Ok(forced) = std::env::var("BUILD_COUNT") {
        // 显式指定（CI 用），不改写状态文件
        forced.trim().parse::<u32>().unwrap_or(0)
    } else if is_release {
        // 同一天：序号 +1；新的一天：从 0 开始
        let next = if stored_date == yymmdd {
            stored_count + 1
        } else {
            0
        };
        if let Err(e) = fs::write(&state_path, format!("{} {}\n", yymmdd, next)) {
            println!("cargo:warning=failed to write build_no.txt: {}", e);
        }
        next
    } else {
        // debug 编译不消耗序号
        if stored_date == yymmdd {
            stored_count
        } else {
            0
        }
    };

    // 主版本号来自 Cargo.toml package.version
    let pkg_version =
        std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());

    // 完整版本号: "1.56.0.261105.0"
    let full_version = format!("{}.{}.{}", pkg_version, yymmdd, build_count);

    println!("cargo:rustc-env=FULL_VERSION={}", full_version);
    println!("cargo:rustc-env=BUILD_YYMMDD={}", yymmdd);
    println!("cargo:rustc-env=BUILD_COUNT={}", build_count);

    // 源文件变化时重新执行本脚本
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");
    // build_no.txt 在 release 编译时被本脚本改写，因此每次 release 编译都会
    // 触发本脚本重跑 → 日期/序号得到更新（debug 编译只在首次检测到变化时重跑一次）
    println!("cargo:rerun-if-changed=build_no.txt");
    // 允许通过环境变量强制指定序号
    println!("cargo:rerun-if-env-changed=BUILD_COUNT");

    println!(
        "cargo:warning=showkeyboard_server version: {} (profile={})",
        full_version,
        if is_release { "release" } else { "debug" }
    );
}
