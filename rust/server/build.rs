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
use std::path::{Path, PathBuf};

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

    // 把版本号写入可执行文件的 RT_VERSION 资源（文件属性 → 详细信息）
    embed_version_resource(&pkg_version, &full_version, build_count);

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

/// 生成 RT_VERSION(16) 资源并交给链接器（仅 Windows）。
/// 不依赖 rc.exe/windres：直接构造 .res 文件（32 字节空资源头 + 一条 RT_VERSION 记录）。
fn embed_version_resource(pkg_version: &str, full_version: &str, build_count: u32) {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut res = vec![0u8; 32];
    res[4..8].copy_from_slice(&32u32.to_le_bytes()); // HeaderSize
    res[8..12].copy_from_slice(&[0xFF, 0xFF, 0, 0]); // Type = 0
    res[12..16].copy_from_slice(&[0xFF, 0xFF, 0, 0]); // Name = 0

    let data = build_version_info(pkg_version, full_version, build_count);
    push_res(&mut res, 16, 1, &data);

    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    let res_path = PathBuf::from(out_dir).join("showkeyboard_version.res");
    match fs::write(&res_path, &res) {
        Ok(_) => println!("cargo:rustc-link-arg-bins={}", res_path.display()),
        Err(e) => println!("cargo:warning=failed to write version resource: {}", e),
    }
}

/// 构造 VS_VERSIONINFO：VS_VERSIONINFO -> VS_FIXEDFILEINFO + StringFileInfo -> StringTable -> String
fn build_version_info(pkg_version: &str, full_version: &str, build_count: u32) -> Vec<u8> {
    let parts: Vec<&str> = pkg_version.split('.').collect();
    let major = parts.first().and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
    let build = build_count.min(u16::MAX as u32) as u16;

    let mut fixed = Vec::with_capacity(52);
    fixed.extend_from_slice(&0xFEEF04BDu32.to_le_bytes()); // dwSignature
    fixed.extend_from_slice(&0x0001_0000u32.to_le_bytes()); // dwStrucVersion
    fixed.extend_from_slice(&((major as u32) << 16 | minor as u32).to_le_bytes()); // FileVersionMS
    fixed.extend_from_slice(&((patch as u32) << 16 | build as u32).to_le_bytes()); // FileVersionLS
    fixed.extend_from_slice(&((major as u32) << 16 | minor as u32).to_le_bytes()); // ProductVersionMS
    fixed.extend_from_slice(&((patch as u32) << 16 | build as u32).to_le_bytes()); // ProductVersionLS
    fixed.extend_from_slice(&0u32.to_le_bytes()); // dwFileFlagsMask
    fixed.extend_from_slice(&0u32.to_le_bytes()); // dwFileFlags
    fixed.extend_from_slice(&0x0004_0000u32.to_le_bytes()); // dwFileOS = VOS_NT_WINDOWS32
    fixed.extend_from_slice(&0x0000_0001u32.to_le_bytes()); // dwFileType = VFT_APP
    fixed.extend_from_slice(&0u32.to_le_bytes()); // dwFileSubtype
    fixed.extend_from_slice(&0u32.to_le_bytes()); // dwFileDateMS
    fixed.extend_from_slice(&0u32.to_le_bytes()); // dwFileDateLS

    let mut table = Vec::new();
    table.extend_from_slice(&0u16.to_le_bytes()); // wLength（占位）
    table.extend_from_slice(&0u16.to_le_bytes()); // wValueLength
    table.extend_from_slice(&1u16.to_le_bytes()); // wType = 1 (text)
    push_wstr(&mut table, "040904B0"); // 语言 0x0409、代码页 0x04B0(Unicode)
    align4(&mut table);

    let strings = [
        ("CompanyName", "Austin.Young".to_string()),
        ("FileDescription", "ShowKeyBoard Server".to_string()),
        ("FileVersion", full_version.to_string()),
        ("InternalName", "showkeyboard_server".to_string()),
        ("OriginalFilename", "showkeyboard_server.exe".to_string()),
        ("ProductName", "ShowKeyBoard".to_string()),
        ("ProductVersion", full_version.to_string()),
    ];
    for (key, value) in strings {
        push_version_string(&mut table, key, &value);
    }
    set_length(&mut table);

    let mut sfi = Vec::new();
    sfi.extend_from_slice(&0u16.to_le_bytes()); // wLength（占位）
    sfi.extend_from_slice(&0u16.to_le_bytes()); // wValueLength
    sfi.extend_from_slice(&1u16.to_le_bytes()); // wType = 1
    push_wstr(&mut sfi, "StringFileInfo");
    sfi.extend_from_slice(&table);
    set_length(&mut sfi);

    let mut info = Vec::new();
    info.extend_from_slice(&0u16.to_le_bytes()); // wLength（占位）
    info.extend_from_slice(&(fixed.len() as u16).to_le_bytes()); // wValueLength
    info.extend_from_slice(&0u16.to_le_bytes()); // wType = 0 (binary)
    push_wstr(&mut info, "VS_VERSION_INFO");
    align4(&mut info);
    info.extend_from_slice(&fixed);
    info.extend_from_slice(&sfi);
    set_length(&mut info);
    info
}

/// 补 0 使当前长度按 4 字节对齐（VS_VERSIONINFO 各子节点要求 DWORD 对齐）
fn align4(buf: &mut Vec<u8>) {
    while buf.len() % 4 != 0 {
        buf.push(0);
    }
}

/// 追加 UTF-16LE 字符串（含结尾 NUL）
fn push_wstr(out: &mut Vec<u8>, s: &str) {
    for c in s.encode_utf16() {
        out.extend_from_slice(&c.to_le_bytes());
    }
    out.extend_from_slice(&0u16.to_le_bytes());
}

/// 追加一条 String 节点：wLength / wValueLength(字符数) / wType=1 / szKey / Value
fn push_version_string(out: &mut Vec<u8>, key: &str, value: &str) {
    let mut node = Vec::new();
    node.extend_from_slice(&0u16.to_le_bytes()); // wLength（占位）
    node.extend_from_slice(&(value.encode_utf16().count() as u16).to_le_bytes());
    node.extend_from_slice(&1u16.to_le_bytes()); // wType = 1
    push_wstr(&mut node, key);
    while node.len() % 4 != 0 {
        node.push(0);
    }
    push_wstr(&mut node, value);
    while node.len() % 4 != 0 {
        node.push(0);
    }
    set_length(&mut node);
    out.extend_from_slice(&node);
}

/// 回填节点头部的 wLength（字节数，含头部自身）
fn set_length(node: &mut [u8]) {
    let len = node.len() as u16;
    node[0..2].copy_from_slice(&len.to_le_bytes());
}

/// 追加一条 .res 资源记录（头部固定 32 字节，数据 4 字节对齐）
fn push_res(out: &mut Vec<u8>, type_id: u16, name_id: u16, data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&32u32.to_le_bytes());
    out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    out.extend_from_slice(&type_id.to_le_bytes());
    out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    out.extend_from_slice(&name_id.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // DataVersion
    out.extend_from_slice(&0x1030u16.to_le_bytes()); // MemoryFlags
    out.extend_from_slice(&0x0409u16.to_le_bytes()); // LanguageId
    out.extend_from_slice(&0u32.to_le_bytes()); // Version
    out.extend_from_slice(&0u32.to_le_bytes()); // Characteristics
    out.extend_from_slice(data);
    while out.len() % 4 != 0 {
        out.push(0);
    }
}