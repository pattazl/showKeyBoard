# ShowKeyBoard Rust 客户端（Slint UI 版）

showKeyBoard.ahk 的 Rust 全量重写。

## 功能对照

| AHK 原功能 | 本实现 |
|---|---|
| 全局键盘钩子（hook/getKeyInput.ahk，经 WM_COPYDATA 转发） | `WH_KEYBOARD_LL` 低级钩子直连进程内通道，无跨进程消息 |
| 鼠标按键/滚轮/X1/X2 | `WH_MOUSE_LL` 钩子 |
| 鼠标移动距离（recordMouseMove + 越界重取屏幕） | 100ms 轮询 + 虚拟屏边界校验，越界自动 `/sendPCInfo` |
| 游戏手柄（lib/XInput.ahk，joyMethod 1/2） | `joyGetPosEx` / `XInputGetState`（libloading 动态加载） |
| 按键统计/分钟统计/App 维度/历史（AllKeyRecord、MinuteRecords） | `stats.rs` 逐条移植 |
| showKeyBoard.ini / showKeyBoard.desc.ini（DescRead、GBK） | `config.rs`，含 KeyList.txt 映射 |
| HTTP /version /data /sendPCInfo /exit，断线重连、拉起 node 服务 | `net.rs`（ureq） |
| WebSocket ahkClient / IniMonitor 热重载 | `net.rs::start_ws`（tungstenite） |
| 浮窗：渐隐(guiFadeMs)、圆角(guiRadius)、自定义字体/背景/透明度、TL/TR/BL/BR、多屏、合并 ×N | `ui.rs` + `ui/app.slint`（Slint） |
| Ctrl 键状态窗、activeAppShow 路径窗 | `ui.rs` |
| 托盘菜单、hotkey4Show 热键、单实例、kb.pid、lastRecord.json、退出 /exit | `tray.rs` / `main.rs` |
| 版本号 `主版本.YYMMDD.当日序号` | `build.rs`（`cargo build --release` 自动递增） |

## 构建与运行

```powershell
cd rust
cargo build --release
```

运行前将以下文件放到 exe 同目录（与 AHK 版布局一致）：
- `showKeyBoard.ini`（必须）
- `KeyList.txt`（按键显示名映射）
- `ShowKeyBoard.ico`（可选，托盘图标）
- `httpDistPath` 指向 node 后台 dist 目录（用于拉起 server.js 与 lastRecord.json）

## 版本号规则

- `Cargo.toml` 的 `package.version` + `.YYMMDD` + `.当日次数`，例如 `1.61.0.261105.3`
- 每次 `cargo build --release` 自动 +1（状态存于本机 `build_no.txt`，跨天归零）
- debug 编译不消耗次数；`BUILD_COUNT=5 cargo build --release` 可固定序号
