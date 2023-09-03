function setWS() {
    // 创建WebSocket对象并连接服务器
    const socket = new WebSocket('ws://localhost:9901');

    // 监听连接成功事件
    socket.onopen = () => {
        console.log('Connected to WebSocket server');

        // 发送消息给服务器
        socket.send('Hello, Server!');
    };

    // 监听服务器发送的消息事件
    socket.onmessage = (event) => {
        const message = event.data;
        console.log('Received message:', message);
    };

    // 监听连接关闭事件
    socket.onclose = () => {
        console.log('Disconnected from WebSocket server');
    };
}

setWS()