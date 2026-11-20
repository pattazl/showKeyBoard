#Requires AutoHotkey v2.0
; 发送 WM_CLOSE 消息 (0x0010)
DetectHiddenWindows(true)

; 如果是未编译的脚本
targetWindow := "getKeyInput ahk_class AutoHotkey"
if WinExist(targetWindow) {
    PostMessage 0x0010, 0, 0, , targetWindow
    MsgBox "已发送关闭信号"
}
else {
    MsgBox "找不到 Receiver"
}