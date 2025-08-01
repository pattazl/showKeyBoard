; 用于进行对外通讯，只读本地文件 和发送数据
#Include "lib/WebSocket.ahk"
; 如果无需记录，那么将关闭界面设置功能
if(needRecordKey=1){
	Init()  ; 判断后端接口，启动相关程序
	; 启动可能需要花费一些时间,CheckServer进行重复判断
	CheckServer()
}
; 设置检测间隔，单位为毫秒
; SetTimer IniMonitor,2000 ; websoket 代替
; 此函数改为重新读取参数配置文件参数
IniMonitor(reloadAll){
    ; global lastModified
    if not FileExist(IniFile){
        return
    }
    ; modified := FileGetTime(IniFile)
    ; if (modified != lastModified)
    ; 如果端口发生了变化则需要完全重启
    ; OutputDebug('AHK IniMonitor') 
    newPort :=IniRead(IniFile,"common","serverPort",9900 )
    newRemoteType :=IniRead(IniFile,"common","remoteType",1 )
    newShowHttpDebug :=IniRead(IniFile,"common","showHttpDebug",0 )
    if newPort != serverPort || newRemoteType != remoteType || newShowHttpDebug != showHttpDebug
    {
        ExitServer()
        if serverState = 1 {
            Sleep(200)  ; 如果后台网络已经启动，退出最好等待一会儿
        }
        ;  当文件发生变化后，需要重新载入
        ; OutputDebug('AHK Reload') 
        Reload() 
        return
    }
    ; 参数变化直接修改即可 调用 GetKeyList
    if(reloadAll == 1)
    {
       ; OutputDebug('AHK reloadAll') 
       ReadAllIni()
       GetKeyList()
       ; 可重新载入 ctrl 和 app
       DestoryCtrlState()
       CreateCtrlState()
       ; 可清空当前 tooltip
       ToolTip("")
    }
}

; 网络相关函数代码
Init(){
    try{
        global reqXMLHTTP := ComObject("Msxml2.XMLHTTP")  ; Msxml2.XMLHTTP Microsoft.XMLHTTP
    }
    catch
    {
        MsgBox msgXML msgNotLaunch
    }
}
; 发送数据
StartHttp(task,route,data,timeout:=8000)
{
    if reqXMLHTTP = 0 
        return
    ; 如果正在运行则退出，同一时间只会有一个
    if HttpCtrlObj['state'] = 'wait'
    {
        ; OutputDebug ("AutoHotkey :skip " task)
        return 0
    }
    HttpCtrlObj['task'] := task  ; 任务名
    HttpCtrlObj['tick'] := A_TickCount  ; 任务tick
    HttpCtrlObj['timeout'] := timeout  ; 任务超时,一般不用设置
    sendData(route,data)
    ; 检测数据变化情况
    SetTimer ServerCore, 10
    return 1
}
; 是否曾经启动或Cmd
CheckServer(){
    IniMonitor(0) ; 读取配置文件看是否有所变化
    ; 如果不能连接上则需要启动Node服务
	global CheckServerCount
	if CheckServerCount < CheckServerMax
	{
		ShowTxt(msgTry (CheckServerCount+1) msgTimes )
		StartHttp('connect','/version','')
	}else{
		MsgBox msgTry CheckServerCount msgTimes msgNotLaunch
	}
	CheckServerCount +=1
	Sleep 3000
}
; 绑定websocket回调事件
BindWebSocket() {
    global handleWS := WebSocket(serverUrlWs, {
        message: (self, data) => (
            IniMonitor(1) ; ,OutputDebug('AHK BindWebSocket') 
        ),  ; OutputDebug('AHK' Data '`n' '*' 'utf-8'),
        close: (self, status, reason) => '' ; OutputDebug('AHK' status ' ' reason '`n' '*' 'utf-8')
    })
    handleWS.sendText('ahkClient')
}
; 服务核心处理
ServerCore()
{
    state := HttpCtrlObj['state'] ; 此值会随时变化，先记录下
    if( (A_TickCount - HttpCtrlObj['tick']) > HttpCtrlObj['timeout'])
    {
        ; 超时，需要取消定时
        SetTimer ServerCore, 0
        ; 此时可以尝试启动服务
        ShowTxt 'Task[' HttpCtrlObj['task'] ']timeout:' HttpCtrlObj['timeout'] 
        return
    }
    if(state ='wait')
    {
        return
    }
    ; 连接完成，取消定时
    SetTimer ServerCore, 0  
    if HttpCtrlObj['task']='connect' {
        if state ='succ' && InStr(HttpCtrlObj['resp'],'showKeyBoardServer Version:') > 0
        {
            ; 成功启动后端服务
			global serverState := 1
            ShowTxt msgLaunchSucc
			; 准备发送一些数据给后端
			SendPCInfo(0)
            ; 绑定Websocket事件
            BindWebSocket()
        }else{
			 ; 需要重试，超过N次后判断失败
			 if serverState = -1{
				startServer() ; 启动服务
			 }
			 CheckServer()
        }
    }else{
        ; 常规数据发送处理
        global countOfConnectFail
        if state !='succ'{
            ShowTxt HttpCtrlObj['task'] ',connect fail!'   ; 可能后端服务被关闭了
            OutputDebug 'connect fail'
            countOfConnectFail += 1
            if(countOfConnectFail >= maxCountOfConnectFail){
                MsgBox msgTry countOfConnectFail msgTimes msgConnectFail
                countOfConnectFail := 0 ; 成功一次则复位
            }
        }Else{
            countOfConnectFail := 0 ; 成功一次则复位
        }
    }
}
; 根据进程id获取命令行
GetCommand(procId)
{
    cmdLine := ""
    try{
        wmi := ComObjGet("winmgmts:")
        queryEnum := wmi.ExecQuery(""
            . "Select * from Win32_Process where ProcessId=" . procId)
            ._NewEnum()
        ; Get first matching process.
        if queryEnum(&proc)
            cmdLine := proc.CommandLine
    }
    return cmdLine
}
; 启动服务
startServer()
{
    cmdExe := serverExecName ' '  ; Node命令带空格支持后面的参数
    cmd := cmdExe '"' httpPath 'server.js"'  ; 完整命令行
    global serverState :=0 ; 假设启动失败
    ; 读取 httpPath+ kbserver.pid 进程文件检查是否是node的
    ;  需要启动服务,要防止路径上有空格
    pidFile := httpPath 'kbserver.pid'
    if FileExist(pidFile){
        strPid := FileRead(pidFile, "`n UTF-8")
        iPid := ProcessExist(strPid)
        if iPid>0{
            try {
                path := ProcessGetPath(iPid)
                SplitPath Path ,&OutFileName, &OutDir
                if OutFileName = trim(cmdExe) {
                    cmdLine := GetCommand(iPid)  ; 尝试获取命令行根据命令行参数排挤
                    if cmdLine !="" {
                        canKill :=  (cmdLine = cmd)
                    }else{ ; 没有成功获取命令行则直接用node的路径判断
                        canKill := ((OutDir '\') = httpPath)
                    }
                    if canKill {
                        ProcessClose(iPid) ;如果是 自己的进程则杀进程
                    }
                }
            }
            FileDelete pidFile  ; 删除进程文件
        }
    }
	try {
	; Run cmd ,,'Hide' Show
		ShowFlag := 'Hide'
		if showHttpDebug {
			ShowFlag := 'Show'
		}
		Run cmd,httpPath,ShowFlag, &OutputVarPID
		Sleep 1000  ;启动服务需要等待
	}catch{
		MsgBox msgLaunch "`n" cmd
	}
}
; 自动发送 AllKeyRecord 数据
AutoSendData()
{
    if serverState = 1 {
        StartHttp('data','/data',AllKeyRecord,timeout:=8000)
    }
}

; 发送退出数据
ExitServer()
{
    if serverState = 1 {
        StartHttp('exit','/exit','')
    }
}
; 发送数据给后端服务,flag=0 表示第一次发送数据
SendPCInfo(flag)
{
	; 获取不同屏幕
	MCount := MonitorGetCount()
	JSONStr := '{"screen":['
	loop MCount {
		if A_Index != 1{
			JSONStr := JSONStr ','
		}
        try {
            MonitorGet(A_Index, &Left, &Top, &Right, &Bottom)
            global minLeft := Min(minLeft,Left)
            global minTop := Min(minTop,Top)
            global maxRight := Max(maxRight,Right)
            global maxBottom := Max(maxBottom,Bottom)
            tmp := '{"Left":' Left ',"Top":' Top ',"Right":' Right ',"Bottom":' Bottom '}'
            JSONStr := JSONStr tmp
        }catch{
            ; 出现异常无需发送数据
            return 
        }
	}
	JSONStr := JSONStr '],"flag":' flag ' , "majorVersion":"' ver '"}'
	if serverState = 1 {
        StartHttp('sendPCInfo','/sendPCInfo',JSONStr,timeout:=8000)
    }
}
; 数据回调和核心发送
preReadyState := 0  ; http前一个状态
Ready() {
    if reqXMLHTTP = 0 {
        return
    }
    ; OutputDebug 'readyState : ' reqXMLHTTP.readyState
    if (reqXMLHTTP.readyState !=4 )  ; Not done yet.
    {
        ; HttpCtrlObj['state'] := 'wait'
        global preReadyState := reqXMLHTTP.readyState  ; 获取前一个状态
        return
    }
    ; OutputDebug 'state : ' reqXMLHTTP.status ' preReadyState:' preReadyState
    if (reqXMLHTTP.status == 200 && preReadyState == 3)  ; 不知道为何 status == 200 不能判断，所以需要增加一个前一个状态为3的判断
    {    ; OK.
        HttpCtrlObj['state'] := 'succ'
        ; 可能出现 Error: (0x8000000A) 完成该操作所需的数据还不可使用。屏蔽掉
        try{
            HttpCtrlObj['resp'] := reqXMLHTTP.responseText
        }
    }
    else{
        HttpCtrlObj['state'] := 'error'
    }
    ;MsgBox "Status " reqXMLHTTP.status,, 16
}

sendData(route,data:=''){
    HttpCtrlObj['state'] := 'wait'
	reqXMLHTTP.open('POST', serverUrl route , true)
	reqXMLHTTP.setRequestHeader("Content-Type", "application/json")
	; reqXMLHTTP.setRequestHeader("Cache-Control", "no-cache")
	; Set our callback function.application/json  text/plain"
	reqXMLHTTP.onreadystatechange := Ready
	; Send the reqXMLHTTPuest.  Ready() will be called when it's complete.
    ;data["time"] := A_Now
    str := ''
    if(data!=''){
		if( Type(data) = 'Map'){
			str := JSON.stringify(data,0) ; 无需换行或缩进
		}else{
			str := data   ; 普通字符串
		}
    }
    if(showHttpDebug){
      ; OutputDebug('POST ' str)
    }
	reqXMLHTTP.send( str )
}