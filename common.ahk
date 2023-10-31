Persistent  ; 持久运行脚本
; 全局通用变量和函数
global IniFile := "showKeyBoard.ini"
     , AllKeyRecord := Map()
A_MaxHotkeysPerInterval := 240  ; 应对快速的宏操作
; 默认需要忽略的按键清单 "{LCtrl}{RCtrl}{LAlt}{RAlt}{LShift}{RShift}{LWin}{RWin}"
; 这些按键用独立的监控来发送
skipKeys := "{LCtrl}{RCtrl}{LShift}{RShift}{LWin}{RWin}{LAlt}{RAlt}"
; 基础配置文件读取
; 自定义函数，优先从目标的 showKeyBoard.desc.ini 文件中读取默认参数，如果没有才使用第三个默认参数
global DescIniPath := '',httpDistPath := '',httpPath := '' , needRecordKey := -1
DescRead(sect,key,defaultVal){
    global DescIniPath
    if DescIniPath = '' {
        global httpDistPath :=IniRead(IniFile,"common","httpDistPath", "\httpdist\dist\" )
        global httpPath := A_WorkingDir httpDistPath   ; node 脚本所在目录
        if !FileExist(httpPath){
            global needRecordKey := 0  ; 如果不存在则不用启动后端,后续也无需读取参数
        }
        tmpDescIni := A_WorkingDir httpDistPath 'showKeyBoard.desc.ini'
        if !FileExist(tmpDescIni){
            DescIniPath := 'NotFound'
        }else{
            DescIniPath := tmpDescIni
        }
    }
    if DescIniPath != 'NotFound' {
    ; 存在默认配置文件，可以开始读取数据
        defaultVal := IniRead(DescIniPath,sect,key,defaultVal)
    }
    return IniRead(IniFile,sect,key,defaultVal)
}
skipRecord := StrSplit(DescRead("common","skipRecord",""),'|')
; 哪些按键要忽略记录
skipCtrlKey := DescRead("common","skipCtrlKey","0")
; 是否忽略单独的控制键，不记录
showMouseEvent := DescRead("common","showMouseEvent","1")
; 是否显示和记录鼠标事件
recordMouseMove := DescRead("common","recordMouseMove","0")
; 是否记录鼠标移动距离
needShowKey := DescRead("common","needShowKey","1")
preNeedShowKey := needShowKey
; 是否显示按键

if needRecordKey = -1 {
    needRecordKey := DescRead("common","needRecordKey","1")
}
; 是否记录按键

hotkey4Show := DescRead("common","hotkey4Show","^!+s")
; 按键显示开关快捷键
; b_4show := True ; 是否显示按键的默认开关，此变量和 needShowKey 合并

ctrlState :=DescRead("common","ctrlState",1 ) 
; 是否显示 控制键状态，如果出现则显示
serverPort :=DescRead("common","serverPort",9900 ) 
; 是否显示 控制键状态，如果出现则显示

activeWindowProc :=DescRead("common","activeWindowProc","" ) 
; 按键显示仅仅针对活跃窗口，正则匹配

showHttpDebug :=DescRead("common","showHttpDebug","0" ) 
; 是否显示http调试框

maxKeypressCount :=DescRead("common","maxKeypressCount","10" ) 
; 按键不放重复次数上限
maxCtrlpressCount :=DescRead("common","maxCtrlpressCount","1" ) 
; 控制键按键不放重复次数上限
remoteType :=DescRead("common","remoteType",0 ) 
; 远程控制模式，0 禁止远程 1 可远程查看，2 可远程查看和设置

hideInWinPwd :=DescRead("common","hideInWinPwd", "0" ) 
; 在window密码框中输入自动隐藏显示
statProcInfo :=DescRead("common","statProcInfo", "1" )
; 是否发送按键对应的进程信息，可用于统计不同进程下的按键数
; 配置参数
guiWidth :=DescRead("dialog","guiWidth", 300  ) ; 宽度
guiHeigth :=DescRead("dialog","guiHeigth", 0 ) ;高度 0 为自动高度
guiBgcolor :=DescRead("dialog","guiBgcolor", "11AA99" ) ; 背景色
guiBgTrans :=DescRead("dialog","guiBgTrans", 0 ) ; 背景完全透明
guiTrans :=DescRead("dialog","guiTrans", 1 ) ; 鼠标穿透
guiOpacity :=DescRead("dialog","guiOpacity", 150 ) ; 0-255
guiTextFont := DescRead("dialog","guiTextFont", "Verdana" ) ; 字体
guiTextSize := DescRead("dialog","guiTextSize", 26 ) ; 字体大小
guiTextWeight :=DescRead("dialog","guiTextWeight", "bold" ) ; 是否粗体
guiTextColor :=DescRead("dialog","guiTextColor", "FF0000" ) ; 是否粗体
guiLife :=DescRead("dialog","guiLife", 7000 ) ; 显示多少毫秒消失
guiInterval :=DescRead("dialog","guiInterval", 1000 ) ; 窗体间隔毫秒
guiPos :=DescRead("dialog","guiPos", "BR" ) ; TL,TR,BL,BR 上下左右位置
guiPosXY :=DescRead("dialog","guiPosXY", "Y" ) ; 旧窗口移动位置 X / Y 默认为 Y
guiPosOffsetX :=DescRead("dialog","guiPosOffsetX",0 ) ;位置X调整
guiPosOffsetY :=DescRead("dialog","guiPosOffsetY", -50 ) ;位置y调整
guiDpiscale :=DescRead("dialog","guiDpiscale",0 ) ; 是否进行DPI 缩放
guiMonitorNum :=DescRead("dialog","guiMonitorNum", 2 ) ; 第几个屏幕
guiMargin :=DescRead("dialog","guiMargin", 5    ) ; 多个窗口间的间隔
guiEdge :=DescRead("dialog","guiEdge",1 ) ; 是否有细边框
txtSplit :=DescRead("dialog","txtSplit"," " ) ; 按键的分隔符
ctrlX :=DescRead("dialog","ctrlX",10 ) ; 控制键X位置
ctrlY :=DescRead("dialog","ctrlY",10 ) ; 控制键Y位置
ctrlTextSize :=DescRead("dialog","ctrlTextSize", 20 ) ; 字体大小
ctrlList := StrSplit(DescRead("dialog","ctrlList", "Ctrl|Alt|LWin|Shift|RWin|CapsLock"),"|") ; 哪些按键长按会单独显示出来
skipShow := StrSplit(DescRead("dialog","skipShow", ""),"|") ; 哪些按键不会显示，但会记录

; 部分变量设置
serverUrl := "http://127.0.0.1:" serverPort
; 内部参数
global guiArr := Array() ; 保存guiObj 对象
global inArr := Array() ; 保存传入进来的数组
global guiShowing := 0
global KeyMapping:=Map()

global minLeft := 0
global minTop := 0
global maxRight := 0
global maxBottom := 0

global ctrlKeyCount := 0
global repeatRecord := 0

; 获取1970年开始的时间戳
AllKeyRecord['tick'] := DateDiff(A_NowUTC, '19700101', 'Seconds')*1000 + A_MSec ; tick数据不一样表示程序重启过，需要累计计数

; events 中变量控制
reqXMLHTTP := 0 
msgNotLaunch := 'Start Server fail, can not set parameter and data statistics!'
lastModified := FileGetTime(IniFile)
global HttpCtrlObj := Map()  ; 和http任务相关的数据
HttpCtrlObj['resp'] := '' ; 返回的数据
HttpCtrlObj['task'] := '' ; 任务名
HttpCtrlObj['state'] := '' ; 当前状态 wait succ error
serverState := -1  ; 是否连接到服务器， -1 还未启动过，0连失败，1 成功连接 
CheckServerCount :=0
CheckServerMax := 8