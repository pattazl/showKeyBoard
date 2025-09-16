
import { MinuteType, MinuteData, MinuteAppData, seriesData } from '../../myType.d'
import dayjs from 'dayjs'
import { defineComponent, onMounted, PropType, ref, computed, h, watch, onUnmounted } from 'vue'
import { useMessage, NTag } from 'naive-ui'
// 引入 echarts 核心模块，核心模块提供了 echarts 使用必须要的接口。
import * as echarts from 'echarts/core';
import { deepCopy, arrRemove, getHistory, showLeftKey, railStyle, showAppChart, appPath2Name, closeWS, ajax,minute2Hour } from '@/common';

let appNameListMap = {}  // 由外部数据改变
let optByMinute = {
  title: {
    text: '',
    // subtext: '纯属虚构',
  },
  aria: {
    enabled: true,
    show: true,
    decal: {
      show: true,
      decals: {
        symbol: 'react'
      }
    }
  },
  tooltip: {
    trigger: 'axis',
    axisPointer: {
      type: 'cross'
    }
  },
  legend: {},
  toolbox: {
    feature: {
      dataZoom: {
        yAxisIndex: 'none'
      },
      restore: {},
      saveAsImage: {}
    }
  },
  xAxis: {
    type: 'time',
    boundaryGap: false
  },
  yAxis: [
    {
      // 左侧 Y 轴配置
      name: 'Count',
      type: 'value',
      boundaryGap: [0, 0.2],
    },
    {
      // 右侧 Y 轴配置
      type: 'value',
      name: 'Distance(Pixels)',
    },
  ],
  dataZoom: [
    {
      start: 0,
      end: 100
    }
  ],
  series: [
    {
      name: 'Keystroke',
      type: 'line',
      smooth: true,
      symbol: 'none',
      // areaStyle: {},
      data: [],
      yAxisIndex: 0, // 使用左侧 Y 轴
    },
    {
      name: 'Mouse',
      type: 'line',
      smooth: true,
      symbol: 'none',
      data: [],
      yAxisIndex: 0, // 使用左侧 Y 轴
    },
    {
      name: 'Distance',
      type: 'line',
      smooth: true,
      symbol: 'none',
      data: [],
      yAxisIndex: 1, // 使用右侧 Y 轴
    }
  ]
}
let optByhour = deepCopy(optByMinute)

let optDuration = {
  title: {
    // text: '同名数量统计',
    // subtext: '纯属虚构',
    left: 'center'
  },
  tooltip: {
    trigger: 'item',
    confine: true, // 防止tooltip溢出图表
    // 关键样式：限制宽度并自动换行
    extraCssText: 'max-width: 400px; white-space: normal; word-wrap: break-word;',
    formatter: function (params) {
      // 'Click to show detail <br/>{b}: {c} ({d}%)'  // percent name value
      let strHours = minute2Hour(params.value)
      return `${params.name} <br/> ${params.value} ${strHours} (${params.percent}%) <span style='color:blue'>Click to show detail<span>`
    }
  },
  legend: {
    type: 'scroll',
    orient: 'vertical',
    right: 10,
    top: 20,
    bottom: 20,
    data: ['a', 'b'],
    formatter: function (name) {
      return appPath2Name(name, appNameListMap)  // 使用 formatter 自定义图例中的名字格式
    }
  },
  series: [
    {
      name: 'App',
      type: 'pie',
      radius: '75%',
      center: ['40%', '50%'],
      data: [{ name: 'a', value: 12 }, { name: 'b', value: 50 }],
      label: {
        formatter: function (value) {
          return appPath2Name(value.name, appNameListMap)  // 使用 formatter 自定义图例中的名字格式
        }
      },
      emphasis: {
        itemStyle: {
          shadowBlur: 10,
          shadowOffsetX: 0,
          shadowColor: 'rgba(0, 0, 0, 0.5)'
        }
      }
    }
  ]
}
let optAppByMinute = deepCopy(optByMinute)
// 去掉 Distance坐标
optAppByMinute.series.pop();
optAppByMinute.yAxis.pop();
optAppByMinute.yAxis[0].boundaryGap = false
optAppByMinute.title.top = 0; // 设置标题靠顶部
optAppByMinute.legend.top = 30

let optAppByHour = deepCopy(optByMinute)
optAppByHour.yAxis[1].name = 'Actived Minutes per Hour';
optAppByHour.series[2].name = 'Actived Minutes';
optAppByHour.title.top = 0; // 设置标题靠顶部
optAppByHour.legend.top = 30
// 返回Option
function getMinuteOption(typeFlag: Array<MinuteType>): Array<any> {
  let hashMap = {
    [MinuteType.ByMinute]: optByMinute,
    [MinuteType.ByHour]: optByhour,
    [MinuteType.Duration]: optDuration,
    [MinuteType.AppByMinute]: optAppByMinute,
    [MinuteType.AppByHour]: optAppByHour,
  }
  let optionArr = []
  typeFlag.forEach(x => {
    optionArr.push(hashMap[x])
  })
  return optionArr
}
const oneMinute = 60 * 1000; // 一分钟毫秒数
const oneHour = 60 * 60 * 1000; // 一小时毫秒数
let lastAppData: Array<MinuteAppData> = []; // 上一次获取App数据的时间
let lastAppHourData: Array<MinuteAppData> = []; // 上一次获取App小时数据的时间
let appInfoList: Array<seriesData> = [] // APP使用数据清单
let chartAppMinute, chartAppDur, chartAppHour;  // 应用分钟图表，应用耗时图表,应用小时图表
async function setMinuteEcharts(beginDate: string, endDate: string, typeFlag: MinuteType, charts: Array<echarts.ECharts>, applist: any, noLoad: Boolean = false) {
  appNameListMap = applist
  let data: Array<MinuteData> = []
  switch (typeFlag) {
    case MinuteType.ByMinute:
      // let data3 = await ajax('minuteData',{beginDate:strDay, endDate:strDay,freqType:0,isApp:true})
      data = await ajax('minuteData', { beginDate, endDate, freqType: 0, isApp: false })
      setChartByMinute(data, charts[0])
      break;
    case MinuteType.Duration:
      chartAppHour = charts[2]
      chartAppMinute = charts[1]
      chartAppDur = charts[0]
      bindAppCharts()  // 绑定事件
      if (!noLoad) {
        lastAppData = await ajax('minuteData', { beginDate, endDate, freqType: 0, isApp: true })
        lastAppHourData = await ajax('minuteData', { beginDate, endDate, freqType: 1, isApp: true })
      }
      setChartByDura()
      break;
    case MinuteType.ByHour:
      data = await ajax('minuteData', { beginDate, endDate, freqType: 1, isApp: false })
      setChartByHour(data, charts[0])
      break;
  }

}
// 根据分钟设置参数
function setChartByMinute(data, charts) {
  // let today = new Date();
  // let now = new Date().getTime()
  // // 将时分秒设置为 0
  // today.setHours(0, 0, 0, 0);
  // 转换为每分钟的hash数据
  let hash = {}, firstDate, firstMinute = 0, lastMinute = 0;
  data.forEach(x => {
    let minute = dayjs(x.Minute, 'YYYYMMDDHHmm').valueOf()
    if (firstMinute == 0) {
      firstMinute = minute
      firstDate = x.Date
    }
    lastMinute = minute
    hash[minute] = x
  })
  // let base = today.getTime()
  //let firstDay = dayjs(firstDate, dateFormat).valueOf()
  lastMinute += oneMinute  // 需要增加1单元以便于显示最后一个
  let keyData = [], mouseData = [], disData = [];
  let afterMinute = 0;// 延后10分钟
  let totalMs = lastMinute - firstMinute + afterMinute * 60 * 1000
  let totalMinute = totalMs / 1000 / 60

  for (let i = 0; i < totalMinute; i++) {
    let now = +new Date(firstMinute + oneMinute * i);
    keyData.push([now, hash[now]?.KeyCount ?? 0]);
    mouseData.push([now, hash[now]?.MouseCount ?? 0]);
    disData.push([now, hash[now]?.Distance ?? 0]);
  }
  let opt = getMinuteOption([MinuteType.ByMinute])[0]
  opt.series[0].data = keyData
  opt.series[1].data = mouseData
  opt.series[2].data = disData
  // 最近时间的前1小时，后10分钟
  let startPer = ((lastMinute - firstMinute - 60 * oneMinute) / totalMs) * 100
  let endPer = ((lastMinute + afterMinute * oneMinute - firstMinute) / totalMs) * 100
  opt.dataZoom[0].start = startPer
  opt.dataZoom[0].end = endPer
  charts.setOption(<any>opt)
}
// 根据小时设置参数
function setChartByHour(data, charts) {
  let hash = {}, firstHour = 0, lastHour = 0;
  data.forEach(x => {
    let minute = dayjs(x.Minute, 'YYYYMMDDHH').valueOf()
    if (firstHour == 0) {
      firstHour = minute
    }
    lastHour = minute
    hash[minute] = x
  })
  // let base = today.getTime()
  //let firstDay = dayjs(firstDate, dateFormat).valueOf()
  lastHour += oneHour  // 需要增加1单元以便于显示最后一个
  let keyData = [], mouseData = [], disData = [];
  let totalMs = lastHour - firstHour
  let totalHour = totalMs / 1000 / 60 / 60

  for (let i = 0; i < totalHour; i++) {
    let now = +new Date(firstHour + oneHour * i);
    keyData.push([now, hash[now]?.KeyCount ?? 0]);
    mouseData.push([now, hash[now]?.MouseCount ?? 0]);
    disData.push([now, hash[now]?.Distance ?? 0]);
  }
  let opt = getMinuteOption([MinuteType.ByHour])[0]
  opt.series[0].data = keyData
  opt.series[1].data = mouseData
  opt.series[2].data = disData
  // 最近时间的前24小时
  let startPer = ((lastHour - firstHour - 24 * oneHour) / totalMs) * 100
  let endPer = 100
  opt.dataZoom[0].start = startPer
  opt.dataZoom[0].end = endPer
  charts.setOption(<any>opt)
}
// 设置时长相关界面和功能
function setChartByDura() {
  let hash = {} // 记录APP的出现次数，既为分钟数
  let firstMinuteDate = '' // 分钟数据的第一天，需要和小时数据结合起来
  let hashMinites = {} // 用于计算每分钟的应用数量，从而统计分钟占比
  lastAppData.forEach(x => {
    let minute = x.Minute
    if (hashMinites[minute] == null) {
      hashMinites[minute] = 0
    }
    hashMinites[minute] += 1  // 统计每分钟的应用计数，用于计算分钟占用时间
  })
  lastAppData.forEach(x => {
    if (firstMinuteDate == '') {
      firstMinuteDate = x.Date  // lastAppData 是按日期排序好的
    }
    let app = x.Apps
    if (hash[app] == null) {
      hash[app] = 0
    }
    // 更新新的统计算法，如果一分钟内有多个应用激活，那么多个应用将平分这一分钟时间
    hash[app] += 1 / hashMinites[x.Minute]  // 每个应用每分钟的占用时间
  })
  // lastAppHourData 数据也要参与其中共同组合为 应用统计列表
  lastAppHourData.forEach(x => {
    let app = x.Apps
    if (x.Date < firstMinuteDate)  // 小时数据小于分钟的日期，才计算
    {
      if (hash[app] == null) {
        hash[app] = 0
      }
      hash[app] += x.Duration  // 累计每小时的分钟数
    }
  })
  // 得到最终的应用列表数
  let appList = Object.keys(hash)
  appInfoList = []
  appList.forEach(x => {
    appInfoList.push({ name: x, value: Math.round(hash[x] * 10) / 10 }) // 保留小数后一位即可
  })
  // 可以对 appInfoList 排序
  appInfoList.sort((x, y) => y.value - x.value)
  showAppDuration([1, appInfoList.length])
  // typeFlag == MinuteType.AppByMinute 同步处理应用分钟数据
}
// 根据 range 筛选内容 
function showAppDuration(range) {
  let newInfoList = appInfoList.slice(range[0] - 1, range[1])
  let opt = getMinuteOption([MinuteType.Duration])[0]
  let appList = newInfoList.map(x => {
    return x.name
  })
  opt.legend.data = appList
  opt.series[0].data = newInfoList
  chartAppDur.setOption(<any>opt)
  showAppMinute(appList[0]) // 显示第一个
  showAppHour(appList[0])  // 显示第一个
}
// 绑定 chart的事件
function bindAppCharts() {
  // console.log('hasBind', b1['hasBind'])
  if (chartAppDur['hasBind'] == null) {
    // 需要防止重复绑定
    chartAppDur.on('click', function (params) {
      // 控制台打印数据的名称
      showAppMinute(params.name);
      showAppHour(params.name)  // 显示小时统计
    });
    chartAppDur['hasBind'] = true
  }
}
// 根据应用名操作应用使用分钟数 从 lastAppData 中获取数据 [{"Apps":"C:\\Windows\\explorer.exe","KeyCount":0,"Minute":"202312030000","MouseCount":3,"Date":"2023-12-03"},...]
function showAppMinute(appName) {
  if (chartAppMinute == null) {
    console.log('showAppMinute fail')
    return
  }
  // let today = new Date();
  // let strToday = dayjs(today).format(dateFormat)
  // let now = new Date().getTime()
  // 将时分秒设置为 0
  // today.setHours(0, 0, 0, 0);
  let hash = {}, firstMinute = 0, lastMinute = 0;
  lastAppData.forEach(x => {
    if (appName == x.Apps) {
      let minute = dayjs(x.Minute, 'YYYYMMDDHHmm').valueOf()
      if (firstMinute == 0) {
        firstMinute = minute
      }
      lastMinute = minute + oneMinute // 可能需要增加1分钟，否则最后一条可能看不到
      hash[minute] = x
    }
  })
  // console.log(lastAppData)
  // console.log(lastMinute)

  // let firstDay = dayjs(firstDate, dateFormat).valueOf()
  let keyData = [], mouseData = [];
  let afterMinute = 0;// 延后10分钟
  let totalMs = lastMinute - firstMinute + afterMinute * 60 * 1000
  let totalMinute = totalMs / 1000 / 60
  for (let i = 0; i < totalMinute; i++) {
    let now = +new Date(firstMinute + oneMinute * i);
    keyData.push([now, hash[now]?.KeyCount ?? 0]);
    mouseData.push([now, hash[now]?.MouseCount ?? 0]);
  }
  let opt = getMinuteOption([MinuteType.AppByMinute])[0]
  opt.series[0].data = keyData
  opt.series[1].data = mouseData
  // 当前时间的前1小时，后10分钟
  let startPer = ((lastMinute - firstMinute - 60 * oneMinute) / totalMs) * 100
  let endPer = ((lastMinute + afterMinute * oneMinute - firstMinute) / totalMs) * 100
  opt.dataZoom[0].start = startPer
  opt.dataZoom[0].end = endPer
  opt.title.text = appName
  chartAppMinute.setOption(<any>opt, true)
}
// 显示应用小时数据
function showAppHour(appName) {
  if (chartAppHour == null) {
    if (lastAppHourData.length > 0) {
      // 有小时数据，但没显示对象了可以提示
      console.log('showAppHour fail')
    }
    return
  }
  let hash = {}, firstHour = 0, lastHour = 0;
  lastAppHourData.forEach(x => {
    if (appName == x.Apps) {
      let minute = dayjs(x.Minute, 'YYYYMMDDHH').valueOf()
      if (firstHour == 0) {
        firstHour = minute
      }
      lastHour = minute + oneHour // 可能需要增加1小时，否则最后一条可能看不到
      hash[minute] = x
    }
  })
  let keyData = [], mouseData = [], durationData = [];
  let totalMs = lastHour - firstHour
  let totalHour = totalMs / 1000 / 60 / 60
  for (let i = 0; i < totalHour; i++) {
    let now = +new Date(firstHour + oneHour * i);
    keyData.push([now, hash[now]?.KeyCount ?? 0]);
    mouseData.push([now, hash[now]?.MouseCount ?? 0]);
    durationData.push([now, (hash[now]?.Duration ?? 0).toFixed(2)]);
  }
  let opt = getMinuteOption([MinuteType.AppByHour])[0]
  opt.series[0].data = keyData
  opt.series[1].data = mouseData
  opt.series[2].data = durationData
  // 当前时间的前24小时
  let startPer = ((lastHour - firstHour - 24 * oneHour) / totalMs) * 100
  let endPer = 100
  opt.dataZoom[0].start = startPer
  opt.dataZoom[0].end = endPer
  opt.title.text = appName
  chartAppHour.setOption(<any>opt, true)
}

export { getMinuteOption, setMinuteEcharts, bindAppCharts, appInfoList, showAppDuration }