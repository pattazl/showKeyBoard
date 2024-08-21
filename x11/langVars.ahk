; 全局变量语言包

global menu_msg_noserver := '服务未成功启动，不能用界面查看和设置'
global L_menu_startup:="开机启动"
global L_menu_reload:="重启程序"
global L_menu_reset:="完全重启"
global L_menu_pause:="暂停运行"
global L_menu_4show:="按键显示(" hotkey4Show ")"
global L_menu_set:="参数设置"
global L_menu_stat:="数据统计"
global L_menu_exit:="退出程序"
global msgNotLaunch := '服务启动失败，不能通过界面设置参数和查看统计信息!`n可尝试用管理员方式重启应用或重启系统'
global msgXML := '不能创建XMLHTTP,'
global msgTry := '尝试'
global msgTimes := '次,'
global msgLaunch := '无法启动Node，请确保已经安装node环境或使用非node的安装包'
global msgLaunchSucc := '启动服务成功!'
global msgConnectFail:= '后台服务无法连接,请检查或重启应用!'
if defaultLang = 'en-US' {
    L_menu_startup:="Auto Start"
    L_menu_reload :="Reset Client"
    L_menu_reset  :="Full Reset"
    L_menu_pause  :="Pause"
    L_menu_4show  :="Show/Hide Key(" hotkey4Show ")"
    L_menu_set    :="Setting"
    L_menu_stat   :="Statistics"
    L_menu_exit   :="Exit"

    menu_msg_noserver := 'Launch Server Fail! Can not Statistics and set parameters by UI'
    
    msgNotLaunch := 'Start Server fail, can not set parameter and data statistics, please relaunch app or restart system!'
    msgXML := 'Can not create XMLHTTP,'
    msgTry := 'Try '
    msgTimes := ' times ,'
    msgLaunch := 'Launch node fail, make sure node be installed or use another setup!'
    msgLaunchSucc := 'Start Server succ!'
    msgConnectFail:= 'can not connect server, please relaunch app!'
}