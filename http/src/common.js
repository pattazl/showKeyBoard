const fs = require('fs');
const path = require('path');
const WebSocket = require('ws');
const http = require('http');
const express = require('express')
const { insertData, getDataSetting,setDataSetting, getKeymaps,optKeyMap} = require('./records');
const dayjs = require('dayjs');
const net = require('net');
const app = express()
const ini = require('ini')
const fontList = require('font-list')
// 2个配置文件
const iniPath = '../../showKeyBoard.ini'
const keyPath = '../../keyList.txt'
var config = ini.parse(fs.readFileSync(iniPath, 'utf-8'))
var keyList; // 用于保存KeyList.txt 的文件信息
var dataSetting = {}; // 用于保存 dataSetting 的信息
var statPara; // 统计的配置参数保存在 数据库中
var infoPC; // 用于保存PC的关键信息
var port = parseInt(config.common.serverPort)

function checkPort(port) {
  const server = net.createServer();
  return new Promise((resolve, reject) => {
    server.once('error', (err) => {
      if (err.code === 'EADDRINUSE') {
        resolve(true); // 端口被占用
      } else {
        reject(err); // 发生其他错误
      }
    });

    server.once('listening', () => {
      server.close(() => {
        resolve(false); // 端口未被占用
      });
    });

    server.listen(port, '127.0.0.1');
  });
}
// 创建WebSocket服务器
const server = http.createServer(app);
// 监听WS 连接事件
let wss = null
function startWS() {
  wss = new WebSocket.Server({ server });
  wss.on('connection', (ws) => {
    // 监听消息接收事件
    ws.on('message', (message) => {
      console.log('Received message:', message);

      // 发送消息给客户端
      //ws.send('Server received your message: ' + message);
      ws.send(JSON.stringify(preData));
    });
    // 监听断开连接事件
    ws.on('close', () => {
      console.log('Client disconnected');
    });
  });
  wss.on('error', () => {
    console.log('wss error');
    wss.close() // 发现异常需要关闭重新开始
  });
}
// 创建服务器函数
function createServer() {
  // 尝试启动服务器
  startWS()  // 配置WS服务
  server.listen(port, '127.0.0.1', () => {
    console.log(`Express server正在监听端口 ${port}`);
    writePort()
  }).on('error', (err) => {
    // 端口被占用时的错误处理
    console.log('创建服务异常');
  })
}

function writePort() {
  if (port != parseInt(config.common.serverPort)) {
    config.common.serverPort = port;
    // 写入将导致参数重启
    fs.writeFileSync(iniPath, ini.stringify(config))
  }
}
//  直接启动
async function startUp() {
  // 程序不能重复运行
  if (!oneInstance()) {
    return;
  }
  // 首先判断端口是否被占用
  var attemptCount = 0, maxAttempts = 5
  while (1) {
    let isPortInUse = true
    try {
      isPortInUse = await checkPort(port);
    } catch (e) { }
    if (!isPortInUse) {
      createServer()
      break;
    }
    port++;
    if (attemptCount < maxAttempts) {
      // 增加尝试次数并自增端口号
      attemptCount++;
      console.log(`端口 ${port - 1} 被占用，尝试使用下一个端口 ${port}...`);
    } else {
      // 达到最大尝试次数，无法启动服务器
      console.log(`无法启动服务器，已达到最大尝试次数 ${maxAttempts}`);
      break;
    }
  }
}
// 只允许一个进程
function oneInstance() {
  const pidfilePath = path.join(__dirname, 'kbserver.pid');
  // 读取文件判断是否有此进程，有可能是异常遗留的文件

  // 检查是否已存在 pidfile 文件
  if (fs.existsSync(pidfilePath)) {
    let pid = fs.readFileSync(pidfilePath, 'utf-8')
    try{
      // 判断pid是否存在，如果存在则的确程序在运行，否则删除文件
      process.kill(pid,0)
      console.log('程序已在运行中');
      return false
    }catch (err) {
      console.log('意外关闭,先删除Pid文件');
      fs.unlinkSync(pidfilePath)
    }
  }
  // 将当前进程的 PID 写入 pidfile 文件
  fs.writeFileSync(pidfilePath, process.pid.toString(), 'utf8');
  // 退出前插入数据库
  // 应用程序退出时删除 pidfile 文件
  process.on('exit', () => {
    fs.unlinkSync(pidfilePath); // 无法操作数据库删除
    console.log('exit')
  });
  // 处理 SIGINT 和 SIGTERM 信号，确保应用程序正常退出时删除 pidfile 文件
  //   'CTRL + C ,
  ['SIGINT', 'SIGTERM', 'SIGQUIT','SIGKILL']
  .forEach(signal => process.on(signal, () => {
    exitFun()
    //process.exit(0);
  }));
  return true;
}
// 获取 和设置  KeyList.txt showKeyBoard.ini ，从文件中读取 
async function getParaFun(req, res) {
  console.log('getPara')
  // 重新再读取一次
  keyList = {}
  config = ini.parse(fs.readFileSync(iniPath, 'utf-8'))
  const keyTxt = (fs.readFileSync(keyPath, 'utf-8'))
  const arr = keyTxt.split('\n')
  for(v of arr)
  {
    let arr2 = v.split(':')
    if(arr2.length == 2)
    {
      const k = arr2[0].trim()
      const vv = arr2[1].trim()
      keyList[k] = vv
    }
  }
  //console.log(keyList)
  let fonts = await fontList.getFonts()
  //console.log(fonts)
  // 获取数据库中的setting
  dataSetting = await getDataSetting()
  // 获取数据库中的setting
  let keymaps = await getKeymaps()
  
  // 返回大 JSON
  res.send(JSON.stringify( {config,keyList,fonts,infoPC,dataSetting,keymaps}));
}

// 保存参数 ，包括各种文件和数据的保存
function setParaFun(req, res) {
  console.log('setPara')
  var data = req.body  // 包含 config 和 keyList, dataSetting
  let isUpdate = false
  let newKeyList = JSON.stringify(data.keyList)
  if( JSON.stringify(keyList) != newKeyList){
    let keyArr = []
    for(let k in data.keyList)
    {
      keyArr.push(k+' : '+data.keyList[k])
    }
    keyList = data.keyList
    console.log('write keyPath')
    isUpdate = true;
    fs.writeFileSync(keyPath,keyArr.join('\n'), 'utf-8')
  }
  let newConf = JSON.stringify(data.config)
  if (JSON.stringify(config) != newConf || isUpdate) {
    console.log('write ini')
    config = data.config
    fs.writeFileSync(iniPath, ini.stringify(config))
  }
  // 对于 dataSetting 需要更新数据库
  let newDataSetting  = JSON.stringify(data.dataSetting)
  if(JSON.stringify(keyList) != newDataSetting ){
	setDataSetting(data.dataSetting);
	console.log('setDataSetting')
  }
  res.send({ code: 200 });
}
// 接收客户端发送的PC相关信息，比如屏幕等
function sendPCInfo(req, res){
  var data = req.body
  infoPC = data; // 将数据保存给全局变量
  console.log(infoPC)
  res.send({ code: 200 });
}
// 接受客户端发送的数据
let preData = {}
function dataFun(req, res) {
  var data = req.body
  //console.log('mouseDistance,tick',data.mouseDistance,data.tick)
  if (preData['tick'] > 0 && data['tick'] > 0 && data['tick'] != preData['tick']) {
    // tick不一样需要保存
    console.log( 'preTick,tick:', preData['tick'],data['tick'],)
    insertData(preData)
  }
  preData = data;
  //myWS.send(JSON.stringify(data) );
  wss.clients.forEach((client) => {
    if (client.readyState === WebSocket.OPEN) {
      client.send(JSON.stringify(data));
    }
  });
  // 如果 data.tick 不一样，则需要累计保存
  res.send('OK');
}
// 触发正常退出
async function exitFun() {
  await saveLastData()
  console.log('exitFun')
  wss.close()
  server.close(() => { })  //  退出服务
  setTimeout(function () { process.exit(0); }, 1);
}
// 退出前保存最后的数据
async function saveLastData() {
  if (preData['tick'] > 0) {
    // 需要保存
    console.log('insertData')
    await insertData(preData)
    preData['tick'] = 0
  }
}

// 接收keymap 
async function optKeymapFun(req, res){
  var data = req.body
  await optKeyMap(data)
  res.send({ code: 200 });
}

module.exports = {
  startUp, getParaFun, setParaFun, app, dataFun, exitFun,sendPCInfo,saveLastData,optKeymapFun
};