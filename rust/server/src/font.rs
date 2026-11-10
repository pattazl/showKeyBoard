/// System font enumeration matching Node.js `font-list` npm package behavior.
/// 
/// Node.js `font-list` uses:
///   - Windows: EnumFontFamiliesEx via Win32 API
///   - macOS: NSFontManager
///   - Linux: fc-list
///
/// We use system commands to achieve similar results without heavy native deps.

use std::sync::OnceLock;

pub fn get_system_fonts() -> Vec<String> {
    // 字体列表在进程生命周期内不会变化，缓存避免每次 /getPara 重复枚举
    //（macOS 的 system_profiler 兜底路径较慢）
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();
    CACHE.get_or_init(compute_system_fonts).clone()
}

fn compute_system_fonts() -> Vec<String> {
    let result = if cfg!(target_os = "windows") {
        get_fonts_windows()
    } else if cfg!(target_os = "macos") {
        get_fonts_macos()
    } else {
        get_fonts_linux()
    };
    
    if result.is_empty() {
        // Fallback: common CJK + Latin fonts
        return vec![
            "Arial".to_string(),
            "Microsoft YaHei".to_string(),
            "SimSun".to_string(),
            "SimHei".to_string(),
            "KaiTi".to_string(),
            "FangSong".to_string(),
            "DengXian".to_string(),
            "YouYuan".to_string(),
            "NSimSun".to_string(),
            "MingLiU".to_string(),
            "PMingLiU".to_string(),
            "DFKai-SB".to_string(),
            "MS Gothic".to_string(),
            "MS Mincho".to_string(),
            "Yu Gothic".to_string(),
        ];
    }
    
    result
}

/// Windows: enumerate fonts via PowerShell + System.Drawing
fn get_fonts_windows() -> Vec<String> {
    // Use Add-Type to load System.Drawing, then enumerate installed fonts
    let script = r#"
        Add-Type -AssemblyName System.Drawing -ErrorAction SilentlyContinue
        if ($?) {
            (New-Object System.Drawing.Text.InstalledFontCollection).Families | ForEach-Object { $_.Name }
        } else {
            # Fallback: list from registry
            Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts' | 
            Get-Member -MemberType NoteProperty | 
            ForEach-Object { $_.Name } | 
            Where-Object { $_ -notmatch '\(.*\)' }
        }
    "#;
    
    // PowerShell 字体枚举是内部实现，任何情况下都不能弹窗：
    // 不带 CREATE_NO_WINDOW 时会在桌面上闪出控制台窗口（与 showHttpDebug 无关，
    // showHttpDebug 只控制 showkeyboard_server 自身窗口由客户端 spawn 时决定）
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        return match std::process::Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect()
            }
            _ => get_fonts_windows_fallback(),
        };
    }
    #[cfg(not(windows))]
    {
        let _ = script;
        get_fonts_windows_fallback()
    }
}

/// Windows 兜底：直接列 C:\Windows\Fonts 字体文件
fn get_fonts_windows_fallback() -> Vec<String> {
    if let Ok(entries) = std::fs::read_dir("C:\\Windows\\Fonts") {
                let mut fonts = Vec::new();
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if let Some(stem) = name.strip_suffix(".ttf")
                        .or_else(|| name.strip_suffix(".TTF"))
                        .or_else(|| name.strip_suffix(".otf"))
                        .or_else(|| name.strip_suffix(".OTF"))
                    {
                        fonts.push(stem.to_string());
                    }
                }
                fonts.sort();
                fonts.dedup();
                return fonts;
            }
            Vec::new()
}

/// macOS: 优先使用 CoreText 枚举字体家族（等价 Node font-list 的 NSFontManager，
/// 速度快且无需外部命令）；CoreText 返回空时回退到 system_profiler。
fn get_fonts_macos() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        let fonts = get_fonts_macos_coretext();
        if !fonts.is_empty() {
            return fonts;
        }
    }

    match std::process::Command::new("sh")
        .args(["-c", "system_profiler SPFontsDataType 2>/dev/null | grep 'Family:' | sed 's/.*Family: //' | sort -u"])
        .output()
    {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        }
        _ => Vec::new(),
    }
}

/// CoreText / CoreFoundation FFI：枚举系统可用字体家族
#[cfg(target_os = "macos")]
fn get_fonts_macos_coretext() -> Vec<String> {
    use std::ffi::c_void;

    type CFArrayRef = *const c_void;
    type CFStringRef = *const c_void;
    // kCFStringEncodingUTF8
    const UTF8_ENCODING: u32 = 0x0800_0100;

    #[link(name = "CoreText", kind = "framework")]
    extern "C" {
        fn CTFontManagerCopyAvailableFontFamilyNames() -> CFArrayRef;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFArrayGetCount(array: CFArrayRef) -> isize;
        fn CFArrayGetValueAtIndex(array: CFArrayRef, index: isize) -> *const c_void;
        fn CFStringGetLength(s: CFStringRef) -> isize;
        fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
        fn CFStringGetCString(
            s: CFStringRef,
            buffer: *mut u8,
            buffer_size: isize,
            encoding: u32,
        ) -> u8;
        fn CFRelease(cf: *const c_void);
    }

    let array = unsafe { CTFontManagerCopyAvailableFontFamilyNames() };
    if array.is_null() {
        return Vec::new();
    }

    let mut fonts = Vec::new();
    unsafe {
        let count = CFArrayGetCount(array);
        for i in 0..count {
            let s = CFArrayGetValueAtIndex(array, i) as CFStringRef;
            if s.is_null() {
                continue;
            }
            let len = CFStringGetLength(s);
            if len <= 0 {
                continue;
            }
            let cap = CFStringGetMaximumSizeForEncoding(len, UTF8_ENCODING) + 1;
            if cap <= 0 {
                continue;
            }
            let mut buf = vec![0u8; cap as usize];
            if CFStringGetCString(s, buf.as_mut_ptr(), cap, UTF8_ENCODING) != 0 {
                let name = String::from_utf8_lossy(&buf)
                    .trim_end_matches('\0')
                    .trim()
                    .to_string();
                if !name.is_empty() {
                    fonts.push(name);
                }
            }
        }
        CFRelease(array as *const c_void);
    }

    fonts.sort();
    fonts.dedup();
    fonts
}

/// Linux: use fc-list
fn get_fonts_linux() -> Vec<String> {
    match std::process::Command::new("fc-list")
        .args([":", "family"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let mut fonts: Vec<String> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .flat_map(|l| {
                    // fc-list output: "family1,family2:style=Regular"
                    l.split(':').next().unwrap_or("")
                        .split(',')
                        .map(|f| f.trim().to_string())
                        .filter(|f| !f.is_empty())
                        .collect::<Vec<_>>()
                })
                .collect();
            fonts.sort();
            fonts.dedup();
            fonts
        }
        _ => Vec::new(),
    }
}
