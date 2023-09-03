Persistent  ; 持久运行脚本
; 全局通用变量和函数
global APPName:="ShowKeyBoard", ver:="1.1" 
     , IniFile := "showKeyBoard.ini"
     , AllKeyRecord := Map()
     
; 默认需要忽略的按键清单 "{LCtrl}{RCtrl}{LAlt}{RAlt}{LShift}{RShift}{LWin}{RWin}"
; 这些按键用独立的监控来发送
skipKeys := "{LCtrl}{RCtrl}{LShift}{RShift}{LWin}{RWin}{LAlt}{RAlt}"
; 基础配置文件读取

skipRecord := StrSplit(IniRead(IniFile,"common","skipRecord",""),'|')
; 哪些按键要忽略记录
skipCtrlKey := IniRead(IniFile,"common","skipCtrlKey","0")
; 是否忽略单独的控制键，不记录
onlyRecordKey := IniRead(IniFile,"common","onlyRecordKey","0")
; 只记录按键，不显示
ctrlState :=IniRead(IniFile,"common","ctrlState",1 ) 
; 是否显示 控制键状态，如果出现则显示
serverPort :=IniRead(IniFile,"common","serverPort",9900 ) 
; 是否显示 控制键状态，如果出现则显示
serverUrl := "http://127.0.0.1:" serverPort
