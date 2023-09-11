const WebSocket = require('ws');
const http = require('http');
const express = require('express')
const {insertData,getRecords} = require('./records');
const dayjs = require('dayjs');
const net = require('net');
const app = express()
const version = '1.1'

var fs = require('fs')
  , ini = require('ini')
const iniPath = '../../showKeyBoard.ini'
var config = ini.parse(fs.readFileSync(iniPath, 'utf-8'))

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
 //{ server } {port: 8080 }

//app.use(express.text());
app.use(express.json());
app.use(express.static('ui'));
// 返回全部参数信息
app.post('/getPara', (req, res) => {
  console.log('getPara')
  res.send(JSON.stringify(config));
});
// 写入修改的参数信息
app.post('/setPara', (req, res) => {
  console.log('setPara')
  var data = req.body
  let newConf = JSON.stringify(data)
  if( JSON.stringify(config) != newConf)
  {
		console.log('write ini')
		config = JSON.parse(newConf)
		fs.writeFileSync(iniPath, ini.stringify(data))
  }
  res.send({code:200});
});
app.get('/', (req, res) => {
  console.log(req.body)
  res.send('Welcome to showKeyBoard backend service');
});
app.post('/exit', (req, res) => {
  if( preData['tick']>0 )
  {
    // 需要保存
    insertData(preData)
  }
  console.log('exit')
  res.send('OK');
  wss.close()
  server.close(()=>{})  //  退出服务
  setTimeout(function(){process.exit();},1000);
});

// 发送数据
let preData = {}
app.post('/data', (req, res) => {
  var data = req.body
  console.log(data)
  if( preData['tick']>0 &&  data['tick']>0 &&  data['tick'] != preData['tick'] )
  {
    // tick不一样需要保存
    insertData(preData)
  }
  preData = data;
  //myWS.send(JSON.stringify(data) );
  wss.clients.forEach((client) => {
    if (client.readyState === WebSocket.OPEN) {
      client.send(`服务器端数据更新：${JSON.stringify(data)}`);
    }
  });
  // 如果 data.tick 不一样，则需要累计保存
  res.send('OK');
});
// 获取某某天，或某个tick的数据, 参数 date 如果是当天，返回的是所有tick的清单，否则返回
app.post('/historyData', async (req, res) => {
    let arr = await getRecords( req.body?.beginDate , req.body?.endDate )
    res.send(JSON.stringify(data))
});
// 版本和服务判断
app.post('/version', (req, res) => {
  console.log('version')
  res.send('showKeyBoardServer Version:'+ version);
});
// 监听WS 连接事件
let wss = null
function startWS(){
    wss = new WebSocket.Server(  { server });
    wss.on('connection', (ws) => {
      // 监听消息接收事件
      ws.on('message', (message) => {
        console.log('Received message:', message);

        // 发送消息给客户端
        ws.send('Server received your message: ' + message);
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
  server.listen(port,'127.0.0.1', () => {
    console.log(`Express server正在监听端口 ${port}`);
    writePort()
  }).on('error', (err) => {
    // 端口被占用时的错误处理
    console.log('创建服务异常');
  })
}

function writePort()
{
    if(port != parseInt(config.common.serverPort))
    {
        config.common.serverPort = port;
        // 写入将导致参数重启
        fs.writeFileSync(iniPath, ini.stringify(config))
    }
}
//  直接启动
(async()=>{
	// 首先判断端口是否被占用
	var attemptCount = 0,maxAttempts = 5
	while(1){
		let isPortInUse = true
		try{
		   isPortInUse = await checkPort(port);
		}catch(e){}
		if(!isPortInUse){
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
})()
