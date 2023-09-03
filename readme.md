实现如下功能

1. 按键统计，可自定义键盘，导出统计数据
2. 按键显示，类似 Carnac 功能



技术架构

按键监控采用autohotkey + windows API
界面显示用html , node做websocket服务通讯

客户端程序只管读取配置文件和发送数据，不写配置文件

HTTP端负责写配置文件接收数据

HTTP的服务端路由
/setting
管理设置界面

/stat
管理数据统计界面

/exit
退出服务

/version
服务端相关版本，也可以用于检测是否存在 'showKeyBoardServer Version:'
