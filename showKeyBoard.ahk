#Requires AutoHotkey v2
#SingleInstance Ignore
#include "common.ahk"
#Include events.ahk
; 正式代码开始
loop skipRecord.length {
	skipKeys := skipKeys "{" GetKeyName(skipRecord[A_Index]) "}"
}
; 不要阻塞按键
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
		PushTxt GetKeyName(StrReplace(A_ThisHotkey,'~',''))
	}
}

; 建立钩子抓取数据,默认不要阻塞 V I0
ih := InputHook("V I99")   ; Level 定为100，可以忽略一些 send 发送的字符，默认send的level 为0
; 设置所有按键的监听
ih.KeyOpt("{All}", "E")  ; End
; 去掉控制按键的响应计数
ih.KeyOpt(skipKeys, "-E")
KeyWaitCombo()
{
	;InputHook.VisibleText := true
	;InputHook.VisibleNonText  := true
	; 开始监控
    ih.Start()
	;OutputDebug ("AutoHotkey InProgress " . ih.InProgress )
    ih.Wait()
	;OutputDebug ("AutoHotkey " . ih.EndMods . ih.EndKey ) ; 类似 <^<+Esc
	PushTxt( ih.EndMods . ih.EndKey )
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
global L_menu_reload:="重新启动"
global L_menu_pause:="暂停运行"
global L_menu_set:="参数设置"
global L_menu_stat:="数据统计"
global L_menu_exit:="退出程序"
LinkPath := A_Startup "\" APPName ".Lnk"
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
      FileCreateShortcut A_ScriptFullPath, A_Startup "\" APPName ".Lnk", "A_ScriptDir"
      MyMenu.Check(L_menu_startup)
    }
  }
  if(ItemName = L_menu_reload)
  {
	Reload()
  }
  if(ItemName = L_menu_exit)
  {
    ExitServer()
	ExitApp()
  }
  if(ItemName = L_menu_set)
  {
	Run serverUrl "/setting"
  }
  if(ItemName = L_menu_stat)
  {
	Run serverUrl "/stat"
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
  MyMenu.Add(L_menu_pause, MenuHandler)
  MyMenu.Add(L_menu_set, MenuHandler)
  MyMenu.Add(L_menu_stat, MenuHandler)
  MyMenu.Add(L_menu_exit, MenuHandler)
  ; 初始化默认状态
  If FileExist(LinkPath)
  {
  
    MyMenu.Check(L_menu_startup)
  }
}
CreateMenu()

#Include dialog.ahk

; 这个可能导致死循环，必须最后
Loop {
    KeyWaitCombo()
}
