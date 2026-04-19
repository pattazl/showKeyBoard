import { CSSProperties } from 'vue'
import dayjs from 'dayjs'

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
  if (['getHistoryDate', 'minuteData', 'statData', 'getAppMinute'].includes(path)) {
    data = { ...data, db: dbSel }
  }
  if (data == null) {
    data = ''
  } else if (typeof data != 'string') {
    // 对于特定接口需要增加db参数
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
  'mergeAppName', 'activeAppShow', 'preAppNameEnable', 'allKeySwitch', 'needTraytip'
  , 'showKeyOnlyWeb','mergeChar']
// 转换字符串为数字或boolean
function str2Type(hash, flag) {
  for (const k in hash) {
    let val = hash[k];
    if (flag == 0) {  // 进行数据转换给界面
      // 如果不是字符串类型，证明之前已经处理过无需再处理
      if (typeof val !== 'string') {
        continue;
      }
      if (boolArr.indexOf(k) > -1) {
        hash[k] = (parseInt(val) == 1) as boolean
      } else if (/^-?(?!0\d)\d+$/.test(val)) { // 如果不是0，但是用0开头的数字，则为字符串，解决颜色的bug
        hash[k] = Number(val);
      }
    } else { // 界面转换给数据
      if (boolArr.indexOf(k) > -1) {
        hash[k] = val ? '1' : '0'
      } else if (typeof val !== 'string') {
        hash[k] = val.toString(); // 默认都是字符串
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
function showAppChart(leftKey/**out */, keyStatHash, opt, chart, mergeApp) {
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
      appName = getMatchAppname(appName, mergeApp)
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
  chart.setOption(opt);
  return retArr;
}
let dbSel = '' // 当前选择的数据源
function setDbSel(db) {
  dbSel = db
}
// 获取历史时间
async function getHistory(beginDate, endDate) {
  let res = [];
  // format(dateFormat)
  if (/^\d{4}-\d{2}-\d{2}$/.test(beginDate) && /^\d{4}-\d{2}-\d{2}$/.test(endDate) && endDate >= beginDate) {
    res = await ajax('historyData', { beginDate, endDate, db: dbSel })
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
function showLeftKey(leftAll, switchVal, leftKey, allKey, keyStatHash) {
  // 需要合并 左右控制键
  let newArr = new Set<string>();
  let newItemHash = {};
  let keys = allKey
  if (leftAll == 0) {
    keys = leftKey
  }
  let tempLeftKey = keys;
  if (switchVal) { // 合并
    keys.forEach(function (item) {
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
// 如果 map中的名字以Reg:开头则为正则
function getMatchAppname(val, map) {
  // 循环获取 map 中是否有 Reg:开头的
  try {
    for (let key in map) {
      if (key.indexOf('Reg:') == 0) {
        let regStr = key.substring(4);
        let reg = new RegExp(regStr, 'i')
        if (reg.test(val)) {
          return map[key]
        }
      }
    }
  } catch (e) {
    console.log('getMatchAppname error', e)
  }
  return map[val] ?? val;
}
// 应用程序名转换为显示名
function appPath2Name(val, map) {
  let newName = getMatchAppname(val, map).split(/[\\\/]/).pop().replace(/\.exe$/i, '')
  if (newName.length > 20) {
    newName = newName.substring(0, 18) + '...'
  }
  return newName
}
const dateFormat = 'YYYY-MM-DD'
const timeFormat = 'YYYY-MM-DD HH:mm:ss.SSS'

// 额外window的绑定事件
function addExtListener(chartArr) {
  // 时间框可拖动
  addDragFixedSelect();
  // 调整窗口大小
  if (chartArr != null) {
    winResize(chartArr)
  }
}
// window 调整窗口
let lastChartArr = [] // 保存当前 echarts 清单
function winResizeCore() {
  // console.log('winResizeCore')
  // 调用 ECharts 提供的 resize 方法，动态重绘图表
  lastChartArr?.forEach(x => {
    x.resize()
  })
}
function winResize(chartArr) {
  if (lastChartArr?.length == 0) {
    window.addEventListener('resize', winResizeCore);
  }
  lastChartArr = chartArr
}
// 以下为拖动时间选项
var isDragging = false;
var offset = { x: 0, y: 0 };
function addDragFixedSelect() {
  var mySpan = document.querySelector('#fixedSpan') as HTMLElement;
  var myDiv = document.querySelector('.fixedSelect>div:first-child>div:first-child') as HTMLElement;
  if (mySpan == null || myDiv == null) {
    setTimeout(addExtListener, 1000)
    return;
  }
  mySpan.addEventListener('mousedown', function (event: any) {
    isDragging = true;
    offset.x = event.offsetX;
    offset.y = event.offsetY;
  });
  document.addEventListener('mousemove', function (event: any) {
    if (isDragging) {
      myDiv.style.left = (event.clientX - offset.x) + 'px';
      myDiv.style.top = (event.clientY - offset.y) + 'px';
    }
  });
  document.addEventListener('mouseup', function (event) {
    isDragging = false;
  });
}
// 获取备份数据源清单
async function getDbs(dbs) {
  let res = await ajax('getDbs')
  if (res.code == 200) {
    // [{ label: contentText.intro33, value: 'TL' }]
    res.dbs.forEach(x => {
      try {
        let labArr = x.split('_')
        let label = labArr[0] + ' ' + dayjs(parseInt(labArr[1]) * 1000).format('YYYY-MM-DD HH:mm:ss')
        let line = { label, value: x }
        dbs.push(line)
      } catch (error) { }
    })
  }
}
// 分钟数变小时数
function minute2Hour(minutes) {
  let hour = Math.floor(minutes / 60)
  let mins = minutes % 60;
  let minInt = Math.floor(mins)
  let minStr = String(minInt).padStart(2, '0')
  let sec = ((mins - minInt) * 60).toFixed(0).padStart(2, '0')
  if (minutes > 60) {
    return `[${hour}:${minStr}:${sec}]`;
  } else {
    return `[${minStr}:${sec}]`;
  }
}
// 导出为制表符分隔的文本
const exportToText = (columns, tableData) => {
  debugger
  // 1. 创建表头行
  const headerRow = columns.map(col => col.title).join('\t')

  // 2. 创建数据行
  const dataRows = tableData.map(row => {
    return columns.map(col => row[col.key]).join('\t')
  }).join('\n')

  // 3. 合并表头和数据
  const textContent = `${headerRow}\n${dataRows}`

  // 4. 创建Blob并下载
  const blob = new Blob([textContent], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)

  // 创建下载链接
  const link = document.createElement('a')
  link.href = url
  link.download = 'Keys_' + new Date().toISOString().slice(0, 10) + '.txt'
  link.click()

  // 释放URL对象
  URL.revokeObjectURL(url)
}

const fingerOption = [
{
  title: {
      "text": "Left VS Right"
  },
  series: [{
    type: 'pie',
    data: [
      { value: 70, name: '类别A' },
      { value: 30, name: '类别B' }
    ],
    label: { show: true, formatter: '{b}: {d}%' },
    itemStyle: {
      borderWidth: 5,         // 间隙宽度
      borderColor: '#fff'
    }
  }]
},
{
  title: {
      "text": "By Rows"
  },
  xAxis: { type: 'value', name: '数值' },
  yAxis: { type: 'category', data: ['项目A', '项目B', '项目C', '项目D', '项目E'], name: '项目' },
  series: [{
    type: 'bar',
    data: [120, 200, 150, 80, 70],
    label: { show: true, position: 'right' },
    itemStyle: { color: '#18a058', borderRadius: [0, 4, 4, 0] }
  }]
},
{
  title: {
      "text": "By Finger"
  },
  xAxis: { type: 'value', name: '数值' },
  yAxis: { type: 'category', data: ['项目A', '项目B', '项目C', '项目D', '项目E'] },
  tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
  legend: { data: ['类别1', '类别2'] },
  series: [
    {
      name: '类别1',
      type: 'bar',
      stack: 'total',
      data: [120, 200, 150, 80, 70],
      label: { show: true, position: 'right' },
      itemStyle: { color: '#18a058' }
    },
    {
      name: '类别2',
      type: 'bar',
      stack: 'total',
      data: [80, 120, 100, 60, 50],
      label: { show: true, position: 'right' },
      itemStyle: { color: '#f0a020' }
    }
  ]
},
{
  title: {
      "text": "Every Finger"
  },
  xAxis: { type: 'category', data: ['A', 'B', 'C', 'D', 'E'] },
  yAxis: { type: 'value' },
  series: [{ type: 'bar', data: [120, 200, 150, 80, 70] }]
}
]
const fingerKeyMap = {
  leftPinky: ['Q', 'A', 'Z', '1', '`', '~', '!', '@'],
  leftRing: ['W', 'S', 'X', '2'],
  leftMiddle: ['E', 'D', 'C', '3', '#'],
  leftIndex: ['R', 'T', 'F', 'G', 'V', 'B', '4', '5', '$', '%'],
  rightIndex: ['Y', 'U', 'H', 'J', 'N', 'M', '6', '7', '^', '&'],
  rightMiddle: ['I', 'K', ',', '<', '8', '*'],
  rightRing: ['O', 'L', '.', '>', '9', '('],
  rightPinky: ['P', ';', ':', "'", '"', '[', '{', ']', '}', '\\', '|', '-', '_', '=', '+',
    '/', '?', 'Backspace', 'Enter', '0', ')'],
  thumb: [' ']
}

const rowKeyMap = {
  row1: ['`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', 'Backspace', '~', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+'],
  row2: ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '[', ']', '\\', '{', '}', '|'],
  row3: ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ';', "'", 'Enter', ':', '"'],
  row4: ['Z', 'X', 'C', 'V', 'B', 'N', 'M', ',', '.', '/', '<', '>', '?'],
  row5: [' ']
}

const fingerNames = {
  leftPinky: '小指',
  leftRing: '无名指',
  leftMiddle: '中指',
  leftIndex: '食指',
  rightIndex: '食指',
  rightMiddle: '中指',
  rightRing: '无名指',
  rightPinky: '小指',
  thumb: '拇指'
}

const rowNames = {
  row1: '第一行(数字行)',
  row2: '第二行(QWER行)',
  row3: '第三行(ASDF行)',
  row4: '第四行(ZXCV行)',
  row5: '第五行(空格)'
}

function matchKey(keyName, matchList) {
  let cleanKey = keyName.replace(/[!+^#<>]/g, '').toUpperCase()
  return matchList.some(k =>
    cleanKey.includes(k) ||
    keyName.toUpperCase().includes(k) ||
    k.toUpperCase().includes(cleanKey)
  )
}

function calculateStats(allKey, keyStatHash) {
  let fingerStats = {}, handStats = { left: 0, right: 0, thumb: 0 }
  let rowStats = {}, totalCount = 0

  Object.keys(fingerKeyMap).forEach(finger => { fingerStats[finger] = 0 })
  Object.keys(rowKeyMap).forEach(row => { rowStats[row] = 0 })

  allKey.forEach(keyName => {
    let count = keyStatHash[keyName] || 0
    if (count === 0 || keyName.startsWith('App-')) return

    totalCount += count

    Object.keys(fingerKeyMap).forEach(finger => {
      if (matchKey(keyName, fingerKeyMap[finger])) {
        fingerStats[finger] += count
        if (finger.includes('left')) handStats.left += count
        else if (finger.includes('right')) handStats.right += count
        else handStats.thumb += count
      }
    })

    Object.keys(rowKeyMap).forEach(row => {
      if (matchKey(keyName, rowKeyMap[row])) {
        rowStats[row] += count
      }
    })
  })

  return { fingerStats, handStats, rowStats, totalCount }
}

function showFinger(myChartArr, optionArr, chartIndices, allKey, keyStatHash,keyMap) {
  if (!myChartArr || !optionArr || !allKey || !keyStatHash || !chartIndices) return
  if (chartIndices.length !== 4) return

  const { fingerStats, handStats, rowStats, totalCount } = calculateStats(allKey, keyStatHash)
  console.log('----')
  console.log(optionArr)
  chartIndices.forEach((chartIdx, i) => {
    if (chartIdx >= myChartArr.length) return

    const chart = myChartArr[chartIdx]
    let opt = optionArr[chartIdx]
    console.log(opt)
    chart.setOption(opt)
    return
    if (i === 0) {
      opt = {
        title: { text: '左右手按键总数', left: 'center', top: 10 },
        tooltip: { trigger: 'item', formatter: '{b}: {c}次 ({d}%)' },
        legend: { bottom: 10 },
        series: [{
          type: 'pie',
          radius: ['40%', '70%'],
          data: [
            { name: '左手', value: handStats.left, itemStyle: { color: '#2080f0' } },
            { name: '右手', value: handStats.right, itemStyle: { color: '#d03050' } },
            { name: '拇指', value: handStats.thumb, itemStyle: { color: '#18a058' } }
          ].filter(d => d.value > 0),
          label: { show: true, formatter: '{b}: {d}%' }
        }]
      }
    } else if (i === 1) {
      const rowData = Object.keys(rowStats).map(row => ({
        name: rowNames[row],
        value: rowStats[row]
      })).filter(d => d.value > 0)

      opt = {
        title: { text: '键盘每行按键数', left: 'center', top: 10 },
        tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
        grid: { left: '3%', right: '10%', bottom: '3%', top: '60px', containLabel: true },
        xAxis: { type: 'value', name: '按键次数' },
        yAxis: { type: 'category', data: rowData.map(d => d.name), name: '键盘行' },
        series: [{
          type: 'bar',
          data: rowData.map(d => d.value),
          label: { show: true, position: 'right', formatter: '{c}次' },
          itemStyle: {
            color: ['#d03050', '#f0a020', '#18a058', '#2080f0', '#8050f0']
          }
        }]
      }
    } else if (i === 2) {
      const leftFingers = ['leftPinky', 'leftRing', 'leftMiddle', 'leftIndex']
      const rightFingers = ['rightPinky', 'rightRing', 'rightMiddle', 'rightIndex']

      opt = {
        title: { text: '左右手各手指对比', left: 'center', top: 10 },
        tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
        legend: { bottom: 10, data: ['左手', '右手'] },
        grid: { left: '3%', right: '10%', bottom: '60px', top: '60px', containLabel: true },
        xAxis: {
          type: 'category',
          data: ['小指', '无名指', '中指', '食指']
        },
        yAxis: { type: 'value', name: '按键次数' },
        series: [
          {
            name: '左手',
            type: 'bar',
            data: leftFingers.map(f => fingerStats[f]),
            itemStyle: { color: '#2080f0' },
            label: { show: true, position: 'top' }
          },
          {
            name: '右手',
            type: 'bar',
            data: rightFingers.map(f => fingerStats[f]),
            itemStyle: { color: '#d03050' },
            label: { show: true, position: 'top' }
          }
        ]
      }
    } else if (i === 3) {
      const allFingers = [
        { key: 'leftPinky', name: '左小指', color: '#a0c8ff' },
        { key: 'leftRing', name: '左无名指', color: '#60a0f0' },
        { key: 'leftMiddle', name: '左中指', color: '#2080f0' },
        { key: 'leftIndex', name: '左食指', color: '#0060d0' },
        { key: 'thumb', name: '拇指', color: '#18a058' },
        { key: 'rightIndex', name: '右食指', color: '#a03050' },
        { key: 'rightMiddle', name: '右中指', color: '#d03050' },
        { key: 'rightRing', name: '右无名指', color: '#f05070' },
        { key: 'rightPinky', name: '右小指', color: '#ff80a0' }
      ]

      opt = {
        title: { text: '各手指按键次数', left: 'center', top: 10 },
        tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
        grid: { left: '3%', right: '10%', bottom: '3%', top: '60px', containLabel: true },
        xAxis: { type: 'value', name: '按键次数' },
        yAxis: {
          type: 'category',
          data: allFingers.map(f => f.name)
        },
        series: [{
          type: 'bar',
          data: allFingers.map(f => ({
            value: fingerStats[f.key],
            itemStyle: { color: f.color }
          })),
          label: { show: true, position: 'right', formatter: '{c}次' },
          itemStyle: { borderRadius: [0, 4, 4, 0] }
        }]
      }
    }

    chart.setOption(opt)
  })
}

export {
  deepCopy, ajax, splitArr, str2Type, setWS, arrRemove, getHistory, getServer,
  getKeyDesc, showLeftKey, railStyle, showAppChart, appPath2Name, closeWS,
  dateFormat, timeFormat, addExtListener, getDbs, setDbSel, minute2Hour, exportToText,
  showFinger,fingerOption
}