const WebSocket = require('ws');
const http = require('http');
const express = require('express')
const app = express()
const version = '1.1'

var fs = require('fs')
  , ini = require('ini')
const iniPath = './showKeyBoard.ini'
var config = ini.parse(fs.readFileSync(iniPath, 'utf-8'))

var port = parseInt(config.common.serverPort)
    
// 创建WebSocket服务器
const server = http.createServer(app);
 //{ server } {port: 8080 }

//app.use(express.text());
app.use(express.json());
app.get('/', (req, res) => {
  console.log(req.body)
  res.send('Hello, World! get');
});
app.post('/exit', (req, res) => {
  console.log('exit')
  res.send('1');
  server.close(()=>{})  //  退出服务
});

// 发送数据
app.post('/data', (req, res) => {
  console.log(req.body)
  res.send('1');
});
// 版本和服务判断
app.post('/version', (req, res) => {
  console.log('version')
  res.send('showKeyBoardServer Version:'+ version);
});
// 监听WS 连接事件
function startWS(){
    const wss = new WebSocket.Server(  { server });
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
maxAttempts = 5
attemptCount = 0
function createServer() {
  // 尝试启动服务器
  startWS()  // 配置WS服务
  server.listen(port,'127.0.0.1', () => {
    console.log(`Express server正在监听端口 ${port}`);
    writePort()
  }).on('error', (err) => {
    // 端口被占用时的错误处理
    console.log('端口被占用时的错误处理');
    if (attemptCount < maxAttempts) {
      // 增加尝试次数并自增端口号
      attemptCount++;
      port++;
      console.log(`端口 ${port - 1} 被占用，尝试使用下一个端口 ${port}...`);
      createServer(); // 递归调用创建服务器函数，尝试下一个端口
    } else {
      // 达到最大尝试次数，无法启动服务器
      console.log(`无法启动服务器，已达到最大尝试次数 ${maxAttempts}`);
    }
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
createServer()