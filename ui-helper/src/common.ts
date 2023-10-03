
function deepCopy(obj) {
  if (typeof obj !== 'object' || obj === null) {
    return obj;
  }
  const copy = Array.isArray(obj) ? [] : {};
  for (let key in obj) {
    if (Object.prototype.hasOwnProperty.call(obj, key)) {
      copy[key] = deepCopy(obj[key]);
    }
  }
  return copy;
}
function getHost(){
  let port = location.port
  if (port == '3000') {
    port = '9900' // 调试阶段
  }
  return `127.0.0.1:${port}`
}
// ajax核心模块
async function ajax(path, data = null) {
  console.log('ajax')
  // 测试环境
  
  const headers = {
    "Content-Type": "application/json",
  };
  if (data == null) {
    data = ''
  } else if (typeof data != 'string') {
    data = JSON.stringify(data)
  }
  let rsp = await fetch(`http://${getHost()}/${path}`, {
    method: "POST",
    headers: headers,
    body: data
  })
  let result = await rsp.json();
  return result
}
// 布尔类型清单
const boolArr = ['skipCtrlKey', 'recordMouseMove', 'needShowKey', 'needRecordKey', 'ctrlState', 'guiBgTrans', 'guiTrans', 'guiEdge', 'guiDpiscale']
// 转换字符串为数字或boolean
function str2Type(hash, flag) {
  for (let k in hash) {
    if (flag == 0) {  // 进行数据转换给界面
      // 如果不是字符串类型，证明之前已经处理过无需再处理
      if (typeof hash[k] !== 'string') {
        return;
      }
      if (boolArr.indexOf(k) > -1) {
        hash[k] = hash[k] == 1
      } else if ('guiBgcolor' == k && hash[k].indexOf('#') == -1) {
        // 有颜色字段,需要加上透明度
        hash[k] = '#' + hash[k] + parseInt(hash.guiOpacity, 10).toString(16)
      } else if ('guiTextColor' == k && hash[k].indexOf('#') == -1) {
        // 有颜色字段
        hash[k] = '#' + hash[k]
      } else if (/^-?\d+$/.test(hash[k])) {
        hash[k] = Number(hash[k]);
      }
    } else { // 界面转换给数据
      if (boolArr.indexOf(k) > -1) {
        hash[k] = hash[k] ? '1' : '0'
      } else if ('guiBgcolor' == k && hash[k].indexOf('#') != -1) {
        // 有颜色字段,需要加上透明度
        let color = hash[k].replace(/#/, '')
        hash[k] = color.substr(0, 6);
        hash.guiOpacity = parseInt(color.substr(6, 2), 16)
        hash.guiBgTrans = (hash.guiOpacity == 0) ? 1 : 0
      } else if ('guiTextColor' == k) {
        // 有颜色字段
        hash[k] = hash[k].replace(/#/, '')
      } if (typeof hash[k] !== 'string') {
        hash[k] = hash[k].toString(); // 默认都是字符串
      }
    }
  }
}
function splitArr(str) {
  let arr = []
  if (str.length > 0) {
    arr = str.split('|')
  }
  return arr;
}

let socket = null

function setWS(callback) {
    // 创建WebSocket对象并连接服务器
    socket = new WebSocket('ws://'+getHost());

    // 监听连接成功事件
    socket.onopen = () => {
        console.log('Connected to WebSocket server');

        // 发送消息给服务器
        socket.send('Hello, Server!');
    };

    // 监听服务器发送的消息事件
    socket.onmessage = (event) => {
        const message = event.data;
        //console.log('Received message:', message);
        callback(message)
    };

    // 监听连接关闭事件
    socket.onclose = () => {
        console.log('Disconnected from WebSocket server');
    };
}
// 从数组中删除指定数据
function arrRemove(arr /*out */,key)
{
  let index = arr.indexOf(key);  // 查找指定的字符串在数组中的位置
  if (index !== -1) {  // 如果找到了
    arr.splice(index, 1);  // 从数组中删除该元素
  }
}

// 获取历史时间
async function getHistory(beginDate,endDate){
  let res = [];
  // format('YYYY-MM-DD')
  if(/^\d{4}-\d{2}-\d{2}$/.test(beginDate) && /^\d{4}-\d{2}-\d{2}$/.test(endDate) && endDate>=beginDate ){
    res = await ajax('historyData', { beginDate, endDate })
  }
  //console.log('getHistory',res)
  return res
}
export {deepCopy,ajax,splitArr,str2Type,setWS,arrRemove,getHistory}