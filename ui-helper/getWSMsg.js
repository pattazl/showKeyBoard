const ws = new WebSocket('ws://localhost:9900');
ws.onopen = () => {
  console.log('已连接到服务器');
  // 发送字符串（浏览器会自动转换为二进制）
  ws.send('ahkKeyShow'); // 表明要收的数据
};
ws.onmessage = async (event) => {
  const blob = event.data;
  // 直接将 Blob 转为 UTF-8 字符串（默认编码，无需额外配置）
  const text = blob instanceof Blob ? await blob.text() : blob; 
  console.log('解码后的字符串:', text);
};