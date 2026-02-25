// =====================================
//  极简 OBS 键盘显示版
//  仅连接 ws://127.0.0.1:9000
// =====================================

// 建立本地 WebSocket 连接
const ws = new WebSocket("ws://127.0.0.1:9900");

// 连接成功
ws.onopen = () => {
    console.log("已连接到 127.0.0.1:9900");
    ws.send("ahkKeyShow");   // 告诉服务器我们要接收键盘数据
};

// 用于处理 0:: 动态刷新
let lastNewDiv = null;

// 创建文本显示
function createNewTxt(text) {
    let danmu = document.getElementById("danmu");
    if (!danmu) return;

    let container = document.createElement("div");
    let span = document.createElement("span");

    span.textContent = text;
    container.appendChild(span);

    lastNewDiv = span;

    // 插入到最前面
    if (danmu.firstChild) {
        danmu.insertBefore(container, danmu.firstChild);
    } else {
        danmu.appendChild(container);
    }

    // 最多保留 5 条
    while (danmu.children.length > 5) {
        danmu.removeChild(danmu.lastChild);
    }
}

// 接收消息
ws.onmessage = async (event) => {
    let text = event.data instanceof Blob
        ? await event.data.text()
        : event.data;

    console.log("收到:", text);

    // 如果是动态刷新
    if (text.startsWith("0::")) {
        text = text.replace("0::", "");
        if (lastNewDiv) {
            lastNewDiv.textContent = text;
        }
    } else {
        createNewTxt(text);
    }
};

// 连接关闭
ws.onclose = () => {
    console.log("连接已关闭");
};

// 错误处理
ws.onerror = (err) => {
    console.error("WebSocket错误:", err);
};