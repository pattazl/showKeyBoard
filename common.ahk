Persistent  ; 持久运行脚本
; 全局通用变量和函数
global APPName:="ShowKeyBoard", ver:="1.7" 
     , IniFile := "showKeyBoard.ini"
     , AllKeyRecord := Map()
A_MaxHotkeysPerInterval := 240  ; 应对快速的宏操作
; 默认需要忽略的按键清单 "{LCtrl}{RCtrl}{LAlt}{RAlt}{LShift}{RShift}{LWin}{RWin}"
; 这些按键用独立的监控来发送
skipKeys := "{LCtrl}{RCtrl}{LShift}{RShift}{LWin}{RWin}{LAlt}{RAlt}"
; 基础配置文件读取

skipRecord := StrSplit(IniRead(IniFile,"common","skipRecord",""),'|')
; 哪些按键要忽略记录
skipCtrlKey := IniRead(IniFile,"common","skipCtrlKey","0")
; 是否忽略单独的控制键，不记录
showMouseEvent := IniRead(IniFile,"common","showMouseEvent","1")
; 是否显示和记录鼠标事件
recordMouseMove := IniRead(IniFile,"common","recordMouseMove","0")
; 是否记录鼠标移动距离
needShowKey := IniRead(IniFile,"common","needShowKey","1")
; 是否显示按键
needRecordKey := IniRead(IniFile,"common","needRecordKey","1")
; 是否记录按键
hotkey4Show := IniRead(IniFile,"common","hotkey4Show","^!+s")
; 按键显示开关快捷键
b_4show := True ; 是否显示按键的默认开关

ctrlState :=IniRead(IniFile,"common","ctrlState",1 ) 
; 是否显示 控制键状态，如果出现则显示
serverPort :=IniRead(IniFile,"common","serverPort",9900 ) 
; 是否显示 控制键状态，如果出现则显示

activeWindowProc :=IniRead(IniFile,"common","activeWindowProc","" ) 
; 按键显示仅仅针对活跃窗口，正则匹配

showHttpDebug :=IniRead(IniFile,"common","showHttpDebug","0" ) 
; 是否显示http调试框

; 配置参数
guiWidth :=IniRead(IniFile,"dialog","guiWidth", 300  ) ; 宽度
guiHeigth :=IniRead(IniFile,"dialog","guiHeigth", 0 ) ;高度 0 为自动高度
guiBgcolor :=IniRead(IniFile,"dialog","guiBgcolor", "11AA99" ) ; 背景色
guiBgTrans :=IniRead(IniFile,"dialog","guiBgTrans", 0 ) ; 背景完全透明
guiTrans :=IniRead(IniFile,"dialog","guiTrans", 1 ) ; 鼠标穿透
guiOpacity :=IniRead(IniFile,"dialog","guiOpacity", 150 ) ; 0-255
guiTextFont := IniRead(IniFile,"dialog","guiTextFont", "Verdana" ) ; 字体
guiTextSize := IniRead(IniFile,"dialog","guiTextSize", 26 ) ; 字体大小
guiTextWeight :=IniRead(IniFile,"dialog","guiTextWeight", "bold" ) ; 是否粗体
guiTextColor :=IniRead(IniFile,"dialog","guiTextColor", "FF0000" ) ; 是否粗体
guiLife :=IniRead(IniFile,"dialog","guiLife", 7000 ) ; 显示多少毫秒消失
guiInterval :=IniRead(IniFile,"dialog","guiInterval", 1000 ) ; 窗体间隔毫秒
guiPos :=IniRead(IniFile,"dialog","guiPos", "BR" ) ; TL,TR,BL,BR 上下左右位置
guiPosXY :=IniRead(IniFile,"dialog","guiPosXY", "Y" ) ; 旧窗口移动位置 X / Y 默认为 Y
guiPosOffsetX :=IniRead(IniFile,"dialog","guiPosOffsetX",0 ) ;位置X调整
guiPosOffsetY :=IniRead(IniFile,"dialog","guiPosOffsetY", -50 ) ;位置y调整
guiDpiscale :=IniRead(IniFile,"dialog","guiDpiscale",0 ) ; 是否进行DPI 缩放
guiMonitorNum :=IniRead(IniFile,"dialog","guiMonitorNum", 2 ) ; 第几个屏幕
guiMargin :=IniRead(IniFile,"dialog","guiMargin", 5    ) ; 多个窗口间的间隔
guiEdge :=IniRead(IniFile,"dialog","guiEdge",1 ) ; 是否有细边框
txtSplit :=IniRead(IniFile,"dialog","txtSplit"," " ) ; 按键的分隔符
ctrlX :=IniRead(IniFile,"dialog","ctrlX",10 ) ; 控制键X位置
ctrlY :=IniRead(IniFile,"dialog","ctrlY",10 ) ; 控制键Y位置
ctrlTextSize :=IniRead(IniFile,"dialog","ctrlTextSize", 20 ) ; 字体大小
ctrlList := StrSplit(IniRead(IniFile,"dialog","ctrlList", "Ctrl|Alt|LWin|Shift|RWin|CapsLock"),"|") ; 哪些按键长按会单独显示出来
skipShow := StrSplit(IniRead(IniFile,"dialog","skipShow", ""),"|") ; 哪些按键不会显示，但会记录

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

; 获取1970年开始的时间戳
AllKeyRecord['tick'] := DateDiff(A_NowUTC, '19700101', 'Seconds')*1000 + A_MSec ; tick数据不一样表示程序重启过，需要累计计数
global httpPath := A_WorkingDir '\httpdist\dist\'   ; node 脚本所在目录
if !FileExist(httpPath){
	needRecordKey := 0  ; 如果不存在则不用启动后端
}
