;编译信息
;@Ahk2Exe-SetName ShowKeyBoard
;@Ahk2Exe-SetDescription Show and Analyse Mouse/KeyBoard
;@Ahk2Exe-SetProductVersion 1.57.0.0
;@Ahk2Exe-SetFileVersion 1.57.0.0
;@Ahk2Exe-SetCopyright Austing.Young (2023 - )
;@Ahk2Exe-SetMainIcon res\keyboard.ico
;@Ahk2Exe-ExeName build/release/ShowKeyBoard.exe
#Requires AutoHotkey v2
#SingleInstance Ignore
global APPName := "ShowKeyBoard", ver:= "1.57"
#Include "lib/JSON.ahk"
#include common.ahk
#include langVars.ahk
#Include events.ahk
; 正式代码开始
loop skipRecord.length {
  skipKeys := defaultSkipKeys "{" GetKeyName(skipRecord[A_Index]) "}"
}
; 切换是否显示按键
Switch4show() {
  global needShowKey := not needShowKey
  UpdatMenu4Show()
}
; 尝试绑定热键
try {
  Hotkey hotkey4Show, Switch4show
}
; 不要阻塞按键
CountCtrlKey()
{
  global ctrlKeyCount := 0
}
~LCtrl Up:: CountCtrlKey
~RCtrl Up:: CountCtrlKey
~LShift Up:: CountCtrlKey
~RShift Up:: CountCtrlKey
~LWin Up:: CountCtrlKey
~RWin Up:: CountCtrlKey
~LAlt Up:: CountCtrlKey
~RAlt Up:: CountCtrlKey

~LCtrl:: SendCtrlKey
~RCtrl:: SendCtrlKey
~LShift:: SendCtrlKey
~RShift:: SendCtrlKey
~LWin:: SendCtrlKey
~RWin:: SendCtrlKey
~LAlt:: SendCtrlKey
~RAlt:: SendCtrlKey
SendCtrlKey()
{
  if (skipCtrlKey = 0) {
    ; pressKey := GetKeyName(StrReplace(StrReplace(A_ThisHotkey,'~',''),' Up',''))
    if ctrlKeyCount < maxCtrlpressCount
    {
      pressKey := GetKeyName(StrReplace(A_ThisHotkey, '~', ''))
      PushTxt pressKey
      global ctrlKeyCount += 1
    }
  }
}
; 鼠标事件
#HotIf InStr(skipKeys, "{WheelUp}") = 0
~WheelUp:: SendMouse

#HotIf InStr(skipKeys, "{WheelDown}") = 0
~WheelDown:: SendMouse

#HotIf InStr(skipKeys, "{LButton}") = 0
~LButton:: SendMouse

#HotIf InStr(skipKeys, "{MButton}") = 0
~MButton:: SendMouse

#HotIf InStr(skipKeys, "{RButton}") = 0
~RButton:: SendMouse

#HotIf InStr(skipKeys, "{WheelLeft}") = 0
~WheelLeft:: SendMouse

#HotIf InStr(skipKeys, "{WheelRight}") = 0
~WheelRight:: SendMouse

#HotIf InStr(skipKeys, "{XButton1}") = 0
~XButton1:: SendMouse

#HotIf InStr(skipKeys, "{XButton2}") = 0
~XButton2:: SendMouse

SendMouse()
{
  if (showMouseEvent > 0) {
    PushTxt(GetKeyName(StrReplace(A_ThisHotkey, '~', '')), True)
  }
}
CoordMode "ToolTip", "Screen"
CoordMode "Mouse", "Screen"
GetDistance(){
  try {
    GetDistanceCore()
  }
}
GetDistanceCore() {
  MouseGetPos &currentX, &currentY
  if currentX < minLeft || currentX > maxRight || currentY < minTop || currentY > maxBottom
  {
    ; OutputDebug  'AHK currentX:'  currentX ',currentY:' currentY ' minLeft ' minLeft ' maxRight' maxRight ' minTop ' minTop ' maxBottom ' maxBottom
    if (currentY > maxBottom * 100 || currentX > maxRight * 100) {
      ; 坐标差异太大，可能锁屏状态下直接抛弃数据
      return
    }
    ; 暂时抛弃异常数据，更新屏幕信息
    ; MsgBox 'currentX:'  currentX ',currentY:' currentY ' minLeft ' minLeft ' maxRight' maxRight ' minTop ' minTop ' maxBottom ' maxBottom
    SendPCInfo(1)
    ; Sleep(500) ;  可能是锁屏状态，等待下，不必马上循环 ; 防止影响按键响应取消sleep
    return
  }
  ; 计算鼠标移动距离
  ; 计算两点间的直线距离
  distance := Integer(Sqrt((currentX - mouseStartX) ** 2 + (currentY - mouseStartY) ** 2))

  if distance > 0 {
    global mouseDistance += distance
    ; 在命令行窗口中输出距离
    ;ShowTxt 'Distance: ' mouseDistance
    AllKeyRecord['mouseDistance'] := mouseDistance
    global mouseStartX := currentX
    global mouseStartY := currentY
  }
}
; 是否需要记录距离
if recordMouseMove = 1 {
  SetTimer(GetDistance, 100)  ; 每n毫秒记录一次位置
}
; 判断某内容是否在数组中
HasVal(arr, val) {
  For Index, Value in arr {
    if Value == val {
      return Index
    }
  }
  return 0
}
; 是否统计分钟数据
GetMinuteData(isMouse, isPress) {
  ; 如果同时调用
  if (GetMinuteDataFlag) {
    OutputDebug ("AutoHotkey GetMinuteData ")
    return
  }
  ;startT := A_TickCount
  global GetMinuteDataFlag := True
  GetMinuteDataCore(isMouse, isPress)
  global GetMinuteDataFlag := False
  ;OutputDebug ("AutoHotkey GetMinuteData: " (A_TickCount - startT) ' : ' A_TickCount)
}
; 统计核心
GetMinuteDataCore(isMouse, isPress) {
  currMinute := FormatTime(A_Now, "yyyyMMddHHmm")
  initMouse := 0
  initKey := 0
  if isPress {
    if isMouse {
      initMouse := 1
    } else {
      initKey := 1
    }
  }
  AppPath := GetProcPath()
  if(AppPath=''){
    return
  }
  ; 有操作数据才更新APP信息
  currData := Map("Minute", currMinute, "MouseCount", globalMouseCount, "KeyCount", globalKeyCount
    , "Distance", mouseDistance, "Apps", Map(AppPath, Map("Mouse", initMouse, "Key", initKey)))
  Len := MinuteRecords.Length  ; 最后的数据不准确，不能计算
  MaxLen := 10
  if Len > MaxLen {
    ; 清理 MaxLen 个以外 的数据，为了减少带宽，至少为 MaxLen 分钟数据
    ; MaxLen分钟内没有任何点击，但有鼠标移动,因为只有按键或点击才上传数据
    MinuteRecords.removeAt(1)
  }
  if Len == 0 {
    MinuteRecords.push(currData)
    return  ; 刚才开始不用做任何计算
  }
  ; 最后一个时间不一样则插入
  last := MinuteRecords[-1]
  if last['Minute'] != currMinute {
    ; 更新之前的数据,由总数变成统计数据
    last['MouseCount'] := globalMouseCount - last['MouseCount']
    last['KeyCount'] := globalKeyCount - last['KeyCount']
    last['Distance'] := mouseDistance - last['Distance']
    ; 如果 Distance 距离小于 50 ，则认为没移动
    if last['MouseCount'] = 0 && last['KeyCount'] = 0 && last['Distance'] <50 {
      MinuteRecords[-1] := currData ; 因为为空数据，那么则直接替换
    } else {
      ; 插入新数据
      MinuteRecords.push(currData)
    }
  } else {
    ; 同一分钟内，那么需要添加不同的APP名
    if isPress {
      if (last["Apps"].has(AppPath)) {
        if isMouse {
          last["Apps"][AppPath]["Mouse"] += 1
        } else {
          last["Apps"][AppPath]["Key"] += 1
        }
      } else {
        last["Apps"][AppPath] := Map("Mouse", initMouse, "Key", initKey)
      }
    }
  }
}
GetMinuteDataTimer() {
  GetMinuteData(False, False)
}
if recordMinute = 1 {
  SetTimer(GetMinuteDataTimer, 1000)  ; 每1秒刷新一次分钟数据
}

; 计算重启时间
today := SubStr(A_Now, 1, 8)
tomorrow := DateAdd(today, 1, "Days")
ShutDownLeft()
{
  ; 距离明天凌晨 0:00:05 的秒数，+5秒是为了给系统时间不准留点余量
  return (DateDiff(tomorrow, A_Now, "Seconds") + 5) * 1000
}
; 设置一个计时器用于跨夜时重启进程以便保存当日数据并开始新的一天
SetTimer(Reload, ShutDownLeft())
; 托盘相关
global MyMenu
global LinkPath := A_Startup "\" APPName ".Lnk"
MenuHandler(ItemName, ItemPos, MyMenu) {
  if (ItemName = L_menu_startup)
  {
    If FileExist(LinkPath)
    {
      FileDelete LinkPath
      MyMenu.Uncheck(L_menu_startup)
    }
    else
    {
      FileCreateShortcut A_ScriptFullPath, A_Startup "\" APPName ".Lnk", A_ScriptDir
      MyMenu.Check(L_menu_startup)
    }
  }
  if (ItemName = L_menu_reload)
  {
    Reload()
  }
  if (ItemName = L_menu_reset)
  {
    ExitServer()
    Reload()
  }
  if (ItemName = L_menu_exit)
  {
    ExitServer()
    ExitApp()
  }
  if (ItemName = L_menu_set)
  {
    if serverState = 1 {
      Run serverUrl "/Setting"
    } else {
      MsgBox menu_msg_noserver
    }
  }
  if (ItemName = L_menu_stat)
  {
    if serverState = 1 {
      Run serverUrl "/Today"
    } else {
      MsgBox menu_msg_noserver
    }
  }
  if (ItemName = L_menu_pause)
  {
    if A_IsPaused = 1 {
      Pause(False)
      MyMenu.Uncheck(L_menu_pause)
    } else {
      Pause(True)
      MyMenu.Check(L_menu_pause)
    }
  }
  if (ItemName = L_menu_4show)
  {
    Switch4show()
  }
}
UpdatMenu4Show() {
  if needShowKey {
    A_TrayMenu.Check(L_menu_4show)
  } else {
    A_TrayMenu.Uncheck(L_menu_4show)
  }
}
CreateMenu()
{
  A_IconTip := APPName " v" ver
  if (needTraytip) {
    TrayTip(A_IconTip)                ; 托盘提示信息
  }
  MyMenu := A_TrayMenu
  ; 清空默认菜单
  MyMenu.Delete()
  MyMenu.Add(L_menu_startup, MenuHandler)
  MyMenu.Add(L_menu_reload, MenuHandler)
  if needRecordKey = 1 {
    MyMenu.Add(L_menu_reset, MenuHandler)
  }
  MyMenu.Add(L_menu_pause, MenuHandler)
  MyMenu.Add(L_menu_4show, MenuHandler)
  UpdatMenu4Show()
  MyMenu.Add(L_menu_set, MenuHandler)
  MyMenu.Add(L_menu_stat, MenuHandler)
  MyMenu.Add(L_menu_exit, MenuHandler)
  ; 初始化默认状态
  If FileExist(LinkPath)
  {
    FileGetShortcut LinkPath, &OutTarget, &OutDir, &OutArgs, &OutDesc, &OutIcon, &OutIconNum, &OutRunState
    if (OutTarget = A_ScriptFullPath) {
      MyMenu.Check(L_menu_startup)
    }
  }
}
CreateMenu()
; 是否可以控制隐藏的窗口
DetectHiddenWindows(true)
CloseGetKeyInput(){
  ; 如果是未编译的脚本
  str := getKeyInputTitle " ahk_class AutoHotkey"
  if WinExist(str) {
      PostMessage 0x0010, 0, 0, , str
  }
}
; 关闭前需要退出后台服务
OnExit ExitFunc
ExitFunc(ExitReason, ExitCode)
{
  ;if ExitReason != "Logoff" and ExitReason != "Shutdown"
  ;{
  ;Result := MsgBox("Are you sure you want to exit?",, 4)
  ;if Result = "No"
  ;    return 1  ; Callbacks must return non-zero to avoid exit.
  ;}
  ; 需要将 临时的配置开关保存
  If FileExist(IniFile) {
    ; 当文件中的参数没有被人修改时才写入状态，否则以文件中数据为准
    tempVal := DescRead("common", "needShowKey", "1")
    if tempVal != needShowKey {
      IniWrite(needShowKey, IniFile, "common", "needShowKey")
    }
  }
  ; 检查后台服务情况
  pidPath := httpPath 'kbserver.pid'
  lastRecordPath := httpPath 'lastRecord.json'
  If FileExist(pidPath) {
    ; 如果存在进程文件，那么要保存到临时文件中，后端重启启动后优先载入临时文件内容更新数据
    If FileExist(lastRecordPath) {
      FileDelete lastRecordPath   ; 先删
    }
    ; 对于 AllKeyRecord 中最后的数据需要处理
    if MinuteRecords.Length > 0 {
      last := MinuteRecords[-1]
      last['MouseCount'] := globalMouseCount - last['MouseCount']
      last['KeyCount'] := globalKeyCount - last['KeyCount']
      last['Distance'] := mouseDistance - last['Distance']
      last['LastFlag'] := 1  ; 标注为最后的一次
    }
    FileAppend(JSON.stringify(AllKeyRecord, 0), lastRecordPath)
  }
  CloseGetKeyInput()
}

#Include dialog.ahk

; 原先的方法监控全局按键，缺点:快速按键时候容易丢失
; ; 建立钩子抓取数据,默认不要阻塞 V I0
; ih := InputHook("V I99")   ; Level 定为100，可以忽略一些 send 发送的字符，默认send的level 为0
; ; 设置所有按键的监听
; ih.KeyOpt("{All}", "NE")  ; End
; ; 去掉控制按键的响应计数
; ih.KeyOpt(skipKeys, "-E")
; MyKeyUp(ih ,VK, SC)
; {
; ; OutputDebug ("AutoHotkey Up:" GetKeyName(Format("vk{:x}sc{:x}", VK, SC)) )
; global repeatRecord := 0
; }
; ih.OnKeyUp := MyKeyUp
; KeyWaitCombo()
; {
; ;InputHook.VisibleText := true
; ;InputHook.VisibleNonText  := true
; ; 开始监控
; ih.Start()
; ;OutputDebug ("AutoHotkey InProgress " . ih.InProgress )
; ih.Wait()
; global repeatRecord
; if(repeatRecord < maxKeypressCount)
; {
; ; OutputDebug ("AutoHotkey " . ih.EndMods   . ih.EndKey ) ; 类似 <^<+Esc
; ;OutputDebug ("AutoHotkey KeyState " . GetKeyState(ih.EndKey, "P") ) ; 类似 <^<+Esc
; PushTxt( ih.EndMods . ih.EndKey )
; repeatRecord += 1  ;防止重复记录
; }
; ;OutputDebug ("AutoHotkey : " . ih.EndMods . ih.EndKey . " InProgress:" . ih.InProgress )
; ;return ih.EndMods . ih.EndKey  ; Return a string like <^<+Esc
; }

; ; 这个可能导致死循环，必须最后
; Loop {
; KeyWaitCombo()
; }

; 第三种方法监控全局按键，缺点: 可能丢失一些未知的按键
; LogKey(thisKey) {
; ; Critical -1
; k := GetKeyName(vksc := SubStr(A_ThisHotkey, 3))
; k := StrReplace(k, "Control", "Ctrl"), r := SubStr(k, 2)
; ; 判断 Shift 键是否按下
; isShiftDown := GetKeyState("Shift")
; ; 判断 Control 键是否按下
; isCtrlDown := GetKeyState("Control")
; OutputDebug ("AutoHotkey key:"  thisKey " k: "  k ' shift: ' isShiftDown ' Control: ' isCtrlDown )
; }

; SetHotkey(f := 0) {
; ; These keys are already used as hotkeys
; global UsedKeys
; f := f ? "On" : "Off"
; Loop 254 {
; k := GetKeyName(vk := Format("vk{:X}", A_Index))
; if k != "" && !("Control" = k) && !("Alt" = k) && !("Shift" = k) &&
; !("F1" = k) && !("F2" = k) && !("F3" = k) && !("F4" = k) &&
; !("F5" = k) && !("F6" = k) && !("F9" = k) {
; try {
; Hotkey "~*" vk, LogKey, f
; } catch as e {
; }
; }

; }
; For i, k in StrSplit("NumpadEnter|Home|End|PgUp"
; . "|PgDn|Left|Right|Up|Down|Delete|Insert", "|")
; {
; sc := Format("sc{:03X}", GetKeySC(k))
; if not k = "" && !("Control" = k) && !("Alt" = k) && !("Shift" = k) &&
; !("F1" = k) && !("F2" = k) && !("F3" = k) && !("F4" = k) &&
; !("F5" = k) && !("F6" = k) && !("F9" = k) {
; try {
; Hotkey "~*" sc, LogKey, f
; } catch as e {
; }
; }
; }
; }

; SetHotkey(1)

; 启动接收函数用于接收按键消息
ReceiveKeyInput(){
    ; 监听 WM_COPYDATA 消息 (0x004A)
  OnMessage(0x004A, Receive_WM_COPYDATA)

  Receive_WM_COPYDATA(wParam, lParam, msg, hwnd) {
      ; 从 lParam 指向的 COPYDATASTRUCT 结构中读取数据
      ; 结构布局: [dwData, cbData, lpData]
      ; 每个字段在 64 位系统中占 8 字节 (A_PtrSize)
      
      ; 1. 获取字符串数据的内存地址 (lpData)
      ; 偏移量为 2 * A_PtrSize 是因为前两个字段各占 A_PtrSize 字节
      dataAddress := NumGet(lParam + 2 * A_PtrSize, "Ptr")
      
      ; 2. 获取字符串数据的字节大小 (cbData)
      dataSize := NumGet(lParam + A_PtrSize, "UInt")
      
      ; 3. 从内存地址中读取字符串
      ; 除以 2 得到字符数 (因为是 UTF-16)
      receivedString := StrGet(dataAddress, dataSize // 2, "UTF-16")
      
      ; 显示接收到的内容
      ; 使用 ToolTip 而不是 MsgBox 以避免阻塞消息处理
      sendKeySep := '|'  ; 链接分隔符,和发送端一致，用于合并快速发送的的消息
      keyArr := StrSplit(receivedString, sendKeySep)
      For index, key in keyArr
      {
        PushTxt(key)
      }
      ; 返回值 1 表示已处理该消息（这是惯例）
      return 1
  }
}
ReceiveKeyInput()
; 启动进程用于读取按键
CreateGetKeyInput(){
  try {
    Run(getKeyInputTitle " " maxKeypressCount " " skipKeys)
  } catch {
    MsgBox(msgNotLaunchHook ":" getKeyInputTitle)
  }
}
CreateGetKeyInput()

; 以下为对 游戏手柄的按键读取
; 可以支持多个摇杆
maxJoyPressCount := 1 ; 设置为长按只有一次，如果某个按键一直按住没松开过，那么不大于 maxKeypressCount 次
joyNameMap := Map()

; 以下为方案1的手柄读取方案
ControllerNumber := [] ; 默认无摇杆,可能出现1没有，但2有的情况
JoyNameList := [] ; 游戏手柄清单
TestJoyList := ["Z", "R", "U", "V", "P"] ; ["JoyX","JoyY","JoyZ","JoyR","JoyU","JoyV","JoyPOV"]
JoyList := Map()
; 检查游戏手柄整体信息
checkJoyInfo() {
  global ControllerNumber
  global JoyNameList
  global JoyList
  tmpControllerNumber := [] ; 默认无摇杆
  tmpJoyNameList := []
  Loop 4  ; Query each controller number to find out which ones exist.
  {
    joyName := GetKeyState(A_Index "JoyName")

    ; 需要有控制器名，且能获取到X的浮点数据
    if joyName
    {
      JoyXType := Type(GetKeyState(A_Index "JoyX"))  ; Float
      if (JoyXType != "Float") {
        Reload()  ; 识别数据异常，可能手柄拔出了，目前AHK存在bug，只有重启可正确识别
        return
      }
      tmpJoyNameList.Push(joyName)
      tmpControllerNumber.Push(A_Index)
    }
  }
  ; 检查新的游戏列表是否和之前的一样,一样则跳过
  if (arrayEqual(tmpJoyNameList, JoyNameList)) {
    ; OutputDebug("Joy same")
    return
  } else {
    ; 获取摇杆后，获取最大按键数，绑定所有按键
    for index in tmpControllerNumber {
      cont_buttons := GetKeyState(index "JoyButtons")
      ControllerPrefix := index "Joy"
      Loop cont_buttons {
        Hotkey ControllerPrefix . A_Index, MyShowKey  ; () => MyShowKey
      }
    }
    JoyList := Map()
    for index in tmpControllerNumber {
      cont_info := GetKeyState(index "JoyInfo")
      JoyList[index] := ["X", "Y"] ; 必然有 X Y
      for value in TestJoyList {
        if InStr(cont_info, value) {
          if (value == "P")
          {
            value := "POV"  ; 需要转换下
          }
          JoyList[index].push(value)  ; 真实有效的摇杆清单
        }
      }
    }
    ; 需要设置生效
    ControllerNumber := tmpControllerNumber
    JoyNameList := tmpJoyNameList
  }
  ; for controlId in ControllerNumber {
  ; OutputDebug("Joy controlId:" controlId)
  ; }
  ;   for joyname in JoyNameList {
  ; OutputDebug("Joy joyname:" joyname)
  ; }
}
; 比较数组一致性
arrayEqual(arr1, arr2) {
  if (arr1.Length != arr2.Length)
    return false
  for i, v in arr1
    if (v != arr2[i])
      return false
  return true
}
MyShowKey(*) {
  PushTxt(A_ThisHotkey)
  ; OutputDebug( "Joy :" A_ThisHotkey)
}
; 不同类别的按键数据代表不同含义
getJoyInfo(controlId, joyN) {
  global joyNameMap
  joyFullName := controlId "Joy" joyN
  if (!joyNameMap.has(joyFullName)) {
    joyNameMap[joyFullName] := 0
  }
  keyName := ""
  val := 50 ; 默认50，表示没动作或复位
  ; 存在才计算
  val := Round(GetKeyState(joyFullName))
  Switch joyN
  {
    Case "Z":
      ; LT / RT
      if (val < 50) {
        keyName := controlId "JoyRT"
      }
      if (val > 50) {
        keyName := controlId "JoyLT"
      }
    Case "POV":
      ; -1  0,4500,9000,13500,18000,22500,27000,31500, 有8个方向
      initPov := 0
      loop 8 {
        if (val == initPov) {
          keyName := joyFullName A_Index
          break
        }
        initPov += 4500
      }
    Default:
      if (val != 50) {
        keyName := joyFullName
      }
  }
  if keyName != "" {
    joyNameMap[joyFullName] += 1
    if (joyNameMap[joyFullName] <= maxJoyPressCount) {
      PushTxt(keyName)
      ; OutputDebug "Joy :" keyName
    }
  } else {
    joyNameMap[joyFullName] := 0 ; 松开
  }
}
checkJoyState()
{
  global JoyList
  global ControllerNumber
  for controlId in ControllerNumber {
    try {
      for index, value in JoyList[controlId] {
        getJoyInfo(controlId, value)
      }
    }
  }
}

; 以下为方案2的手柄监控方案，直接调用XINPUT接口
#Include "lib/XInput.ahk"
; 不同类别的按键数据代表不同含义
getJoyInfo2(controlId, State) {
  ; Constants for gamepad buttons
  XINPUT_GAMEPAD_DPAD_UP := 0x0001
  XINPUT_GAMEPAD_DPAD_DOWN := 0x0002
  XINPUT_GAMEPAD_DPAD_LEFT := 0x0004
  XINPUT_GAMEPAD_DPAD_RIGHT := 0x0008
  XINPUT_GAMEPAD_START := 0x0010
  XINPUT_GAMEPAD_BACK := 0x0020
  XINPUT_GAMEPAD_LEFT_THUMB := 0x0040
  XINPUT_GAMEPAD_RIGHT_THUMB := 0x0080
  XINPUT_GAMEPAD_LEFT_SHOULDER := 0x0100
  XINPUT_GAMEPAD_RIGHT_SHOULDER := 0x0200
  XINPUT_GAMEPAD_GUIDE := 0x0400
  XINPUT_GAMEPAD_A := 0x1000
  XINPUT_GAMEPAD_B := 0x2000
  XINPUT_GAMEPAD_X := 0x4000
  XINPUT_GAMEPAD_Y := 0x8000

  global joyNameMap

  keyNameArr := [] ; 按键是个数组

  ; 方向键，有8个方向，按 上，上右，右，顺时针排序数组
  POVArr := [XINPUT_GAMEPAD_DPAD_UP, XINPUT_GAMEPAD_DPAD_UP | XINPUT_GAMEPAD_DPAD_RIGHT, XINPUT_GAMEPAD_DPAD_RIGHT, XINPUT_GAMEPAD_DPAD_RIGHT | XINPUT_GAMEPAD_DPAD_DOWN,
    XINPUT_GAMEPAD_DPAD_DOWN, XINPUT_GAMEPAD_DPAD_DOWN | XINPUT_GAMEPAD_DPAD_LEFT, XINPUT_GAMEPAD_DPAD_LEFT, XINPUT_GAMEPAD_DPAD_LEFT | XINPUT_GAMEPAD_DPAD_UP]
  low_4_bits := State.wButtons & 0x000F ; 低4位为方向
  for index, poVal in POVArr {
    if (low_4_bits == poVal) {
      keyNameArr.Push(controlId "JoyPOV" index)
      break ; 只可能满足1个，所以可以退出
    }
  }
  ; 按键的顺序编号
  ButtonArr := [XINPUT_GAMEPAD_A, XINPUT_GAMEPAD_B, XINPUT_GAMEPAD_X, XINPUT_GAMEPAD_Y, XINPUT_GAMEPAD_LEFT_SHOULDER, XINPUT_GAMEPAD_RIGHT_SHOULDER,
    XINPUT_GAMEPAD_BACK, XINPUT_GAMEPAD_START, XINPUT_GAMEPAD_LEFT_THUMB, XINPUT_GAMEPAD_RIGHT_THUMB, XINPUT_GAMEPAD_GUIDE
  ]
  for index, val in ButtonArr {
    if (State.wButtons & val) {
      keyNameArr.Push(controlId "Joy" index)
    }
  }
  ; 扳机 ,假设10为触发
  triggerHolds := 10
  thumBholds := 2000 ; 数据较为灵敏
  if (State.bLeftTrigger > triggerHolds) {
    keyNameArr.Push(controlId "JoyLT")
  }
  if (State.bRightTrigger > triggerHolds) {
    keyNameArr.Push(controlId "JoyRT")
  }
  ; X ,Y (左摇杆),R,U(右摇杆)
  if (Abs(State.sThumbLX) > thumBholds) {
    keyNameArr.Push(controlId "JoyX")
  }
  if (Abs(State.sThumbLY) > thumBholds) {
    keyNameArr.Push(controlId "JoyY")
  }
  if (Abs(State.sThumbRX) > thumBholds) {
    keyNameArr.Push(controlId "JoyU")
  }
  if (Abs(State.sThumbRY) > thumBholds) {
    keyNameArr.Push(controlId "JoyR")
  }
  ; 按下则 +1
  for index, joyFullName in keyNameArr {
    ; 不存在就创建一个对象
    if (!joyNameMap.has(joyFullName)) {
      joyNameMap[joyFullName] := 0
    }
    joyNameMap[joyFullName] += 1
    if (joyNameMap[joyFullName] <= maxJoyPressCount) {
      PushTxt(joyFullName)
      ; OutputDebug "Joy :" keyName
    }
  }
  ; 不在 keyNameArr 中的按键则全部复位为 0
  for key, val in joyNameMap {
    ; 如果不是当前设备的，跳过
    if(InStr(key, controlId "Joy") == 0){
      continue
    }
    if (!IsInArray(keyNameArr, key)) {
      joyNameMap[key] := 0 ; 松开
    }
  }

}
; 判断字符串是否存在于数组中（区分大小写）
IsInArray(arr, searchStr) {
  for element in arr {
    if (element = searchStr)
      return true
  }
  return false
}
checkJoyState2()
{
  Loop 4 {
    if State := XInput_GetState(A_Index - 1) {
      ; OutputDebug("Joy " A_Index " dwPacketNumber: " State.dwPacketNumber " wButtons:" State.wButtons
      ; " LT:" State.bLeftTrigger " RT:" State.bRightTrigger " sThumbLX:" State.sThumbLX " sThumbLY:" State.sThumbLY " sThumbRX:" State.sThumbRX " sThumbRY:" State.sThumbRY)
      ; 解析 State
      getJoyInfo2(A_Index, State)
    }
  }
}

; 正式启用方案1
if ( joyMethod == 1) {
  SetTimer(checkJoyState, 100)  ; 0.1秒检测一次
  SetTimer(checkJoyInfo, 2000)  ; 2秒检测一次
} else if (joyMethod == 2)
{
  ; 方案2
  XInput_Init()
  SetTimer(checkJoyState2, 100)  ; 0.1秒检测一次
}

; 初始化托盘图标的核心函数
InitTrayIcon() {
    currentPath := A_ScriptFullPath
    ; 获取不带扩展名的文件名（用于构造ICO文件名）
    fileNameNoExt := SubStr(A_ScriptName, 1, -StrLen('EXE') - 1)
    ; 构造ICO文件路径（与脚本/程序同目录，同名ICO）
    icoPath := StrReplace(currentPath, A_ScriptName, fileNameNoExt ".ico")
    ; 检查ICO文件是否存在
    if (FileExist(icoPath)) {
        ; 存在同名ICO文件，使用它
        TraySetIcon(icoPath)
    }
}
InitTrayIcon()
SetTimer( setMyTitle,-1 )
setMyTitle(){
  WinSetTitle('ShowKeyBoardMainUI',"ahk_id "  A_ScriptHwnd)
}