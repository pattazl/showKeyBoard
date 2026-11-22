//! 崩溃现场记录（Windows）。
//!
//! `panic = "abort"` 下，Rust panic 与原生访问违例都会直接终止进程，
//! 默认拿不到任何线索。这里安装 panic 钩子和未处理异常过滤器，把
//! 异常码、故障地址、寄存器以及栈上的疑似返回地址（换算为相对模块基址
//! 的 RVA）追加写入数据目录的 crash.log，便于用同版本 exe 的 map 文件
//! 反查函数名。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static LOG_PATH: OnceLock<std::path::PathBuf> = OnceLock::new();
/// 启动信息先缓存在内存里，只有真正崩溃时才连同现场一起落盘，
/// 这样正常运行期间不会生成 crash.log。
static INIT_LINE: OnceLock<String> = OnceLock::new();
static INIT_WRITTEN: AtomicBool = AtomicBool::new(false);

pub fn install() {
    #[cfg(windows)]
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static INSTALLED: AtomicBool = AtomicBool::new(false);
        if INSTALLED.swap(true, Ordering::SeqCst) {
            return;
        }

        let dir = crate::config::data_dir();
        let _ = LOG_PATH.set(dir.join("crash.log"));
        let base = unsafe {
            windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null()) as usize
        };
        let (ts, sz) = pe_info(base);
        let _ = INIT_LINE.set(format!(
            "\n[init] {} version={} timestamp=0x{:08X} size=0x{:X} exe={}\n",
            now(),
            env!("FULL_VERSION"),
            ts,
            sz,
            std::env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        ));

        // panic 也留一份记录（钩子执行后由 abort 终止进程）
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let loc = info
                .location()
                .map(|l| l.to_string())
                .unwrap_or_else(|| "?".into());
            append(&format!("[panic] {} @ {}\n", info, loc));
            default_hook(info);
        }));

        unsafe {
            windows_sys::Win32::System::Diagnostics::Debug::SetUnhandledExceptionFilter(Some(
                filter,
            ));
        }
    }
}

fn append(s: &str) {
    use std::io::Write;
    let Some(p) = LOG_PATH.get() else { return };
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
        // 首次落盘时补上启动信息（正常运行时不会创建文件）
        if !INIT_WRITTEN.swap(true, Ordering::SeqCst) {
            if let Some(init) = INIT_LINE.get() {
                let _ = f.write_all(init.as_bytes());
            }
        }
        let _ = f.write_all(s.as_bytes());
        let _ = f.flush();
    }
}

fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

#[cfg(windows)]
unsafe extern "system" fn filter(
    info: *const windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS,
) -> i32 {
    use windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_CONTINUE_SEARCH;

    if !info.is_null() {
        let er = &*(*info).ExceptionRecord;
        let ctx = &*(*info).ContextRecord;
        let base = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null())
            as usize;
        let fault = er.ExceptionAddress as usize;
        let tid = windows_sys::Win32::System::Threading::GetCurrentThreadId();

        let mut msg = String::new();
        msg.push_str(&format!("[exception] {} thread={}\n", now(), tid));
        msg.push_str(&format!(
            "[exception] code=0x{:08X} fault=0x{:X} rva=0x{:X} module_base=0x{:X}\n",
            er.ExceptionCode,
            fault,
            fault.wrapping_sub(base),
            base
        ));
        // 记录镜像指纹：用同一次构建的 map 才能把 RVA 精确还原成函数名
        let (timestamp, image_size) = pe_info(base);
        msg.push_str(&format!(
            "[module] timestamp=0x{:08X} size=0x{:X} path={}\n",
            timestamp,
            image_size,
            std::env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        ));

        // 访问违例：ExceptionInformation[0]=0 读 / 1 写 / 8 取指，[1]=访问地址
        if er.ExceptionCode as u32 == 0xC000_0005 && er.NumberParameters >= 2 {
            let kind = match er.ExceptionInformation[0] {
                0 => "read",
                1 => "write",
                8 => "execute",
                _ => "?",
            };
            let addr = er.ExceptionInformation[1];
            msg.push_str(&format!(
                "[av] {} addr=0x{:X} mem={}\n",
                kind,
                addr,
                mem_state(addr)
            ));
        }
        msg.push_str(&format!(
            "[regs] rip=0x{:X} rsp=0x{:X} rbp=0x{:X} rax=0x{:X} rbx=0x{:X} rcx=0x{:X} rdx=0x{:X}\n",
            ctx.Rip as usize, ctx.Rsp as usize, ctx.Rbp as usize, ctx.Rax as usize,
            ctx.Rbx as usize, ctx.Rcx as usize, ctx.Rdx as usize
        ));
        msg.push_str(&format!(
            "[regs] rsi=0x{:X} rdi=0x{:X} r8=0x{:X} r9=0x{:X} r10=0x{:X} r11=0x{:X} r12=0x{:X} r13=0x{:X} r14=0x{:X} r15=0x{:X}\n",
            ctx.Rsi as usize, ctx.Rdi as usize, ctx.R8 as usize, ctx.R9 as usize,
            ctx.R10 as usize, ctx.R11 as usize, ctx.R12 as usize, ctx.R13 as usize,
            ctx.R14 as usize, ctx.R15 as usize
        ));

        // 扫描栈顶 16KB：把落在本模块代码段内的值当作疑似返回地址记录下来
        let sp = ctx.Rsp as usize;
        for i in 0..2048usize {
            let addr = sp.wrapping_add(i * 8);
            if !readable(addr) {
                continue;
            }
            let v = *(addr as *const usize);
            if v > base + 0x1000 && v < base + 0x800000 {
                msg.push_str(&format!(
                    "[stack] +0x{:04X} rva=0x{:X} abs=0x{:X}\n",
                    i * 8,
                    v - base,
                    v
                ));
            }
        }
        append(&msg);
    }

    EXCEPTION_CONTINUE_SEARCH
}

#[cfg(windows)]
fn readable(addr: usize) -> bool {
    use windows_sys::Win32::System::Memory::{
        VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_GUARD, PAGE_NOACCESS,
    };
    let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let len = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();
    let r = unsafe { VirtualQuery(addr as *const _, &mut mbi, len) };
    r != 0 && mbi.State == MEM_COMMIT && (mbi.Protect & (PAGE_GUARD | PAGE_NOACCESS)) == 0
}

/// 描述地址所在内存区域：提交/保留/空闲、保护属性与区域大小
#[cfg(windows)]
fn mem_state(addr: usize) -> String {
    use windows_sys::Win32::System::Memory::{
        VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_FREE, MEM_RESERVE,
    };
    let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let r = unsafe {
        VirtualQuery(
            addr as *const _,
            &mut mbi,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    if r == 0 {
        return "unknown".into();
    }
    let state = match mbi.State {
        MEM_COMMIT => "commit",
        MEM_RESERVE => "reserve",
        MEM_FREE => "free",
        _ => "?",
    };
    format!(
        "{} prot=0x{:X} base=0x{:X} size=0x{:X}",
        state, mbi.Protect, mbi.BaseAddress as usize, mbi.RegionSize
    )
}

/// 读取模块 PE 头，返回 (TimeDateStamp, SizeOfImage)：
/// 用于确认崩溃设备上的 exe 与本地 map 是否出自同一次构建。
#[cfg(windows)]
fn pe_info(base: usize) -> (u32, u32) {
    unsafe {
        if base == 0 {
            return (0, 0);
        }
        let p = base as *const u8;
        let e_lfanew = *(p.add(0x3C) as *const u32) as usize;
        if e_lfanew > 0x1000 {
            return (0, 0);
        }
        let nt = p.add(e_lfanew);
        if *(nt as *const u32) != 0x0000_4550 {
            return (0, 0);
        }
        let timestamp = *(nt.add(8) as *const u32);
        let image_size = *(nt.add(24).add(56) as *const u32);
        (timestamp, image_size)
    }
}
