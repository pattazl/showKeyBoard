; 用于进行对外通讯，只读本地文件 和发送数据
#include common.ahk
#Include "lib/JSON.ahk"
; 如果无需记录，那么将关闭界面设置功能
if(needRecordKey=1){
	Init()  ; 判断后端接口，启动相关程序
	; 启动可能需要花费一些时间,CheckServer进行重复判断
	CheckServer()
}
; 设置检测间隔，单位为毫秒
SetTimer IniMonitor,2000
IniMonitor(){
    global lastModified
    if not FileExist(IniFile){
        return
    }
    modified := FileGetTime(IniFile)
    if (modified != lastModified)
    {
		; 如果端口发生了变化则需要完全重启
		newPort :=IniRead(IniFile,"common","serverPort",9900 )
		newRemoteType :=IniRead(IniFile,"common","remoteType",1 )
		newShowHttpDebug :=IniRead(IniFile,"common","showHttpDebug",0 )
		if newPort != serverPort || newRemoteType != remoteType || newShowHttpDebug != showHttpDebug
		{
			ExitServer()
            Sleep(200)  ; 最好等待一会儿
		}
        ;  当文件发生变化后，需要重新载入
        Reload()
    }
}

; 网络相关函数代码
Init(){
    try{
        global reqXMLHTTP := ComObject("Msxml2.XMLHTTP")  ; Msxml2.XMLHTTP Microsoft.XMLHTTP
    }
    catch
    {
        MsgBox 'Can not create XMLHTTP ,' msgNotLaunch
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
    ; 如果不能连接上则需要启动Node服务
	global CheckServerCount
	if CheckServerCount < CheckServerMax
	{
		ShowTxt('Try ' (CheckServerCount+1) ' times,')
		StartHttp('connect','/version','')
	}else{
		MsgBox 'Try ' CheckServerCount ' times, but fail: ' msgNotLaunch
	}
	CheckServerCount +=1
	Sleep 3000
	
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
            ShowTxt 'Start Server succ!'
			; 准备发送一些数据给后端
			SendPCInfo(0)
        }else{
			 ; 需要重试，超过N次后判断失败
			 if serverState = -1{
				startServer() ; 启动服务
			 }
			 CheckServer()
        }
    }else{
        ; 常规数据发送处理
        if state !='succ'{
            ShowTxt HttpCtrlObj['task'] ',connect fail!'   ; 可能后端服务被关闭了
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
    cmdExe := 'node.exe '  ; Node命令带空格支持后面的参数
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
                        ProcessClose(iPid) ;如果是 node.exe则杀进程
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
		MsgBox 'Launch node fail'
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
            tmp := '{"Left":' Left ',"Top":' Top ',"Right":' Right ',"Bottom":' Bottom ',"flag":' flag '}'
            JSONStr := JSONStr tmp
        }catch{
            ; 出现异常无需发送数据
            return 
        }
	}
	JSONStr := JSONStr ']}'
	if serverState = 1 {
        StartHttp('sendPCInfo','/sendPCInfo',JSONStr,timeout:=8000)
    }
}
; 数据回调和核心发送
Ready() {
    if reqXMLHTTP = 0 
        return
    if (reqXMLHTTP.readyState != 4)  ; Not done yet.
    {
        ; HttpCtrlObj['state'] := 'wait'
        return
    }
    if (reqXMLHTTP.status == 200)
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
	; Set our callback function.application/json  text/plain"
	reqXMLHTTP.onreadystatechange := Ready
	; Send the reqXMLHTTPuest.  Ready() will be called when it's complete.
    ;data["time"] := A_Now
    str := ''
    if(data!=''){
		if( Type(data) = 'Map'){
			str := JSON.stringify(data)
		}else{
			str := data   ; 普通字符串
		}
    }
	reqXMLHTTP.send( str )
}