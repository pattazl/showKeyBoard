const fs = require('fs');
const path = require('path');
const WebSocket = require('ws');
const http = require('http');
const express = require('express')
const { insertData, getDataSetting, setDataSetting, getKeymaps, optKeyMap, deleteData, dbName, updateDBStruct } = require('./records');
const dayjs = require('dayjs');
const net = require('net');
const app = express()
const ini = require('ini')
const fontList = require('font-list')
const JSZip = require("jszip");
const os = require('os');
// 2个配置文件
const iniPath = '../../showKeyBoard.ini'
const defaultIniPath = './showKeyBoard.desc.ini'
const keyPath = '../../keyList.txt'
const backupPath = '../../backup/'
const updateTimePath = path.join(__dirname, 'updateTime.txt');
const lastRecordPath = path.join(__dirname, 'lastRecord.json');
const pidfilePath = path.join(__dirname, 'kbserver.pid');
if (fs.existsSync(iniPath)) {
  var config = ini.parse(fs.readFileSync(iniPath, 'utf-8'))
}

var keyList; // 用于保存KeyList.txt 的文件信息
var dataSetting = {}; // 用于保存 dataSetting 的信息 统计的配置参数保存在 数据库中
var infoPC; // 用于保存PC的关键信息
var port = parseInt(config.common?.serverPort ?? 9900)
var remoteType = parseInt(config.common?.remoteType ?? 0)

let hostAddress = remoteType == 0 ? '127.0.0.1' : '0.0.0.0'
console.log(hostAddress)
let handleAutoSave = null  // 自动保存到DB的句柄

let networkIP = []
// 获取网络接口列表
function getNetworkList(port) {
  const networkInterfaces = os.networkInterfaces();
  // 遍历网络接口列表获取IP地址
  Object.keys(networkInterfaces).forEach(interfaceName => {
    const interfaceInfoList = networkInterfaces[interfaceName];
    interfaceInfoList.forEach(interfaceInfo => {
      // 排除非IPv4地址和127.0.0.1
      if (interfaceInfo.family === 'IPv4' && interfaceInfo.address !== '127.0.0.1') {
        networkIP.push(interfaceInfo.address + ':' + port);
      }
    });
  });
}
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

    server.listen(port, hostAddress);
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
  server.listen(port, hostAddress, () => {
    console.log(`Express server正在监听端口 ${port}`);
    writePort()
  }).on('error', (err) => {
    // 端口被占用时的错误处理
    console.log('创建服务异常');
  })
}
// 解决 \# 的转义问题
function myIniwrite(config) {
  let s = ini.stringify(config)
  s = s.replace(/\\#/g, '#')
  s = s.replace(/=(.*#.*)/g, '="$1"')
  fs.writeFileSync(iniPath, s)
}
function writePort() {
  if (port != parseInt(config.common.serverPort)) {
    config.common.serverPort = port;
    // 写入将导致参数重启
    myIniwrite(config)
  }
}
//  直接启动
async function startUp() {
  // 补数据
  patchLastData()
  let hasError = false;
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
      if (!isPortInUse) {
        createServer()
        getNetworkList(port)
        break;
      }
    } catch (e) {
      hasError = true
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
      hasError = true
      break;
    }
  }
  // 定时保存数据, 默认180 秒保存一次
  if (!hasError) {
    autoSaveFun()
  }
  // 需要检查和更新数据库表结构
  updateDBStruct();
}
function autoSaveFun(interval) {
  if (interval == null) {
    interval = config.common?.autoSave2Db ?? 180
  }
  if (handleAutoSave != null) {
    clearInterval(handleAutoSave)
  }
  handleAutoSave = setInterval(saveLastData, interval * 1000)
}
// 封装 ps-node的回调函数，性能过于低下，暂时放弃
// const ps = require('ps-node');
// function psNode(pid) {
//   return new Promise((resolve, reject) => {
//     ps.lookup({ pid }, (err, resultList) => {
//       if (err) {
//         console.error(err);
//         resolve({err})
//         return;
//       }
//       if (resultList.length > 0) {
//         const process = resultList[0];
//         console.log(`PID ${process.pid} 的执行文件名: ${JSON.stringify(process)}`);
//         resolve(process)
//       } else {
//         console.log(`没有找到 PID 为 ${pid} 的进程`);
//         resolve({})
//       }
//     })
//   })
// }
// 只允许一个进程
function oneInstance() {
  const pidfilePath = path.join(__dirname, 'kbserver.pid');
  // 读取文件判断是否有此进程，有可能是异常遗留的文件

  // 检查是否已存在 pidfile 文件
  if (fs.existsSync(pidfilePath)) {
    let pid = fs.readFileSync(pidfilePath, 'utf-8')
    try {
      // 判断pid是否存在，如果存在则的确程序在运行，否则删除文件
      process.kill(pid, 0)
      console.log('程序已在运行中');
      return false
    } catch (err) {
      console.log('意外关闭,先删除Pid文件');
      fs.unlinkSync(pidfilePath)
    }
  }
  // 将当前进程的 PID 写入 pidfile 文件
  fs.writeFileSync(pidfilePath, process.pid.toString(), 'utf8');
  // 退出前插入数据库
  // 应用程序退出时删除 pidfile 文件
  process.on('exit', (code) => {
    if (fs.existsSync(pidfilePath)) fs.unlinkSync(pidfilePath); // 无法操作数据库删除
    console.log('exit')
  });
  process.on('beforeExit', (code) => {
    if (fs.existsSync(pidfilePath)) fs.unlinkSync(pidfilePath); // 无法操作数据库删除
    console.log('exit')
  });
  // 处理 SIGINT 和 SIGTERM 信号，确保应用程序正常退出时删除 pidfile 文件
  //   'CTRL + C ,
  ['SIGINT', 'SIGTERM', 'SIGQUIT', 'SIGKILL']
    .forEach(signal => process.on(signal, async () => {
      if (fs.existsSync(pidfilePath)) fs.unlinkSync(pidfilePath);
      await exitFun()
      //process.exit(0);
    }));
  return true;
}
// default 中存在的hash值复制到 obj 中
function mergeObjects(obj1, obj2) {
  // 遍历obj2的属性
  for (let prop in obj2) {
    // 如果obj1对应的属性不存在，则直接将obj2的属性添加到obj1中
    if (typeof obj1[prop] === 'undefined') {
      obj1[prop] = obj2[prop];
    } else if (typeof obj1[prop] === 'object' && typeof obj2[prop] === 'object') {
      // 如果两个属性都是普通对象，则递归调用mergeObjects函数
      mergeObjects(obj1[prop], obj2[prop]);
    }
  }
  return obj1;
}
// 获取 和设置  KeyList.txt showKeyBoard.ini ，从文件中读取 
async function getParaFun(req, res) {
  console.log('getPara')
  // 重新再读取一次
  keyList = {}
  config = ini.parse(fs.readFileSync(iniPath, 'utf-8'))
  if (fs.existsSync(defaultIniPath)) {
    var defaultConfig = ini.parse(fs.readFileSync(defaultIniPath, 'utf-8'))
  }
  //console.log(defaultConfig)
  mergeObjects(config, defaultConfig)
  //console.log(config)
  const keyTxt = (fs.readFileSync(keyPath, 'utf-8'))
  const arr = keyTxt.split('\n')
  for (v of arr) {
    let arr2 = v.split(':')
    if (arr2.length == 2) {
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
  res.send(JSON.stringify({ config, keyList, fonts, infoPC, dataSetting, keymaps, networkIP }));
}

// 保存参数 ，包括各种文件和数据的保存
function setParaFun(req, res) {
  console.log('setPara remoteAddress:', req.connection.remoteAddress)
  // 如果不是2，那么需要判断来源IP是否本地
  if (req.connection.remoteAddress != '127.0.0.1' && remoteType != 2) {
    // 不允许修改，需要返回 Error
    res.send({ code: 2 });
    return
  }
  var data = req.body  // 包含 config 和 keyList, dataSetting
  let isUpdate = false
  // 保存 keylist
  let newKeyList = JSON.stringify(data.keyList)
  if (JSON.stringify(keyList) != newKeyList) {
    let keyArr = []
    for (let k in data.keyList) {
      keyArr.push(k + ' : ' + data.keyList[k])
    }
    keyList = data.keyList
    console.log('write keyPath')
    isUpdate = true;
    fs.writeFileSync(keyPath, keyArr.join('\n'), 'utf-8')
  }
  // 保存配置文件
  let newConf = JSON.stringify(data.config)
  if (data.config?.common?.autoSave2Db != config.common?.autoSave2Db) {
    autoSaveFun();
  }
  // 当端口号改变时候需要服务重启，此步骤由客户端来完成
  if (JSON.stringify(config) != newConf || isUpdate) {
    console.log('write ini')
    config = data.config
    myIniwrite(config)

  }
  // 对于 dataSetting 需要更新数据库
  let newDataSetting = JSON.stringify(data.dataSetting)
  if (JSON.stringify(dataSetting) != newDataSetting) {
    setDataSetting(data.dataSetting);
    console.log('setDataSetting')
  }
  res.send({ code: 200 });
}
// 接收客户端发送的PC相关信息，比如屏幕等
function sendPCInfo(req, res) {
  var data = req.body
  let flag = data.flag ?? 0;
  delete data.flag; // 删除此节点
  infoPC = data; // 将数据保存给全局变量
  console.log(infoPC)
  //每次客户端重启都会调用
  if (flag == 0) {
    autoBackup();
  }
  res.send({ code: 200 });
}
// 接受客户端发送的数据
let preData = {}
function dataFun(req, res) {
  var data = req.body
  //console.log('mouseDistance,tick',data.mouseDistance,data.tick)
  if (preData['tick'] > 0 && data['tick'] > 0 && data['tick'] != preData['tick']) {
    // tick不一样需要保存
    console.log('preTick,tick:', preData['tick'], data['tick'])
    insertDataFun(preData)
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
// 补充上一次的数据
function patchLastData() {
  if (fs.existsSync(lastRecordPath) && fs.existsSync(updateTimePath)) {
    let json = JSON.parse(fs.readFileSync(lastRecordPath))
    let updateTime = fs.readFileSync(updateTimePath)
    if (json['updateTime'] > updateTime) {
      console.log('patchLastData')
      insertDataFun(json)
    }
    fs.unlinkSync(lastRecordPath)
  }
}
// 对 insertData 函数封装处理下
async function insertDataFun(records) {
  // 需要获取 updateTime 字段，并保存到文件中记录最近的更新时间
  if (records['updateTime'] != null) {
    fs.writeFileSync(updateTimePath, records['updateTime'])
    delete records['updateTime'] // 删除数据
  }
  return await insertData(records)
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
  // 如果没有新数据进来将不会保存
  if (preData['tick'] > 0) {
    // 需要保存
    console.log('insertDataFun')
    await insertDataFun(preData)
    preData['tick'] = 0
  }
}

// 接收keymap 
async function optKeymapFun(req, res) {
  var data = req.body
  await optKeyMap(data)
  res.send({ code: 200 });
}
// 删除数据
async function deleteDataFun(req, res) {
  var data = req.body
  if (typeof data.flag == 'number' && data.date instanceof Array) {
    // let dateStr = data.date.map(x=>{ if(typeof x =='string')return "'"+x+"'";else return x }).join(',')
    await deleteData(data.date, data.flag)
    res.send({ code: 200 });
  } else {
    res.send({ code: 10 });
  }

}
// 生成相关 zip文件，处理方式根据函数
function zipCore(cbFun) {
  const zip = new JSZip();
  let fileContent;
  fileContent = fs.readFileSync(iniPath);
  zip.file(path.basename(iniPath), fileContent);
  fileContent = fs.readFileSync(keyPath);
  zip.file(path.basename(keyPath), fileContent);
  fileContent = fs.readFileSync(dbName);
  zip.file(dbName, fileContent);

  zip.generateAsync({ type: "nodebuffer" }).then(function (content) {
    // see FileSaver.js
    cbFun(content)
  });
}
// 打包配置文件
function zipDownload(req, res) {
  zipCore(function (content) {
    // see FileSaver.js
    let fileName = new Date().getTime().toString(36);
    // 设置响应头
    res.set({
      'Content-Type': 'application/octet-stream',
      'Content-Disposition': `attachment; filename="showkeyboard-${fileName}.zip"`
    });
    // 发送二进制数据
    res.send(content);
  })
}
// 清理目录
function clearUpload(hash) {
  let dir = path.dirname(hash.path)
  fs.readdir(dir, (err, files) => {
    if (err) {
      console.error(err);
      return;
    }
    // 对于3分钟前的文件全部删除
    let minName = (new Date().getTime() - 3 * 60 * 1000) + '.zip';
    files.forEach(file => {
      const filePath = path.join(dir, file);
      let name = path.basename(filePath)
      if (name < minName) {
        fs.unlinkSync(filePath)
      }
    });
  });
}
const configHash = {
  'showKeyBoard.ini': iniPath,
  'keyList.txt': keyPath,
  'records.db': dbName,
}
function unZipCore(hash) {
  let content = fs.readFileSync(hash.path)
  return new Promise((resolve, reject) => {
    const zip = new JSZip();
    zip.loadAsync(content)
      .then(async function (contents) {
        //console.log( JSON.stringify(contents))
        for (let relativePath in contents.files) {
          let file = contents.files[relativePath]
          //console.log(relativePath,file.dir,file._data.compressedContent)
          if (!file.dir) { // 过滤掉目录
            const fileData = await file.async("uint8array"); // 以uint8array格式读取文件内容
            // 在此处执行对解压出的文件的操作，例如保存到本地
            let configPath = configHash[relativePath]
            fs.writeFileSync(configPath, fileData)
            hash.msg.push(relativePath + ' Ok')
          }
        }
        /*
        contents.forEach(async function (relativePath, file) {
          console.log(relativePath)
          if (!file.dir) { // 过滤掉目录
            const fileData = await file.async("uint8array"); // 以uint8array格式读取文件内容
            // 在此处执行对解压出的文件的操作，例如保存到本地
            let configPath = configHash[relativePath]
            fs.writeFileSync(configPath, fileData)
            hash.msg.push(relativePath + ' Ok\n')
            console.log('OKOK')
          }
        });*/
        resolve(0)
      })
      .catch(function (error) {
        let msg = "解压缩失败: "
        console.error(msg, error);
        hash.msg.push(msg)
        resolve(0)
      });
  })
}
// 解压配置文件
async function zipUpload(req, res) {
  const file = req.file;
  // 文件信息
  //console.log(file.originalname); // 原始文件名
  // console.log(file.filename); // 存储在服务器上的文件名
  //console.log(file.path); // 存储路径
  let hash = { oriName: file.originalname, path: file.path, msg: ['Upload Success'], code: 0 }
  // 需要清理旧的上传文件
  clearUpload(hash)
  // 解压新的上传文件 // 覆盖新的文件
  try {
    await unZipCore(hash)
    hash.code = 200;
  } catch {
    hash.code = 1;
  }
  console.log(hash)
  res.send(JSON.stringify(hash));
}
// 根据参数自动备份
function autoBackup() {
  let days = parseInt(dataSetting.autoBackupDays ?? 3, 10)
  function getDayStr(before) {
    let beforeDays = dayjs().subtract(before, 'day');
    return `skb_${beforeDays.format('YYYY-MM-DD-HH-mm-ss')}.zip`
  }
  if (!fs.existsSync(backupPath)) {
    fs.mkdirSync(backupPath)
  }
  let lastBackUp = backupPath + getDayStr(1)
  if (days > 0) {
    // backup 下保存文件
    zipCore(function (content) {
      // see FileSaver.js
      fs.writeFileSync(lastBackUp, content);
    })

    // 清理 days前的文件
    let beforeStr = getDayStr(days + 1)
    fs.readdir(backupPath, (err, files) => {
      if (err) {
        console.error('无法读取文件夹内容', err);
        return;
      }
      // 遍历文件列表
      files.forEach((file) => {
        if (file < beforeStr) {
          // 如果文件名表示的数字小于阈值数字，就删除该文件
          fs.unlink(path.join(backupPath, file), (err) => {
            if (err) {
              console.error('无法删除文件', file, err);
            } else {
              console.log('已删除文件', file);
            }
          });
        }
      });
    });
  }
}
module.exports = {
  startUp, getParaFun, setParaFun, app, dataFun, exitFun, sendPCInfo, saveLastData, optKeymapFun,
  deleteDataFun, zipDownload, zipUpload
};