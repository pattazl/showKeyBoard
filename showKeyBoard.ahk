;编译信息
;@Ahk2Exe-SetName ShowKeyBoard
;@Ahk2Exe-SetDescription Show and Analyse Mouse/KeyBoard
;@Ahk2Exe-SetProductVersion 1.15.0.0
;@Ahk2Exe-SetFileVersion 1.15.0.0
;@Ahk2Exe-SetCopyright Austing.Young (2023 - )
;@Ahk2Exe-SetMainIcon res\keyboard.ico
;@Ahk2Exe-ExeName build/ShowKeyBoard.exe
#Requires AutoHotkey v2
#SingleInstance Ignore
global APPName:="ShowKeyBoard", ver:="1.15" 
#include "common.ahk"
#Include events.ahk
; 正式代码开始
loop skipRecord.length {
	skipKeys := skipKeys "{" GetKeyName(skipRecord[A_Index]) "}"
}
; 切换是否显示按键
Switch4show(Key){
    global needShowKey := not needShowKey
    UpdatMenu4Show()
}
; 尝试绑定热键
try{
    Hotkey hotkey4Show, Switch4show
}
; 不要阻塞按键
CountCtrlKey()
{
    global ctrlKeyCount := 0
}
~LCtrl Up::CountCtrlKey
~RCtrl Up::CountCtrlKey
~LShift Up::CountCtrlKey
~RShift Up::CountCtrlKey
~LWin Up::CountCtrlKey
~RWin Up::CountCtrlKey
~LAlt Up::CountCtrlKey
~RAlt Up::CountCtrlKey

~LCtrl::SendCtrlKey
~RCtrl::SendCtrlKey
~LShift::SendCtrlKey
~RShift::SendCtrlKey
~LWin::SendCtrlKey
~RWin::SendCtrlKey
~LAlt::SendCtrlKey
~RAlt::SendCtrlKey
SendCtrlKey()
{
	if(skipCtrlKey = 0){
        ; pressKey := GetKeyName(StrReplace(StrReplace(A_ThisHotkey,'~',''),' Up',''))
        if ctrlKeyCount < maxCtrlpressCount
        {
        pressKey := GetKeyName(StrReplace(A_ThisHotkey,'~',''))
        PushTxt pressKey
        global ctrlKeyCount += 1
        }
    }
}
; 鼠标事件
~WheelUp::SendMouse
~WheelDown::SendMouse
~LButton::SendMouse
~MButton::SendMouse
~RButton::SendMouse
SendMouse()
{
	if(showMouseEvent > 0){
		PushTxt(GetKeyName(StrReplace(A_ThisHotkey,'~','')),True)
	}
}
; 鼠标开始的位置和距离
mouseStartX := 0
mouseStartY := 0
mouseDistance := 0
CoordMode "Mouse", "Screen"
GetDistance(){
	MouseGetPos &currentX, &currentY
	; OutputDebug  'currentX:'  currentX ',currentY:' currentY ' minLeft ' minLeft ' maxRight' maxRight ' minTop ' minTop ' maxBottom ' maxBottom
    if currentX<minLeft || currentX> maxRight || currentY< minTop || currentY> maxBottom
    {
        ; 暂时抛弃异常数据，更新屏幕信息
        ; MsgBox 'currentX:'  currentX ',currentY:' currentY ' minLeft ' minLeft ' maxRight' maxRight ' minTop ' minTop ' maxBottom ' maxBottom
        SendPCInfo()
        return
    }
    ; 计算鼠标移动距离
	; 计算两点间的直线距离
	distance := Integer(Sqrt((currentX - mouseStartX) ** 2 + (currentY - mouseStartY) ** 2))

	if distance > 0{
    global mouseDistance += distance
    ; 在命令行窗口中输出距离
    ;ShowTxt 'Distance: ' mouseDistance
		AllKeyRecord['mouseDistance'] := mouseDistance
		global mouseStartX := currentX
		global mouseStartY := currentY
	}
}
; 是否需要记录距离
if recordMouseMove = 1{
	SetTimer(GetDistance,100)  ; 每n毫秒记录一次位置
}
; 建立钩子抓取数据,默认不要阻塞 V I0
ih := InputHook("V I99")   ; Level 定为100，可以忽略一些 send 发送的字符，默认send的level 为0
; 设置所有按键的监听
ih.KeyOpt("{All}", "NE")  ; End
; 去掉控制按键的响应计数
ih.KeyOpt(skipKeys, "-E")
MyKeyUp(ih ,VK, SC)
{
    ; OutputDebug ("AutoHotkey Up:" GetKeyName(Format("vk{:x}sc{:x}", VK, SC)) )
    global repeatRecord := 0
}
ih.OnKeyUp := MyKeyUp
KeyWaitCombo()
{
	;InputHook.VisibleText := true
	;InputHook.VisibleNonText  := true
	; 开始监控
    ih.Start()
	;OutputDebug ("AutoHotkey InProgress " . ih.InProgress )
    ih.Wait()
    global repeatRecord
    if(repeatRecord < maxKeypressCount)
    {
	;OutputDebug ("AutoHotkey " . ih.EndMods '--' . ih.EndKey ) ; 类似 <^<+Esc
	;OutputDebug ("AutoHotkey KeyState " . GetKeyState(ih.EndKey, "P") ) ; 类似 <^<+Esc
        PushTxt( ih.EndMods . ih.EndKey )
        repeatRecord += 1  ;防止重复记录
    }
	;OutputDebug ("AutoHotkey : " . ih.EndMods . ih.EndKey . " InProgress:" . ih.InProgress )
    ;return ih.EndMods . ih.EndKey  ; Return a string like <^<+Esc
}

; 计算重启时间
today:=SubStr(A_Now, 1, 8)
tomorrow:=DateAdd(today, 1, "Days")
ShutDownLeft()
{
   ; 距离明天凌晨 0:00:05 的秒数，+5秒是为了给系统时间不准留点余量
  return  (DateDiff(tomorrow, A_Now, "Seconds")+5)*1000
}
; 设置一个计时器用于跨夜时重启进程以便保存当日数据并开始新的一天
SetTimer( Reload, ShutDownLeft() )
; 托盘相关
global MyMenu
global L_menu_startup:="开机启动"
global L_menu_reload:="重启程序"
global L_menu_reset:="完全重启"
global L_menu_pause:="暂停运行"
global L_menu_4show:="按键显示(" hotkey4Show ")"
global L_menu_set:="参数设置"
global L_menu_stat:="数据统计"
global L_menu_exit:="退出程序"
LinkPath := A_Startup "\" APPName ".Lnk"
menu_msg_noserver := '服务未成功启动，不能用界面查看和设置'

MenuHandler(ItemName , ItemPos, MyMenu){
  if(ItemName = L_menu_startup)
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
  if(ItemName = L_menu_reload)
  {
	Reload()
  }
  if(ItemName = L_menu_reset)
  {
	ExitServer()
	Reload()
  }
  if(ItemName = L_menu_exit)
  {
    ExitServer()
	ExitApp()
  }
  if(ItemName = L_menu_set)
  {
	if serverState = 1 {
		Run serverUrl "/Setting"
	}else{
		MsgBox menu_msg_noserver
	}
  }
  if(ItemName = L_menu_stat)
  {
	if serverState = 1 {
		Run serverUrl "/Today"
	}else{
		MsgBox menu_msg_noserver
	}
  }
  if(ItemName = L_menu_pause)
  {
	if A_IsPaused =1 {
		Pause(False)
		MyMenu.Uncheck(L_menu_pause)
	}else{
		Pause(True)
		MyMenu.Check(L_menu_pause)
	}
  }
  if(ItemName = L_menu_4show)
  {
    Switch4show(0)
  }
}
UpdatMenu4Show(){
	if needShowKey {
		A_TrayMenu.Check(L_menu_4show)
	}else{
		A_TrayMenu.Uncheck(L_menu_4show)
	}
}  
CreateMenu()
{
  A_IconTip := APPName " v" ver
  TrayTip(A_IconTip)                ; 托盘提示信息
  MyMenu := A_TrayMenu 
  ; 清空默认菜单
  MyMenu.Delete()
  MyMenu.Add(L_menu_startup, MenuHandler)
  MyMenu.Add(L_menu_reload, MenuHandler)
  if needRecordKey = 1{
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
	if(OutTarget = A_ScriptFullPath){
		MyMenu.Check(L_menu_startup)
	}
  }
}
CreateMenu()
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
    If FileExist(IniFile){
        IniWrite(needShowKey,IniFile,"common","needShowKey")
    }
    ; 检查后台服务情况
	pidPath := httpPath 'kbserver.pid'
	lastRecordPath := httpPath 'lastRecord.json'
	If FileExist(pidPath){
	; 如果存在进程文件，那么要保存到临时文件中，后端重启启动后优先载入临时文件内容更新数据
		If FileExist(lastRecordPath){
			FileDelete lastRecordPath   ; 先删
		}
		FileAppend(jxon_dump(AllKeyRecord, indent:=0), lastRecordPath)
	}
}

#Include dialog.ahk

; 这个可能导致死循环，必须最后
Loop {
    KeyWaitCombo()
}
