const WebSocket = require('ws');
const http = require('http');
const express = require('express')
const app = express()
const port = 9900
// 创建WebSocket服务器
const server = http.createServer(app);
const wss = new WebSocket.Server({ server });
//
app.get('/', (req, res) => {
  res.send('Hello, World!');
});
// 监听连接事件
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


try {
  app
    .listen(9900, () => {
      console.log('start success.')
    })
    .on('error', e => {
      console.error(e.code, e.message)
    })
} catch (err) {
  console.error(err)
}