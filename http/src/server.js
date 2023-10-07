const WebSocket = require('ws');
const http = require('http');
const express = require('express')
const {getRecords,getHistoryDate,statData} = require('./records');
const version = '1.1'


const { startUp , getParaFun, setParaFun , app ,exitFun ,dataFun,sendPCInfo,saveLastData,optKeymapFun,deleteDataFun} = require('./common');

//app.use(express.text());
// 定义跨域设置中间件
app.use((req, res, next) => {
    res.setHeader('Access-Control-Allow-Origin', '*'); // 允许所有来源的请求
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE'); // 允许的请求方法
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization'); // 允许的请求头
    next();
  });
app.use(express.json());
app.use(express.static('ui'));
app.use('/Setting', express.static('ui'));
app.use('/Today', express.static('ui'));
app.use('/History', express.static('ui'));
app.use('/Statistics', express.static('ui'));
app.use('/Export', express.static('ui'));
// 返回全部参数信息 
app.post('/getPara', getParaFun);
// 写入修改全部参数信息
app.post('/setPara', setParaFun);
// app.get('/', (req, res) => {res.send('Welcome to showKeyBoard backend service');});
app.post('/exit',exitFun); // 退出系统
app.post('/data', dataFun);  // 上传按键数据
app.post('/sendPCInfo', sendPCInfo);  // 上传PC其他信息，比如显示屏和分辨率信息等
// 获取某某天，或某个tick的数据, 参数 date 如果是当天，返回的是所有tick的清单，否则返回
app.post('/historyData', async (req, res) => {
    let arr = await getRecords( req.body?.beginDate , req.body?.endDate )
    res.send(JSON.stringify(arr))
});
// 更新和删除用户键盘
app.post('/optKeymap', optKeymapFun); 
// 获取历史天数
app.post('/getHistoryDate', async (req, res) => {
    let arr = await getHistoryDate()
    res.send(JSON.stringify(arr))
});
// 获取统计信息
app.post('/statData', async (req, res) => {
    let arr = await statData(req.body?.beginDate , req.body?.endDate)
    res.send(JSON.stringify(arr))
});
// 更新和删除用户数据
app.post('/deleteData', deleteDataFun); 
// 版本和服务判断
app.post('/version', (req, res) => {res.send('showKeyBoardServer Version:'+ version);});
// 监听WS 连接事件
let wss = null


//  直接启动
startUp()

// 定时保存数据, 暂定 180秒保存一次
setInterval(saveLastData,180*1000)
