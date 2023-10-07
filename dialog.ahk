#include common.ahk
;#InstallKeybdHook true
#UseHook  ; Force the use of the hook for hotkeys after this point.

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; HookKeyboard()                          ; 键盘钩子


; 创建或显示内容
ShowTxt(text)
{
	if text = "" or not b_4show {
		return 
	}
    ; 获取活跃窗口进程匹配 FoundPos := RegExMatch(Haystack, NeedleRegEx [, &OutputVar, StartingPos])
    if activeWindowProc != ''{
        ; 获取活跃窗口 WinGetProcessName
        activeWnd := WinExist("A")
        if(activeWnd !=0 ){
            procName := WinGetProcessName(activeWnd)
            FoundPos := RegExMatch(procName, activeWindowProc )
            ; 没有找到匹配则不显示
            if FoundPos = 0{
                return
            }
        }else{
            return
        }
    }
    
	textArr := []
	; 如果正在处理中，则不要销毁窗口
	global guiShowing := 1
	; 获取不同屏幕
	MCount := MonitorGetCount()
	guiMonNum := guiMonitorNum
	if( guiMonNum > MCount )
	{
		guiMonNum := 1
	}
	MonitorGet(guiMonNum, &Left, &Top, &Right, &Bottom)
	needNewGui :=1
	; 先判断下数组最近的时间
	if (guiArr.Length>0){
		lastGui := guiArr[guiArr.Length]
		diff := A_TickCount - lastGui.tick
		;OutputDebug ("AutoHotkey : " diff)
		if( diff < guiInterval ){
		; 直接修改最近的输入框内容
			;lastGui.edit.Destroy()
			;调用API删除旧对象
			DllCall("user32\DestroyWindow", "Ptr", ControlGetHwnd(lastGui.edit))
			;ControlHide lastGui.edit  ; 原生代码不能销毁重新创建，只能隐藏
			MyGui := lastGui.gui
			textArr := lastGui.textArr
			needNewGui := 0
		}
	}
	if needNewGui=1 {
		MyGui := CreateGui(guiTextSize)
	}
	; Edit支持自动换行  BackgroundEEAA99 BackgroundTrans 高度自动
	editOpt := "Multi Background" guiBgcolor " +Wrap -Border +ReadOnly x0 y0 w" guiWidth " c" guiTextColor
	if guiHeigth = 0
	{
		editOpt := editOpt " -VScroll"
	}else
	{
		editOpt := editOpt " +VScroll h" guiHeigth
	}
	textArr.push(text)
	newText := Trim(GetKeyText(textArr)) ; 去掉首尾空格
	editType := "Edit"  ; 默认用edit类型可以滚动，但有边框需要额外去掉
	; 重新创建新的对象，便于计算新的高度，旧对象删除即可
	MyEdit:= MyGui.Add(editType, editOpt ,newText) ;   Edit Text
	; 去掉默认细边框
	if guiEdge =0 {
		WinSetExStyle  "-0x00000200", MyEdit
	}
	ControlGetPos &ex, &ey, &ew, &editHeight, MyEdit
	;MyEdit.Text := eh "-" text
	;lineCount := EditGetLineCount(MyEdit)
	;MsgBox editHeight
	; 需要获取屏幕分辨率 A_ScreenWidth A_ScreenHeight
	guiX := guiPosOffsetX + Left , guiY:=guiPosOffsetY +Top  ;  默认TL
	Switch guiPos
	{
	Case "TR":
		guiX := guiX+ Right - Left - guiWidth
	Case "BL":
		guiY := guiY+ Bottom-Top - editHeight
	Case "BR":
		guiX := guiX+ Right - Left- guiWidth
		guiY := guiY+ Bottom-Top - editHeight
	}
	; 窗体穿透
	if(guiTrans=1){
		WinSetExStyle  "+0x20", MyGui
	}
	; 真正显示
	MyGui.Show("NoActivate x" guiX " y" guiY " w" guiWidth " h" editHeight)	;WinSetTransparent guiOpacity, MyGui  ;WinSet, ExStyle, ^0x20  WS_EX_TRANSPARENT
	
	; 强制滚动到最后
	;ControlSend "^{End}", MyEdit, MyGui ; 不能用这个会导致触发按键
	SendMessage 0x0115, 7, 0, MyEdit

	if needNewGui{
		guiArr.push(
		{
		gui:MyGui,edit:MyEdit,tick:A_TickCount,textArr:textArr,
		x:guiX,y:guiY,w:guiWidth,h:editHeight
		})
	}Else{
		lastGui.edit := MyEdit
		lastGui.tick := A_TickCount
		lastGui.textArr := textArr
		lastGui.x := guiX
		lastGui.y :=guiY
		lastGui.w :=guiWidth
		lastGui.h :=editHeight
	}
	ReLayOut(guiX,guiY,guiWidth,editHeight)
	global guiShowing := 0
}

; 清理无需的窗口
CloseSelf()
{
	if guiShowing = 1
	{
		return
	}
	; 循环检查需要消失的窗口
	loop guiArr.Length {
		obj := guiArr[A_Index]
	    if( A_TickCount - obj.tick> guiLife)
		{
			guiArr.RemoveAt(A_Index)
			Sleep(100)   ; 需要等待一会儿再删
			obj.gui.Destroy()
			break
		}
	}
}
; 循环检查需要放入的数据
CallShow()
{
	if inArr.Length > 0 {
        val1 := inArr[1]
        bShow := 1
        loop skipShow.Length {
            if skipShow[A_Index] = val1 {
                bShow := 0
                break  ; 在不显示列表中跳过
            }
        }
        if(bShow=1){
            ShowTxt( ConvertTxt(val1) )
        }
		inArr.RemoveAt(1)
	}
}
; 定时器，自动从填写的队列中取数据
SetTimer( CallShow, 10)
; 定时器，检查需要关闭的窗口
SetTimer( CloseSelf, 100)
; 需要将之前的对话框位置移动，从最后往前移动
ReLayOut(x,y,w,h)
{
	px := x
	pw := w
	py := y
	ph := h
	loop guiArr.Length-1 {
		lastObj := guiArr[guiArr.Length-A_Index]
		guiX := lastObj.x
		guiY := lastObj.y
		if guiPosXY = "Y"{
			Switch substr(guiPos,1,1)
			{
			Case "T":
			; 从上往下排
				guiY := py + ph + guiMargin
			Case "B":
			; 从下往上排
				guiY := py - lastObj.h - guiMargin
			}
		}Else{
			Switch substr(guiPos,2,1)
			{
			Case "L":
			; 从左往右
				guiX := px + pw + guiMargin
			Case "R":
			; 从右往左
				guiX := px - lastObj.w - guiMargin
			}
		}
		lastObj.gui.Move(guiX,guiY)
		px := guiX
		pw := lastObj.w
		py := guiY
		ph := lastObj.h
	}
}
; 根据数组返回要显示的内容
GetKeyText(arr)
{
	text := ""
	preCh := ""
	preDiffText := ""
	preTextAll := ""
	count :=1
	loop arr.Length{
	; 如果一样用 ❌✖×
		if preCh = arr[A_Index]{
			count += 1
			text := preDiffText "×" count
		}Else{
		; 二次输入不一样
			count := 1
			text := arr[A_Index]
			if preTextAll!=""{
				text := preTextAll txtSplit text
			}
			preDiffText := text
		}
        preTextAll := text
		preCh  := arr[A_Index]
	}
	return text
}
; 通用的计算高地位方法
GetBitState(val,bitPos){
	return (val>>(bitPos-1))&1 ; 获取指定位后清理高位
}
; 将数据放到数组中
PushTxt(txt,isMouse:=False)
{
    if(needShowKey = 1 && (!isMouse || (isMouse && GetBitState(showMouseEvent,2)=1 ))){
        inArr.push(txt)
    }
	; 按键数量统计
	; OutputDebug "AutoHotkey - " txt
	if(needRecordKey = 1 && (!isMouse || (isMouse && GetBitState(showMouseEvent,1)=1 ))){
        RecordKey(txt)
    }
}
;记录全部按键统计
global PrePushKey :="" ; 前一次按键
RecordKey(txt)
{
	global PrePushKey
	AddRecord(txt) ; 统一添加
	;<^>^<!>!<+>+ 只在组合按键中出现
	mapping := Map()
	mapping['<^'] := 'LControl'
	mapping['>^'] := 'RControl'
	mapping['<#'] := 'LWin'
	mapping['>#'] := 'RWin'
	mapping['<!'] := 'LAlt'
	mapping['>!'] := 'RAlt'
	mapping['<+'] := 'LShift'
	mapping['>+'] := 'RShift'
    leftTxt := txt
	for key,val in mapping
	{
		if (InStr(leftTxt,key) >0 )
		{
            if(PrePushKey =val){
                AllKeyRecord[PrePushKey] -= 1 ; 因为会重复计算
            }
            AddRecord(val) ; 按键分解统计
            leftTxt := StrReplace(leftTxt,key,'')
		}
	}
    if( leftTxt != txt){
        AddRecord(leftTxt)
    }
	PrePushKey :=txt
    AutoSendData()  ; 发送数据给后端服务
}
; 如果不存在则创建，存在则+1
AddRecord(key){
    if( !AllKeyRecord.Has(key))
    {
        AllKeyRecord[key] :=0
    }
    AllKeyRecord[key] += 1
}
; 将<#内容转换为自定义的符号 {LCtrl}{RCtrl}{LAlt}{RAlt}{LShift}{RShift}{LWin}{RWin}
ConvertTxt(t){
	; 如果能直接匹配，直接返回
	if KeyMapping.Has(t) {
		return KeyMapping[t]
	}
	for key,val in KeyMapping{
		t := StrReplace(t, key, val)
	}
	return t
}
; 按键转换清单
GetKeyList(){
	Section := FileRead("KeyList.txt", "`n UTF-8")
	;MsgBox Section
	arr := StrSplit(Section,["`n","`r"])
	loop arr.Length{
		arr2 := StrSplit(arr[A_Index],":")
		if arr2.Length = 2{
			v1 := Trim(arr2[1])
			v2 := Trim(arr2[2])
			if v1!="" && v2!=""{
				KeyMapping[v1] := v2
			}
		}
	}
}
GetKeyList()
;ShowTxt("测试代码123❌4567890123↩45⇧6⎇78⌃90123456 ⇧ ⊞  ⬅ ⬆︎ ⬇ ➞ ⇐, ⇑, ⇒, ⇓ ␣ ⌘ 【 】1[PageUp] ➕")


; 显示控制键
global ctrlTextGui
global ctrlStateGui
ShowCtrlState(){
	;if(ctrlState)
	ctrlTxt:=''
	loop ctrlList.Length{
		keyName := ctrlList[A_Index]
		if( keyName == 'CapsLock'
		|| keyName == 'NumLock'
		|| keyName == 'ScrollLock'
		){
			res := GetKeyState(keyName,"T")
		}Else{
			res := GetKeyState(keyName,"P")
		}
		if res == 1{
			ctrlTxt:= ctrlTxt ConvertTxt(keyName) txtSplit
		}
	}
	global ctrlStateGui
	if( ctrlTxt !=''){
		global ctrlTextGui
		ctrlTextGui.Text := ctrlTxt
		ctrlStateGui.show("NoActivate")
	}Else{
		ctrlStateGui.hide()
	}
}
CreateCtrlState()
{
	global ctrlStateGui := CreateGui(ctrlTextSize)
	textOpt := "-Border x0 y0 c" guiTextColor
	; 重新创建新的对象，便于计算新的高度，旧对象删除即可
	txt :=""
	Loop ctrlList.Length{
		txt :=txt ConvertTxt(ctrlList[A_Index]) txtSplit
	}
	global ctrlTextGui:= ctrlStateGui.Add("Text", textOpt, txt) ; 
	; 窗体穿透
	if(guiTrans=1){
		WinSetExStyle  "+0x20", ctrlStateGui
	}
	ControlGetPos &ex, &ey, &ew, &editHeight, ctrlTextGui
	; 真正显示
	if(ctrlState=1){
		ctrlStateGui.Show("NoActivate x" ctrlX " y" ctrlY " w" ew " h" editHeight)
		; 显示控制键
		SetTimer(ShowCtrlState, 50)
	}
}
CreateCtrlState()

; 创建窗口返回窗口对象
CreateGui(TextSize)
{
	MyGui := Gui()
	; 设置窗体特性
	guiOpt := "+AlwaysOnTop -Caption +ToolWindow +Owner"  ; MyGui.Opt("+LastFound")
	if(guiDpiscale = 1){
		guiOpt := guiOpt " +DPIScale" 
	}else
	{
		guiOpt := guiOpt " -DPIScale" 
	}
	MyGui.Opt(guiOpt) 
	MyGui.BackColor := guiBgcolor  ; 设定任意颜色，然后下一行代码后设置窗口完全透明
	transColor := "" ; 默认透明度
	if(guiBgTrans = 1)
	{
		transColor := guiBgcolor  ;窗口完全透明，显示字体的透明度
	}Else{
		transColor := guiBgcolor  "A "  ; 只需要背景颜色和透明色不一样即可，这里颜色后面加了个A
	}
	transColor := transColor " " guiOpacity ; 设置窗体透明度
	WinSetTransColor(transColor, MyGui)  ; 窗口背景色设置
	MyGui.SetFont("s" TextSize " " guiTextWeight, guiTextFont)  ; Set a large font size (32-point).
	return MyGui
}