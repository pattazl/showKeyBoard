实现如下功能

1. 按键统计，可自定义键盘，导出统计数据

2. 按键显示，类似 Carnac 功能

   ![实时显示按键状态和内容](screenshot/%E5%AE%9E%E6%97%B6%E6%98%BE%E7%A4%BA%E6%8C%89%E9%94%AE%E7%8A%B6%E6%80%81%E5%92%8C%E5%86%85%E5%AE%B9.gif)

![历史按键趋势](screenshot/%E5%8E%86%E5%8F%B2%E6%8C%89%E9%94%AE%E8%B6%8B%E5%8A%BF.png)



![实时按键热力图](screenshot/%E5%AE%9E%E6%97%B6%E6%8C%89%E9%94%AE%E7%83%AD%E5%8A%9B%E5%9B%BE.jpg)

![自定义键盘](screenshot/%E8%87%AA%E5%AE%9A%E4%B9%89%E9%94%AE%E7%9B%98.png)

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
