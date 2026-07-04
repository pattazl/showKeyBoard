/// System font enumeration matching Node.js `font-list` npm package behavior.
/// 
/// Node.js `font-list` uses:
///   - Windows: EnumFontFamiliesEx via Win32 API
///   - macOS: NSFontManager
///   - Linux: fc-list
///
/// We use system commands to achieve similar results without heavy native deps.

pub fn get_system_fonts() -> Vec<String> {
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
    
    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
    {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        }
        _ => {
            // Last resort: list .ttf/.otf files in C:\Windows\Fonts
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
    }
}

/// macOS: use system_profiler to enumerate fonts
fn get_fonts_macos() -> Vec<String> {
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
