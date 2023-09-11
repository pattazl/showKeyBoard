const WebSocket = require('ws');
const http = require('http');
const express = require('express')
const {getRecords} = require('./records');
const version = '1.1'


const { startUp , getParaFun, setParaFun , app ,exitFun ,dataFun} = require('./common');

//app.use(express.text());
app.use(express.json());
app.use(express.static('ui'));
// 返回全部参数信息
app.post('/getPara', getParaFun);
// 写入修改的参数信息
app.post('/setPara', setParaFun);
app.get('/', (req, res) => {res.send('Welcome to showKeyBoard backend service');});
app.post('/exit',exitFun);
app.post('/data', dataFun);
// 获取某某天，或某个tick的数据, 参数 date 如果是当天，返回的是所有tick的清单，否则返回
app.post('/historyData', async (req, res) => {
    let arr = await getRecords( req.body?.beginDate , req.body?.endDate )
    res.send(JSON.stringify(arr))
});
// 版本和服务判断
app.post('/version', (req, res) => {res.send('showKeyBoardServer Version:'+ version);});
// 监听WS 连接事件
let wss = null


//  直接启动
startUp()
