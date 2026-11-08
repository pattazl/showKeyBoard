//! ICO 解码（纯 Rust，Windows / macOS 共用）：托盘图标来自 res/keyboard.ico。

/// 从 ICO 字节中挑选最合适的 32bpp 位图，返回 (RGBA, 宽, 高)。
/// 仅支持 BMP 内嵌条目（PNG 内嵌条目跳过），优先选择边长最接近 `prefer` 的条目。
pub fn ico_to_rgba(bytes: &[u8], prefer: u32) -> Option<(Vec<u8>, u32, u32)> {
    if bytes.len() < 6 {
        return None;
    }
    let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    let entry = |i: usize| -> Option<(i32, i32, u16, usize)> {
        let o = 6 + i * 16;
        if o + 16 > bytes.len() {
            return None;
        }
        let w = if bytes[o] == 0 { 256 } else { bytes[o] as i32 };
        let h = if bytes[o + 1] == 0 { 256 } else { bytes[o + 1] as i32 };
        let bpp = u16::from_le_bytes([bytes[o + 6], bytes[o + 7]]);
        let off = u32::from_le_bytes([bytes[o + 12], bytes[o + 13], bytes[o + 14], bytes[o + 15]]) as usize;
        Some((w, h, bpp, off))
    };
    let mut best: Option<(i32, usize)> = None;
    for i in 0..count {
        let Some((w, h, bpp, off)) = entry(i) else { break };
        if w <= 0 || w != h || bpp != 32 {
            continue;
        }
        let Some(data) = bytes.get(off..) else { continue };
        if data.len() < 40 || data.starts_with(&[0x89, b'P', b'N', b'G']) {
            continue;
        }
        let score = (w - prefer as i32).abs();
        if best.map(|(s, _)| score < s).unwrap_or(true) {
            best = Some((score, off));
        }
    }
    let (_, off) = best?;
    let data = bytes.get(off..)?;
    let bmp_w = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let bmp_h = i32::from_le_bytes([data[8], data[9], data[10], data[11]]) / 2;
    let header = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if bmp_w <= 0 || bmp_h <= 0 {
        return None;
    }
    let stride = bmp_w as usize * 4;
    let need = stride * bmp_h as usize;
    let pix = data.get(header..header + need)?;
    let mut rgba = Vec::with_capacity(need);
    // BMP 自底向上 + BGRA
    for y in (0..bmp_h as usize).rev() {
        for x in 0..bmp_w as usize {
            let i = y * stride + x * 4;
            rgba.extend_from_slice(&[pix[i + 2], pix[i + 1], pix[i], pix[i + 3]]);
        }
    }
    Some((rgba, bmp_w as u32, bmp_h as u32))
}
