#Requires AutoHotkey v2.0
; 发送 WM_CLOSE 消息 (0x0010)
DetectHiddenWindows(true)

Run("getKeyInput " 12 " " "{LCtrl}{RCtrl}{LShift}{RShift}{LWin}{RWin}{LAlt}{RAlt}")