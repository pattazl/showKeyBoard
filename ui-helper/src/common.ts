import { CSSProperties } from 'vue'
import dayjs from 'dayjs'
let contentText = {} // 语言包
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
  , 'showKeyOnlyWeb', 'mergeChar']
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

let fingerOption = []; // 手指图表参数
let allCharts = []  // 全部图表对象
let fingerCharts = []  // 手指对象的列表
let fingerIndex = [] // 手指对象的序号
let rowKeyMap: Record<string, string[]> = {} // 当前键盘行按键映射
// 获取手指图表配置，根据当前语言动态生成
function getFingerOption() {
  const t = (key: string) => (contentText as any)[key] || key;
  return [
    {
      title: {
        text: t('intro219'),
      },
      tooltip: { trigger: 'item', formatter: '{b}: {c} ({d}%)' },
      series: [{
        type: 'pie',
        radius: ['45%', '70%'],
        data: [
          { value: 120, name: t('intro224'), itemStyle: { color: '#d03050' } }, // 红 右
          { value: 70, name: t('intro223'), itemStyle: { color: '#2080f0' } },  // 蓝 左
        ],
        itemStyle: { borderWidth: 5, borderColor: '#fff' },
        label: { show: true, position: 'outside', formatter: '{b}\n{c} ({d}%)' }
      }]
    },
    {
      title: { text: t('intro220') },
      tooltip: {
        trigger: 'axis', formatter: (params: any) => {
          const p = params[0];
          return `${p.name}${contentText.intro230}<br/>${listRowKey(p.name)}<br/>${contentText.intro115}: ${p.value}`;
        }
      },
      xAxis: { type: 'value' },
      yAxis: { type: 'category', data: [1, 2, 3, 4] },
      series: [{
        type: 'bar',
        data: [120, 200, 150, 80],
        label: { show: true, position: 'right' },
        itemStyle: { color: '#18a058', borderRadius: [0, 4, 4, 0] }
      }]
    },
    {
      title: { text: t('intro221') },
      xAxis: { type: 'value' },
      yAxis: { type: 'category', data: [t('intro225'), t('intro226'), t('intro227'), t('intro228')] },
      tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
      series: [
        {
          name: t('intro223'),
          type: 'bar',
          stack: 'total',
          data: [120, 200, 150, 80],
          label: { show: true, position: 'inside' },
          itemStyle: { color: '#2080f0' }
        },
        {
          name: t('intro224'),
          type: 'bar',
          stack: 'total',
          data: [80, 120, 100, 60],
          label: { show: true, position: 'inside' },
          itemStyle: { color: '#d03050' }
        }
      ]
    },
    {
      title: { text: t('intro222') },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'shadow' },
        formatter: (params: any) => {
          const p = params[0];
          return `${p.name} (${p.data.hand})<br/>${p.data.hand}: ${p.value}`;
        }
      },
      xAxis: {
        type: 'category',
        data: [t('intro225'), t('intro226'), t('intro227'), t('intro228'), t('intro229'), t('intro228'), t('intro227'), t('intro226'), t('intro225')],
        axisLabel: { rotate: 30 }
      },
      yAxis: { type: 'value' },
      series: [{
        type: 'bar',
        label: { show: true, position: 'top', formatter: '{c}' },
        data: [
          { value: 120, hand: t('intro223'), itemStyle: { color: '#2080f0' } },
          { value: 200, hand: t('intro223'), itemStyle: { color: '#2080f0' } },
          { value: 150, hand: t('intro223'), itemStyle: { color: '#2080f0' } },
          { value: 80, hand: t('intro223'), itemStyle: { color: '#2080f0' } },
          { value: 70, hand: t('intro229'), itemStyle: { color: '#18a058' } },
          { value: 70, hand: t('intro224'), itemStyle: { color: '#d03050' } },
          { value: 140, hand: t('intro224'), itemStyle: { color: '#d03050' } },
          { value: 190, hand: t('intro224'), itemStyle: { color: '#d03050' } },
          { value: 110, hand: t('intro224'), itemStyle: { color: '#d03050' } }
        ]
      }]
    }
  ];
}
// 默认值，可以从配置中读取更新
let fingerKeyMap = {
  leftPinky: ['Q', 'A', 'Z', '1', '`', 'Tab', 'CapsLock', 'LControl', 'LShift', 'LAlt'],
  leftRing: ['W', 'S', 'X', '2'],
  leftMiddle: ['E', 'D', 'C', '3'],
  leftIndex: ['R', 'T', 'F', 'G', 'V', 'B', '4', '5'],
  rightIndex: ['Y', 'U', 'H', 'J', 'N', 'M', '6', '7'],
  rightMiddle: ['I', 'K', ',', '8'],
  rightRing: ['O', 'L', '.', '9'],
  rightPinky: ['P', ';', "'", '[', ']', '\\', '-', '=', '/', 'Backspace', 'Enter', '0', 'RControl', 'RShift', 'RAlt'],
  thumb: ['Space', 'LWin', 'RWin']
}

// 鼠标按键映射
let mouseKeyMap = {
  // 右手左手鼠标
  thumb: ['XButton1', 'XButton2'],  // 拇指：侧键
  index: ['LButton'],      // 食指：左键 
  middle: ['RButton', 'WheelUp', 'WheelDown', 'MButton'], // 中指：右键 滚轮
}

// 默认的键名列表
const defaultFingerKeyNames = ['leftPinky', 'leftRing', 'leftMiddle', 'leftIndex', 'rightIndex', 'rightMiddle', 'rightRing', 'rightPinky', 'thumb']
const defaultMouseKeyNames = ['thumb', 'index', 'middle']

function matchKey(keyName, matchList) {
  const upperKey = keyName.toUpperCase()
  return matchList.some(k => k.toUpperCase() === upperKey)
}
// 获取所有鼠标按键名称
function getMouseKeys(): string[] {
  return [...new Set(Object.values(mouseKeyMap).flat())].map(k => k.toUpperCase());
}
// 根据keymap获取每一行的按键清单形成数组，去除重复的符号，keymap是二维数组
// [19, 1, '1', 'Numpad1 NumpadEnd'] ,第一列为列号，第二列为行号，第三列为按键，但当有第四列时，按键以第四列优先
// 第四列有可能有多个按键用空格分隔
// mouse == 0 时排除鼠标按键
// 返回: { 行号: [所有按键...] }
function getRowKeyMap(keyMap: any[], mouse: number = 1): Record<string, string[]> {
  const rowMap: Record<string, Set<string>> = {};
  const mouseKeys = mouse === 0 ? new Set(getMouseKeys()) : new Set<string>();
  keyMap.forEach(item => {
    if (!item || item.length < 3) return;
    const rowIndex = String(item[1] + 1); // 第二列为行号
    const keyName = item[2];
    const priorityKey = item[3];

    if (!rowMap[rowIndex]) {
      rowMap[rowIndex] = new Set();
    }
    // 有第四列优先使用，否则用第三列
    const addKeys = priorityKey
      ? priorityKey.split(' ').filter(k => k.trim())
      : (keyName ? [keyName.trim()] : []);

    addKeys.forEach(k => {
      if (!mouseKeys.has(k.toUpperCase())) {
        rowMap[rowIndex].add(k);
      }
    });
  });
  // 转换为数组，按行号排序，过滤空数组
  const result: Record<string, string[]> = {};
  Object.keys(rowMap).sort((a, b) => Number(a) - Number(b)).forEach(row => {
    const arr = Array.from(rowMap[row]);
    if (arr.length > 0) {
      result[row] = arr;
    }
  });
  return result;
}
// 根据行号返回该行的按键清单，每5个换行
function listRowKey(rowIndex: string): string {
  const keys = rowKeyMap[rowIndex] || [];
  if (keys.length === 0) return '';
  const grouped: string[] = [];
  for (let i = 0; i < keys.length; i += 5) {
    grouped.push(keys.slice(i, i + 5).join(', '));
  }
  return grouped.join('<br/>');
}
function calculateStats(allKey, keyStatHash, mouse, keyMap) {
  let fingerStats = {}, handStats = { left: 0, right: 0, thumb: 0 }
  let rowStats = {}, totalCount = 0

  // 复制 fingerKeyMap，不修改原对象
  const effectiveFingerMap: Record<string, string[]> = {};
  Object.keys(fingerKeyMap).forEach(finger => {
    effectiveFingerMap[finger] = [...fingerKeyMap[finger]];
  });

  // 鼠标 == 1 时，将鼠标按键添加到对应的手指
  if (mouse === 1) {
    effectiveFingerMap.thumb = [...effectiveFingerMap.thumb, ...mouseKeyMap.thumb];
    effectiveFingerMap.rightIndex = [...effectiveFingerMap.rightIndex, ...mouseKeyMap.index];
    effectiveFingerMap.rightMiddle = [...effectiveFingerMap.rightMiddle, ...mouseKeyMap.middle];
  } else if (mouse === 2) {
    effectiveFingerMap.thumb = [...effectiveFingerMap.thumb, ...mouseKeyMap.thumb];
    effectiveFingerMap.leftIndex = [...effectiveFingerMap.leftIndex, ...mouseKeyMap.index];
    effectiveFingerMap.leftMiddle = [...effectiveFingerMap.leftMiddle, ...mouseKeyMap.middle];
  }

  rowKeyMap = getRowKeyMap(keyMap, mouse) // 需要根据键盘计算
  Object.keys(effectiveFingerMap).forEach(finger => { fingerStats[finger] = 0 })
  Object.keys(rowKeyMap).forEach(row => { rowStats[row] = 0 })

  allKey.forEach(keyName => {
    let count = keyStatHash[keyName] || 0
    if (count === 0 || keyName.startsWith('App-')) return

    totalCount += count

    Object.keys(effectiveFingerMap).forEach(finger => {
      if (matchKey(keyName, effectiveFingerMap[finger])) {
        fingerStats[finger] += count
        if (finger.includes('left')) handStats.left += count
        else if (finger.includes('right')) handStats.right += count
        else handStats.thumb += count
      }
    })

    // 行统计：匹配到某一行后跳出，避免同一按键被统计到多行
    for (const row of Object.keys(rowKeyMap)) {
      if (matchKey(keyName, rowKeyMap[row])) {
        rowStats[row] += count
      }
    }
  })

  return { fingerStats, handStats, rowStats, totalCount }
}
function batchSetOption(myChartArr) {
  myChartArr.forEach((chart, i) => {
    chart.setOption(fingerOption[i])
  })
}
// 将父对象的全局变量重新赋值
function setFingerChart() {
  fingerCharts = fingerIndex.map((idx: number) => allCharts[idx])
}
function showFinger(myChartArr, optionArr, chartIndices, allKey, keyStatHash, keyMap, mouse) {
  if (!myChartArr || !optionArr || !allKey || !keyStatHash || !chartIndices) return
  if (chartIndices.length !== 4) return
  allCharts = myChartArr; // 指向对象
  fingerIndex = chartIndices; // 标记
  setFingerChart()

  const { fingerStats, handStats, rowStats, totalCount } = calculateStats(allKey, keyStatHash, mouse, keyMap)
  //fingerOption = getFingerOption()
  // console.log(fingerStats, handStats,rowStats,totalCount)
  chartIndices.forEach((chartIdx, i) => {
    if (chartIdx >= myChartArr.length) return
    let opt = fingerOption[i]
    if (i == 0) {
      opt.series[0].data[0].value = handStats.right
      opt.series[0].data[1].value = handStats.left
    } else if (i == 1) {
      const rowKeys = Object.keys(rowStats).sort((a, b) => Number(a) - Number(b));
      const rowValues = rowKeys.map(k => rowStats[k]);
      opt.yAxis.data = rowKeys
      opt.series[0].data = rowValues
    } else if (i == 2) {
      opt.series[0].data = [fingerStats['leftPinky'], fingerStats['leftRing']
        , fingerStats['leftMiddle'], fingerStats['leftIndex']] // 左边

      opt.series[1].data = [fingerStats['rightPinky'], fingerStats['rightRing']
        , fingerStats['rightMiddle'], fingerStats['rightIndex']]  // 右边
    } else if (i == 3) {
      opt.series[0].data[0].value = fingerStats['leftPinky']
      opt.series[0].data[1].value = fingerStats['leftRing']
      opt.series[0].data[2].value = fingerStats['leftMiddle']
      opt.series[0].data[3].value = fingerStats['leftIndex']
      opt.series[0].data[4].value = fingerStats['thumb']
      opt.series[0].data[5].value = fingerStats['rightIndex']
      opt.series[0].data[6].value = fingerStats['rightMiddle']
      opt.series[0].data[7].value = fingerStats['rightRing']
      opt.series[0].data[8].value = fingerStats['rightPinky']
    } else {

    }
    // 根据数据跳转 opt 的值
    optionArr[chartIdx] = opt // 替换全局option变量内的值
    fingerOption[i] = opt; // 本模块变量也一样要更新
  })
  // 批量设置数据
  batchSetOption(fingerCharts)
}

// 只更新 fingerOption 中的语言相关文本部分 和 getFingerOption 中的内容相匹配
function updateFingerOptionLang() {
  if (fingerOption.length === 0) return;
  const t = (key: string) => (contentText as any)[key] || key;
  setFingerChart()
  // 图表1：饼图
  if (fingerOption[0]) {
    fingerOption[0].title.text = t('intro219');
    fingerOption[0].series[0].data[0].name = t('intro224');
    fingerOption[0].series[0].data[1].name = t('intro223');
  }

  // 图表2：键盘行
  if (fingerOption[1]) {
    fingerOption[1].title.text = t('intro220');
  }

  // 图表3：手指对比
  if (fingerOption[2]) {
    fingerOption[2].title.text = t('intro221');
    fingerOption[2].yAxis.data = [t('intro225'), t('intro226'), t('intro227'), t('intro228')];
    fingerOption[2].series[0].name = t('intro223');
    fingerOption[2].series[1].name = t('intro224');
  }

  // 图表4：各手指
  if (fingerOption[3]) {
    fingerOption[3].title.text = t('intro222');
    const fingerData = [t('intro225'), t('intro226'), t('intro227'), t('intro228'), t('intro229'), t('intro228'), t('intro227'), t('intro226'), t('intro225')];
    fingerOption[3].xAxis.data = fingerData;
    fingerOption[3].series[0].data.forEach((d: any, i: number) => {
      if (i < 4) d.hand = t('intro223');
      else if (i === 4) d.hand = t('intro229');
      else d.hand = t('intro224');
    });
  }

  batchSetOption(fingerCharts);
}

function setLangContent(content: Object) {
  contentText = content;
  if (fingerOption.length == 0) {
    fingerOption = getFingerOption();
  }
  else {
    updateFingerOptionLang();
  }
}
// 更新 fingerKeyMap 配置
function updateFingerMap(config: String) {
  if (config) {
    try {
      const parsed = typeof config === 'string' ? JSON.parse(config) : config;
      Object.keys(parsed).forEach(key => {
        if (fingerKeyMap.hasOwnProperty(key) && Array.isArray(parsed[key])) {
          fingerKeyMap[key] = parsed[key];
        }
      });
    } catch (e) {
      console.error('Failed to parse fingerKeyMap:', e);
    }
  }
}
// 更新 mouseKeyMap 配置
function updateMouseMap(config: any) {
  if (config) {
    try {
      const parsed = typeof config === 'string' ? JSON.parse(config) : config;
      Object.keys(parsed).forEach(key => {
        if (mouseKeyMap.hasOwnProperty(key) && Array.isArray(parsed[key])) {
          mouseKeyMap[key] = parsed[key];
        }
      });
    } catch (e) {
      console.error('Failed to parse mouseKeyMap:', e);
    }
  }
}
export {
  deepCopy, ajax, splitArr, str2Type, setWS, arrRemove, getHistory, getServer,
  getKeyDesc, showLeftKey, railStyle, showAppChart, appPath2Name, closeWS,
  dateFormat, timeFormat, addExtListener, getDbs, setDbSel, minute2Hour, exportToText,
  showFinger, fingerOption, setLangContent, updateFingerMap, updateMouseMap,
  defaultFingerKeyNames, defaultMouseKeyNames, fingerKeyMap, mouseKeyMap
}