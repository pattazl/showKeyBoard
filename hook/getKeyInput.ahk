#Requires AutoHotkey v2.0
;@Ahk2Exe-SetMainIcon hookKey.ico
#SingleInstance Ignore
#NoTrayIcon
; 脚本一直运行，一直发送消息
Persistent
; 目标窗口的标识。这里使用接收端脚本的文件名和类名。
; AutoHotkey 隐藏主窗口的类名通常是 "AutoHotkey"
; 名字不能错根据窗口类名和进程名发消息 "Receiver.ahk ahk_class AutoHotkey"
targetWindowTitle := "ShowKeyBoardMainUI"
; 参数启动的时候传递过来
maxKeypressCount := 10
skipKeys := "{LCtrl}{RCtrl}{LShift}{RShift}{LWin}{RWin}{LAlt}{RAlt}"
sendKeySep := '|'  ; 链接分隔符
global repeatRecord:=0
DetectHiddenWindows(true)  ; 需要能够找到隐藏的接收端窗口
SetTitleMatchMode(3)  ; 精确匹配
tarPid:=0
try{
  tarPid := WinGetPID(targetWindowTitle)
}
OutputDebug("AHK: tarPid" tarPid)
if( tarPid = 0 ){
  OutputDebug("AHK: no targetWindow")
  Exit()
}
targetWindow :="ahk_pid " tarPid
if A_Args.Length > 0
{
  try {
    ; 
    maxKeypressCount := Integer(A_Args[1] )
    skipKeys := A_Args[2]
  }
  OutputDebug("AHK: skipKeys " skipKeys "," maxKeypressCount )
}

Join(arr, sep := "") {
    result := ""
    for i, v in arr
        result .= (i = 1 ? "" : sep) . v
    return result
}
SendStringViaWM_COPYDATA_Queue(str) {
    static queue := []
    static timerStarted := false
    
    ; 将请求加入队列
    queue.Push(str)
    doInterval := 100
    ; 启动处理定时器（每 doInterval ms处理一次）
    if !timerStarted {
        SetTimer(ProcessQueue, doInterval)  ; 立即启动，但只运行一次
        timerStarted := true
    }
    
    ProcessQueue() {
        if queue.Length = 0 {
            return
        }
        ; 取出全部请求合并处理
        item := Join(queue,sendKeySep)
        queue.Length := 0
        
        ; 发送消息，但设置超时
        cds := Buffer(3 * A_PtrSize, 0)
        byteCount := (StrLen(item) + 1) * 2
        
        NumPut("Ptr", 12345, cds, 0)
        NumPut("UInt", byteCount, cds, A_PtrSize)
        NumPut("Ptr", StrPtr(item), cds, 2 * A_PtrSize)
        
        ; 使用SendMessage但带超时
        try {
          res := SendMessage(0x004A, 0, cds, , targetWindow, , , , 4000)  ; 1秒超时
          ;res := PostMessage(0x004A, 0, cds, , targetWinTitle)  ; 4秒超时
        } catch  {
          OutputDebug("AHK: err SendMessage")
        }
    }
}
PushTxt(textToSend){
    ; OutputDebug("AHK:" textToSend)
    ; 要发送的字符串
    ;textToSend := "Hello from Sender!"
    ; 调用发送函数，需要使用队列防止卡顿
    SendStringViaWM_COPYDATA_Queue(textToSend)
    ; result := SendStringViaWM_COPYDATA(textToSend)
    ; 检查发送结果
    ; if (result = 0) {
    ;     ;MsgBox "发送失败或目标窗口未响应。请确保 " targetWindow " 正在运行。"
    ; } else {
    ;     ;MsgBox "字符串发送成功，接收端已确认。"
    ; }
} 
; --------------------------------------------------------------
; 发送字符串的函数
; --------------------------------------------------------------
SendStringViaWM_COPYDATA(str) {
    ; 1. 准备 COPYDATASTRUCT
    ; 该结构需要 3 个指针大小的内存空间
    cds := Buffer(3 * A_PtrSize, 0)
    
    ; 计算字符串的字节大小 (UTF-16 编码，每个字符 2 字节，加上末尾的 null 终止符)
    byteCount := (StrLen(str) + 1) * 2
    
    ; 填充结构:
    ; 第一个字段 dwData: 可以是用户自定义的标识符，这里用 12345
    NumPut("Ptr", 12345, cds, 0)
    ; 第二个字段 cbData: 数据大小（字节）
    NumPut("UInt", byteCount, cds, A_PtrSize)
    ; 第三个字段 lpData: 字符串数据的内存地址
    NumPut("Ptr", StrPtr(str), cds, 2 * A_PtrSize)
    
    ; 2. 保存并设置必要的环境变量
    ; prevDHW := DetectHiddenWindows(true)  ; 需要能够找到隐藏的接收端窗口
    ; prevTMM := SetTitleMatchMode(2)       ; 使用部分标题匹配
    
    ; 3. 发送消息
    ; 使用 SendMessage (同步)，以确保在函数返回前数据已被处理
    ; 消息编号: 0x004A 即 WM_COPYDATA
    ; wParam: 0 (通常不使用)
    ; lParam: COPYDATASTRUCT 结构的指针
    res := 1
    try {
      res := SendMessage(0x004A, 0, cds, , targetWindow, , , , 1000)  ; 1秒超时
      ;res := PostMessage(0x0010, 0, cds, , targetWinTitle)  ; 4秒超时
    } catch  {
      OutputDebug("AHK: err SendMessage")
    }
    ; 4. 恢复原始设置
    ; DetectHiddenWindows(prevDHW)
    ; SetTitleMatchMode(prevTMM)
    
    return res
}

global hHookKeyboard := 0
HookKeyboard()
{
  ; 定义常量
  WH_KEYBOARD_LL := 13
  ; 注册回调函数
  callback := CallbackCreate(LowLevelKeyboardProc, "Fast", 3)
  ; 获取当前模块句柄
  moduleHandle := DllCall("GetModuleHandle", "UInt", 0, "Ptr")
  ; 全局键盘钩子
  hHookKeyboard := DllCall("SetWindowsHookExW",
    "Int", WH_KEYBOARD_LL,
    "Ptr", callback,
    "Ptr", moduleHandle,
    "UInt", 0,
    "Ptr")
}
; https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms644985(v=vs.85)
; 添加 <^ 这些符号，以确保一致
AddModifier() {
  Modfier := ''
  ModifierMapping := Map()
  ModifierMapping['LControl'] := '<^'
  ModifierMapping['RControl'] := '>^'
  ModifierMapping['LWin'] := '<#'
  ModifierMapping['RWin'] := '>#'
  ModifierMapping['LAlt'] := '<!'
  ModifierMapping['RAlt'] := '>!'
  ModifierMapping['LShift'] := '<+'
  ModifierMapping['RShift'] := '>+'
  ; 判断 Shift 键是否按下
  for key, val in ModifierMapping
  {
    if (GetKeyState(key))
    {
      Modfier .= val
    }
  }
  return Modfier
}

; skipKeys 中的 Ctrl 要转换为 Control
skipKeysDll := StrReplace(skipKeys, 'Ctrl', 'Control')
LowLevelKeyboardProc(nCode, wParam, lParam)
{
  ; Critical
  global repeatRecord
  if (wParam = 0x0101 or wParam = 0x0105)  ; KEYUP
  {
    repeatRecord := 0
  }
  ; 按住一个键，最多持续显示数量
  if (repeatRecord < maxKeypressCount) {
    flags := NumGet(lParam + 0, 8, "UInt") & 0x10                         ; 物理按下是0，模拟是非0。0x10 = 00010000
    if (nCode >= 0 and flags = 0 and (wParam = 0x0100 or wParam = 0x0104))  ; WM_KEYUP = 0x0101 WM_SYSKEYUP = 0x0105 WM_KEYDOWN = 0x0100 WM_SYSKEYDOWN = 0x0104
    {
      ; vk := NumGet(lParam+0, "UInt")                                  ; vk 不能区分数字键盘，所以用 sc
      Extended := NumGet(lParam + 0, 8, "UInt") & 0x1                   ; 扩展键（即功能键或者数字键盘上的键）是1，否则是0
        , sc := (Extended << 8) | NumGet(lParam + 0, 4, "UInt")
      Name := GetKeyName(Format("sc{:X}", sc))
      Name2 := '{' Name '}'
      if (InStr(skipKeysDll, Name2))
      {
        return DllCall("CallNextHookEx", "Ptr", 0, "Int", nCode, "UInt", wParam, "UInt", lParam)
      }
      repeatRecord += 1
      fullKey := AddModifier() Name  ; 控制键和具体按键结合
      ; OutputDebug("AutoHotkey sc " sc ' Name:' fullKey ' C:' repeatRecord )
      PushTxt(fullKey)
    }
  }
  ; CallNextHookEx 让其它钩子可以继续处理消息
  ; 返回非0值 例如1 告诉系统此消息将丢弃
  return DllCall("CallNextHookEx", "Ptr", 0, "Int", nCode, "UInt", wParam, "UInt", lParam)
}
HookKeyboard() 
OnExit ExitFunc
ExitFunc(ExitReason, ExitCode)
{
  OutputDebug("AHK: CLOSE")
  DllCall("UnhookWindowsHookEx", "UInt", hHookKeyboard)
}