Persistent  ; 持久运行脚本
; 全局通用变量和函数
global IniFile := "showKeyBoard.ini"
global AllKeyRecord := Map()
global MinuteRecords := Array()
A_MaxHotkeysPerInterval := 240  ; 应对快速的宏操作
; 默认需要忽略的按键清单 "{LCtrl}{RCtrl}{LAlt}{RAlt}{LShift}{RShift}{LWin}{RWin}"
; 这些按键用独立的监控来发送
skipKeys := "{LCtrl}{RCtrl}{LShift}{RShift}{LWin}{RWin}{LAlt}{RAlt}"
; 基础配置文件读取
; 自定义函数，优先从目标的 showKeyBoard.desc.ini 文件中读取默认参数，如果没有才使用第三个默认参数
global DescIniPath := '', httpDistPath := '', httpPath := '', needRecordKey := -1
DescRead(sect, key, defaultVal) {
    global DescIniPath
    if DescIniPath = '' {
        global httpDistPath := IniRead(IniFile, "common", "httpDistPath", "\httpdist\dist\")
        global httpPath := A_WorkingDir httpDistPath   ; node 脚本所在目录
        if !FileExist(httpPath) {
            global needRecordKey := 0  ; 如果不存在则不用启动后端,后续也无需读取参数
        }
        tmpDescIni := A_WorkingDir httpDistPath 'showKeyBoard.desc.ini'
        if !FileExist(tmpDescIni) {
            DescIniPath := 'NotFound'
        } else {
            DescIniPath := tmpDescIni
        }
    }
    if DescIniPath != 'NotFound' {
        ; 存在默认配置文件，可以开始读取数据
        defaultVal := IniRead(DescIniPath, sect, key, defaultVal)
    }
    return IniRead(IniFile, sect, key, defaultVal)
}
; 将JS写入的ini文件中的格式进行转换
CovertJosn(s) {
    s := StrReplace(s, '\\', '\')
    s := StrReplace(s, '\"', '"')
    s := StrReplace(s, '\n', '`n')
    return s
}

; 按键转换清单
GetKeyList() {
    file := "KeyList.txt"
    If !FileExist(file) {
        return
    }
    global KeyMapping
    KeyMapping := Map()
    Section := FileRead(file, "`n UTF-8")
    ;MsgBox Section
    arr := StrSplit(Section, ["`n", "`r"])
    loop arr.Length {
        arr2 := StrSplit(arr[A_Index], ":")
        if arr2.Length = 2 {
            v1 := Trim(arr2[1])
            v2 := Trim(arr2[2])
            if v1 != "" && v2 != "" {
                KeyMapping[v1] := v2
            }
        }
    }
}
; 读取全部配置文件参数
ReadAllIni() {
    global joyMethod,joyMethod := DescRead("common", "joyMethod", "2")
    ; 游戏手柄监控方案
    global needTraytip,needTraytip := DescRead("common", "needTraytip", "1")
    ; 是否启动时候托盘提示
    global skipRecord,skipRecord := StrSplit(DescRead("common", "skipRecord", ""), '|')
    ; 哪些按键要忽略记录
    global skipCtrlKey,skipCtrlKey := DescRead("common", "skipCtrlKey", "0")
    ; 是否忽略单独的控制键，不记录
    global showMouseEvent,showMouseEvent := DescRead("common", "showMouseEvent", "1")
    ; 是否显示和记录鼠标事件
    global recordMouseMove,recordMouseMove := DescRead("common", "recordMouseMove", "0")
    ; 是否记录鼠标移动距离
    global recordMinute,recordMinute := DescRead("common", "recordMinute", "1")
    ; 是否记录分钟数据
    global needShowKey,needShowKey := DescRead("common", "needShowKey", "1")
    ; 是否显示按键
    global needRecordKey
    if needRecordKey = -1 {
        needRecordKey := DescRead("common", "needRecordKey", "1")
    }
    ; 是否记录按键

    global hotkey4Show,hotkey4Show := DescRead("common", "hotkey4Show", "^!+s")
    ; 按键显示开关快捷键
    ; global b_4show,b_4show := True ; 是否显示按键的默认开关，此变量和 needShowKey 合并

    global ctrlState,ctrlState := DescRead("common", "ctrlState", 1)
    ; 是否显示 控制键状态，如果出现则显示
    serverPortDect := 19999
    global serverPort,serverPort := DescRead("common", "serverPort", serverPortDect)
    ; 尝试判断是否文件无法读取正确内容
    if serverPort = serverPortDect {
        MsgBox('Can not read file: [' IniFile '], will exit!!')
        ExitApp
    }
    ; 是否显示 控制键状态，如果出现则显示

    global activeWindowProc,activeWindowProc := DescRead("common", "activeWindowProc", "")
    ; 按键显示仅仅针对活跃窗口，正则匹配

    global monitorProc,monitorProc := DescRead("common", "monitorProc", "")
    ; 监控的进程名,默认匹配所有进程

    global showHttpDebug,showHttpDebug := DescRead("common", "showHttpDebug", "0")
    ; 是否显示http调试框

    global maxKeypressCount,maxKeypressCount := DescRead("common", "maxKeypressCount", "10")
    ; 按键不放重复次数上限
    global maxCtrlpressCount,maxCtrlpressCount := DescRead("common", "maxCtrlpressCount", "1")
    ; 控制键按键不放重复次数上限
    global remoteType,remoteType := DescRead("common", "remoteType", 0)
    ; 远程控制模式，0 禁止远程 1 可远程查看，2 可远程查看和设置

    global hideInWinPwd,hideInWinPwd := DescRead("common", "hideInWinPwd", "0")
    ; 在window密码框中输入自动隐藏显示
    global statProcInfo,statProcInfo := DescRead("common", "statProcInfo", "1")
    ; 是否发送按键对应的进程信息，可用于统计不同进程下的按键数
    global defaultLang,defaultLang := DescRead("common", "defaultLang", "zh-CN")
    ; 默认语言

    global serverExecName,serverExecName := DescRead("common", "serverExecName", "node.exe")
    ; 后端服务执行程序文件名

    global preAppNameEnable,preAppNameEnable := DescRead("common", "preAppNameEnable", "0")
    ; 是否启用文件转换
    global preAppNameList,preAppNameList := DescRead("common", "preAppNameList", "{}")
    ; 文件转换规则
    if (preAppNameEnable = 1) {
        preAppNameList := CovertJosn(preAppNameList)
        global preAppNameListMap,preAppNameListMap := JSON.parse(preAppNameList)
    }

    global recordHistoryMax,recordHistoryMax := DescRead("common", "recordHistoryMax", "50")

    global shareDbPath,shareDbPath := DescRead("common", "shareDbPath", "")
    global shareDbName,shareDbName := DescRead("common", "shareDbName", "")
    global shareDbHour,shareDbHour := DescRead("common", "shareDbHour", "")
    global shareDbExec,shareDbExec := DescRead("common", "shareDbExec", "")

    ; 配置参数
    global guiWidth,guiWidth := DescRead("dialog", "guiWidth", 300) ; 宽度
    global guiHeigth,guiHeigth := DescRead("dialog", "guiHeigth", 0) ;高度 0 为自动高度
    global guiBgcolor,guiBgcolor := DescRead("dialog", "guiBgcolor", "11AA99") ; 背景色
    global guiBgTrans,guiBgTrans := DescRead("dialog", "guiBgTrans", 0) ; 背景完全透明
    global guiTrans,guiTrans := DescRead("dialog", "guiTrans", 1) ; 鼠标穿透
    global guiOpacity,guiOpacity := DescRead("dialog", "guiOpacity", 150) ; 0-255
    global guiTextFont,guiTextFont := DescRead("dialog", "guiTextFont", "Verdana") ; 字体
    global guiTextSize,guiTextSize := DescRead("dialog", "guiTextSize", 26) ; 字体大小
    global guiTextWeight,guiTextWeight := DescRead("dialog", "guiTextWeight", "bold") ; 是否粗体
    global guiTextColor,guiTextColor := DescRead("dialog", "guiTextColor", "FF0000") ; 文字颜色
    global guiLife,guiLife := DescRead("dialog", "guiLife", 7000) ; 显示多少毫秒消失
    global guiInterval,guiInterval := DescRead("dialog", "guiInterval", 1000) ; 窗体间隔毫秒
    global guiPos,guiPos := DescRead("dialog", "guiPos", "BR") ; TL,TR,BL,BR 上下左右位置
    global guiPosXY,guiPosXY := DescRead("dialog", "guiPosXY", "Y") ; 旧窗口移动位置 X / Y 默认为 Y
    global guiPosOffsetX,guiPosOffsetX := DescRead("dialog", "guiPosOffsetX", 0) ;位置X调整
    global guiPosOffsetY,guiPosOffsetY := DescRead("dialog", "guiPosOffsetY", -50) ;位置y调整
    global guiDpiscale,guiDpiscale := DescRead("dialog", "guiDpiscale", 0) ; 是否进行DPI 缩放
    global guiMonitorNum,guiMonitorNum := DescRead("dialog", "guiMonitorNum", 2) ; 第几个屏幕
    global guiMargin,guiMargin := DescRead("dialog", "guiMargin", 5) ; 多个窗口间的间隔
    global guiEdge,guiEdge := DescRead("dialog", "guiEdge", 1) ; 是否有细边框
    global txtSplit,txtSplit := DescRead("dialog", "txtSplit", " ") ; 按键的分隔符
    global ctrlX,ctrlX := DescRead("dialog", "ctrlX", 10) ; 控制键X位置
    global ctrlY,ctrlY := DescRead("dialog", "ctrlY", 10) ; 控制键Y位置
    global ctrlTextSize,ctrlTextSize := DescRead("dialog", "ctrlTextSize", 20) ; 字体大小
    global ctrlWidth,ctrlWidth := DescRead("dialog", "ctrlWidth", 240)
    global ctrlHeight,ctrlHeight := DescRead("dialog", "ctrlHeight", 0)
    global ctrlBgcolor,ctrlBgcolor := DescRead("dialog", "ctrlBgcolor", "11AA99")
    global ctrlOpacity,ctrlOpacity := DescRead("dialog", "ctrlOpacity", 150)
    global ctrlTextFont,ctrlTextFont := DescRead("dialog", "ctrlTextFont", "Verdana")
    global ctrlTextWeight,ctrlTextWeight := DescRead("dialog", "ctrlTextWeight", "bold")
    global ctrlTextColor,ctrlTextColor := DescRead("dialog", "ctrlTextColor", "FF0000")
    global guiRadius,guiRadius  := DescRead("dialog", "guiRadius", 0) ; 按键显示的圆角半径
    global ctrlRadius,ctrlRadius  := DescRead("dialog", "ctrlRadius", 0)  ; 控制键显示的圆角半径


    global ctrlList,ctrlList := StrSplit(DescRead("dialog", "ctrlList", "Ctrl|Alt|LWin|Shift|RWin|CapsLock"), "|") ; 哪些按键长按会单独显示出来
    global skipShow,skipShow := StrSplit(DescRead("dialog", "skipShow", ""), "|") ; 哪些按键不会显示，但会记录

    global activeAppShow,activeAppShow := DescRead("common", "activeAppShow", 0) ; 是否显示当前激活的窗体所在应用路径
    global activeAppShowX,activeAppShowX := DescRead("dialog", "activeAppShowX", 0) ; 应用路径显示的X轴位置
    global activeAppShowY,activeAppShowY := DescRead("dialog", "activeAppShowY", 0) ; 应用路径显示的Y轴位置

    global CheckServerMax,CheckServerMax := DescRead("common", "CheckServerMax", 9) ; 启动时最大重启尝试连接次数
    global maxCountOfConnectFail,maxCountOfConnectFail := DescRead("common", "maxCountOfConnectFail", 60) ; 连续通讯失败多少次后提示报错
    
    global guiFadeMs,guiFadeMs := DescRead("dialog", "guiFadeMs", 500) ; 窗口渐变消失ms数，0表示立刻消失

    global showKeyOnlyWeb,showKeyOnlyWeb := DescRead("common", "showKeyOnlyWeb", "0")
    ; 按键显示模式，1=只在Web上显示，桌面不显示，0=桌面和web均显示
    global mergeChar,mergeChar := DescRead("dialog", "mergeChar", "1")
    ; 相同字母是否合并，默认为1
}
ReadAllIni()
GetKeyList()
; 部分变量设置
serverName := "127.0.0.1"
serverUrl := "http://" serverName ":" serverPort
serverUrlWs := "ws://" serverName ":" serverPort
; 内部参数
global handleWS := 0 ; websocket 句柄
global guiArr := Array() ; 保存guiObj 对象
global inShowArr := Array() ; 保存传入进来的数组用于显示
global guiShowing := 0

global minLeft := 0
global minTop := 0
global maxRight := 0
global maxBottom := 0

global ctrlKeyCount := 0
global repeatRecord := 0

global countOfConnectFail := 0
; 获取1970年开始的时间戳
AllKeyRecord['tick'] := DateDiff(A_NowUTC, '19700101', 'Seconds') * 1000 + A_MSec ; tick数据不一样表示程序重启过，需要累计计数

; events 中变量控制
reqXMLHTTP := 0
; lastModified := FileGetTime(IniFile)
global HttpCtrlObj := Map()  ; 和http任务相关的数据
HttpCtrlObj['resp'] := '' ; 返回的数据
HttpCtrlObj['task'] := '' ; 任务名
HttpCtrlObj['state'] := '' ; 当前状态 wait succ error
serverState := -1  ; 是否连接到服务器， -1 还未启动过，0连失败，1 成功连接
CheckServerCount := 0
; 鼠标开始的位置和距离
mouseStartX := 0
mouseStartY := 0
mouseDistance := 0
; 全局键盘鼠标统计
globalKeyCount := 0
globalMouseCount := 0
GetMinuteDataFlag := False  ; 标记正在处理分钟数据
; 主显示器编号
LastScreenNum := MonitorGetPrimary()

global recordHistory := Array() ; 保存按键历史数据
global lastSendDataTime := 0

SystemLocked(){
    if (ProcessExist("LogonUI.exe"))
	{
		return true   ; 系统锁定
	}else{
        return false
    }
}
; 最近一次按键文字数组
lastTextArr := []  ; 		lastTextArr := textArr
; 最近一次窗口时间
lastTextTick := 0 ; 用于控制是否显示新窗口

