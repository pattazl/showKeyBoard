## 功能简介

1. 按键统计，可自定义键盘，查看统计数据

2. 按键显示，实时显示按键状态，类似 Carnac 功能

## 截图演示

   ![实时显示按键状态和内容](screenshot/%E5%AE%9E%E6%97%B6%E6%98%BE%E7%A4%BA%E6%8C%89%E9%94%AE%E7%8A%B6%E6%80%81%E5%92%8C%E5%86%85%E5%AE%B9.gif)

![历史按键趋势](screenshot/%E5%8E%86%E5%8F%B2%E6%8C%89%E9%94%AE%E8%B6%8B%E5%8A%BF.png)



![实时按键热力图](screenshot/%E5%AE%9E%E6%97%B6%E6%8C%89%E9%94%AE%E7%83%AD%E5%8A%9B%E5%9B%BE.jpg)

![自定义键盘](screenshot/%E8%87%AA%E5%AE%9A%E4%B9%89%E9%94%AE%E7%9B%98.png)

## 技术架构

按键监控客户端采用autohotkey + windows API

界面显示用html , node做websocket 和http 通讯

客户端程序只管读取配置文件和发送数据，不写配置文件

HTTP端负责写配置文件，接收html前端数据

## 编译说明

1. http里面的是服务器端代码，使用 npm run build 打包，生成 httpdist目录

2. ui-helper 是客户端配置页面 npm run build 生成 dist目录，发布时候要放到 httpdist/dist/ui目录中

3. 根目录是 autohotkey脚本，需要用autohotkey工具对 showkeyboard.ahk打包为 exe ,对应的 exe文件需要放到和httpdist同级目录

## 目录说明

1. 根目录下的 KeyList.txt 键盘映射关系，showKeyBoard.ini 是客户端配置信息
2. 目录 httpdist/dist/records.db 文件保存按键的统计信息和统计的相关配置信息
3. 文件 httpdist/dist/node.exe 用于启动后端http/websocket服务

