import { CSSProperties } from 'vue'

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
function getHost() {
  let port = location.port
  if (port == '3000') {
    port = '9900' // 调试阶段
  }
  return `${location.hostname}:${port}`
}
function getServer() {
  return `http://${getHost()}/`
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
  let rsp = await fetch(`${getServer()}${path}`, {
    method: "POST",
    headers: headers,
    body: data
  })
  let result = await rsp.json();
  return result
}
// 布尔类型清单,bool list
const boolArr = ['skipCtrlKey', 'recordMouseMove', 'needShowKey', 'needRecordKey', 'ctrlState', 'guiBgTrans',
  'guiTrans', 'guiEdge', 'guiDpiscale', 'showHttpDebug', 'hideInWinPwd', 'mergeControl', 'fillDate', 'statProcInfo',
  'mergeAppName','activeAppShow']
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
      } else if (/^-?(?!0\d)\d+$/.test(hash[k])) { // 如果不是0，但是用0开头的数字，则为字符串，解决颜色的bug
        hash[k] = Number(hash[k]);
      }
    } else { // 界面转换给数据
      if (boolArr.indexOf(k) > -1) {
        hash[k] = hash[k] ? '1' : '0'
      } else if (typeof hash[k] !== 'string') {
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
  socket = new WebSocket('ws://' + getHost());

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
function closeWS() {
  if (socket != null) {
    socket.close()
    socket = null
  }
}
// 从数组中删除指定数据 , key 可能是数组或字符串
function arrRemove(arr /*out */, key) {
  let keylist = []
  if (typeof key == 'string') {
    keylist.push(key)
  } else {
    keylist = key
  }
  keylist.forEach(v => {
    let index = arr.indexOf(v);  // 查找指定的字符串在数组中的位置
    if (index !== -1) {  // 如果找到了
      arr.splice(index, 1);  // 从数组中删除该元素
    }
  })
}
// 控制应用图形显示,将 leftKey 中 App-开头的按键剥离出去
function showAppChart(leftKey, keyStatHash, opt, chart, mergeApp) {
  let appArr = leftKey.filter(x => x.indexOf('App-') > -1)
  arrRemove(leftKey, appArr) // 清除应用信息
  // 所有应用的数组清单
  //let newSet = new Set<string>(); let nameList = Array.from(newSet)
  let myList = [] /**用于过滤唯一应用名 */, myHash = [] /**用于图形 */, retArr = [] /**用于表格 */;
  let appSet = new Set<string>(); // 保存唯一APP名
  appArr.forEach(x => {
    let appName = x.replace(/^(App-Mouse-|App-Key-)/, '');
    appSet.add(appName)
  })
  appSet.forEach(appName => {
    let mouse = keyStatHash['App-Mouse-' + appName] ?? 0
    let key = keyStatHash['App-Key-' + appName] ?? 0
    let total = mouse + key
    // 如果能找到且需要合并名称
    if (mergeApp != null) {
      appName = mergeApp[appName] ?? appName
    }
    if (myList.indexOf(appName) == -1) {
      // 不存在就插入，并找对应的鼠标和键盘信息
      myList.push(appName)
      retArr.push({ appPath: appName, keyType: 'Keyboard', keyCount: key })
      retArr.push({ appPath: appName, keyType: 'Mouse', keyCount: mouse })
      // 如果没有预先匹配则取文件名
      // let appPath = appName.replace(/^App-/,'')
      myHash.push({ appName, mouse, key, total })
    } else {
      // 需要叠加数据
      retArr.find(x => x.appPath == appName && x.keyType == 'Keyboard').keyCount += key
      retArr.find(x => x.appPath == appName && x.keyType == 'Mouse').keyCount += mouse
      let obj = myHash.find(x => x.appName == appName)
      obj.mouse += mouse
      obj.key += key
      obj.total += total
    }
  })
  // 对 myHash 进行排序
  myHash.sort((a, b) => {
    if (a.total > b.total) {
      return -1;
    } else {
      return 1;
    }
  });
  // console.log(nameArr,mouseArr,keyArr)
  opt.xAxis[0].data = myHash.map(x => x.appName)
  // 鼠标
  opt.series[0].data = myHash.map(x => x.mouse)
  // 键盘
  opt.series[1].data = myHash.map(x => x.key)
  opt && chart.setOption(opt);
  return retArr;
}
// 获取历史时间
async function getHistory(beginDate, endDate) {
  let res = [];
  // format(dateFormat)
  if (/^\d{4}-\d{2}-\d{2}$/.test(beginDate) && /^\d{4}-\d{2}-\d{2}$/.test(endDate) && endDate >= beginDate) {
    res = await ajax('historyData', { beginDate, endDate })
  }
  //console.log('getHistory',res)
  return res
}
// 根据键名缩写转换为说明
function getKeyDesc(keyName) {
  return keyName
    .replace(/<\+/g, 'LShift ')
    .replace(/>\+/g, 'RShift ')
    .replace(/\+/g, 'Shift ')
    .replace(/<!/g, 'LAlt ')
    .replace(/>!/g, 'RAlt ')
    .replace(/!/g, 'Alt ')
    .replace(/<\^/g, 'LCtrl ')
    .replace(/>\^/g, 'RCtrl ')
    .replace(/\^/g, 'Ctrl ')
    .replace(/<#/g, 'LWin ')
    .replace(/>#/g, 'RWin ')
    .replace(/#/g, 'Win ')
}
function showLeftKey(switchVal, leftKey, keyStatHash) {
  // 需要合并 左右控制键
  let newArr = new Set<string>();
  let newItemHash = {};
  let tempLeftKey = leftKey;
  if (switchVal) { // 合并
    leftKey.forEach(function (item) {
      let newItem = item.replace(/[<>]/g, '')
      if (newItem != item) {
        if (newItemHash[newItem] == null) {
          newItemHash[newItem] = []
        }
        newItemHash[newItem].push(item)
      }
      newArr.add(newItem)
    })
    tempLeftKey = Array.from(newArr);
  }
  return tempLeftKey.map(function (item) {
    let count = 0
    if (newItemHash[item] != null) { // 经过替换，需要循环累加
      count = newItemHash[item].reduce((acc, curr) => acc + keyStatHash[curr], 0);
    } else {
      count = keyStatHash[item]
    }
    return { keyName: item, count, desc: getKeyDesc(item) }
  })
}
function railStyle({
  focused,
  checked
}: {
  focused: boolean
  checked: boolean
}) {
  const style: CSSProperties = {}
  if (checked) {
    style.background = '#d03050'
    if (focused) {
      style.boxShadow = '0 0 0 2px #d0305040'
    }
  } else {
    style.background = '#2080f0'
    if (focused) {
      style.boxShadow = '0 0 0 2px #2080f040'
    }
  }
  return style
}
// 应用程序名转换为显示名
function appPath2Name(val, map) {
  let newName = map[val] ?? val.split(/[\\\/]/).pop().replace(/\.exe$/i, '')
  if (newName.length > 20) {
    newName = newName.substring(0, 18) + '...'
  }
  return newName
}
const dateFormat = 'YYYY-MM-DD'
const timeFormat = 'YYYY-MM-DD HH:mm:ss.SSS'

export {
  deepCopy, ajax, splitArr, str2Type, setWS, arrRemove, getHistory, getServer,
  getKeyDesc, showLeftKey, railStyle, showAppChart, appPath2Name, closeWS,
  dateFormat,timeFormat,
}