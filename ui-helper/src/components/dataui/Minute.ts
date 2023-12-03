
import { MinuteType, MinuteData, MinuteAppData, seriesData } from '../../myType.d'
import dayjs from 'dayjs'
import { defineComponent, onMounted, PropType, ref, computed, h, watch, onUnmounted } from 'vue'
import { useMessage, NTag } from 'naive-ui'
// 引入 echarts 核心模块，核心模块提供了 echarts 使用必须要的接口。
import * as echarts from 'echarts/core';
// 引入柱状图图表，图表后缀都为 Chart
import { HeatmapChart, BarChart } from 'echarts/charts';
// 引入提示框，标题，直角坐标系，数据集，内置数据转换器组件，组件后缀都为 Component
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  DatasetComponent,
  TransformComponent,
  VisualMapComponent,
  LegendComponent,
} from 'echarts/components';
// 标签自动布局、全局过渡动画等特性
import { LabelLayout, UniversalTransition } from 'echarts/features';
// 引入 Canvas 渲染器，注意引入 CanvasRenderer 或者 SVGRenderer 是必须的一步
import { CanvasRenderer } from 'echarts/renderers';
import { deepCopy, arrRemove, getHistory, showLeftKey, railStyle, showAppChart, appPath2Name, closeWS, ajax } from '@/common';

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
      boundaryGap: [0, '50%'],
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
let optByhour = {}
let optDuration = {
  title: {
    // text: '同名数量统计',
    // subtext: '纯属虚构',
    left: 'center'
  },
  tooltip: {
    trigger: 'item',
    formatter: 'Click to show detail <br/>{b}: {c} ({d}%)'
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
      center: ['40%', '60%'],
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

let optAppByHour = {}
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
const oneMinute = 60 * 1000;
let lastAppTime = 0, lastAppData: Array<MinuteAppData> = null; // 上一次获取App数据的时间
async function setMinuteEcharts(beginDate: string, endDate: string, typeFlag: MinuteType, charts: Array<echarts.ECharts>, applist: any) {
  appNameListMap = applist
  let today = new Date();
  let now = new Date().getTime()
  // 将时分秒设置为 0
  today.setHours(0, 0, 0, 0);
  if (typeFlag == MinuteType.ByMinute) {
    let data: Array<MinuteData> = await ajax('minuteData', { beginDate, endDate, freqType: 0, isApp: false })
    // let data3 = await ajax('minuteData',{beginDate:strDay, endDate:strDay,freqType:0,isApp:true})
    // 转换为每分钟的hash数据
    let hash = {};
    data.forEach(x => {
      let minute = dayjs(x.Minute, 'YYYYMMDDHHmm').valueOf()
      hash[minute] = x
    })
    let base = today.getTime()
    let keyData = [], mouseData = [], disData = [];
    for (let i = 0; i < 24 * 60; i++) {
      let now = +new Date(base + oneMinute * i);
      keyData.push([now, hash[now]?.KeyCount ?? 0]);
      mouseData.push([now, hash[now]?.MouseCount ?? 0]);
      disData.push([now, hash[now]?.Distance ?? 0]);
    }
    let opt = getMinuteOption([typeFlag])[0]
    opt.series[0].data = keyData
    opt.series[1].data = mouseData
    opt.series[2].data = disData
    // 当前时间的前1小时，后10分钟
    let startPer = ((now - base - 60 * oneMinute) / (24 * 60 * 60 * 1000)) * 100
    let endPer = ((now + 10 * oneMinute - base) / (24 * 60 * 60 * 1000)) * 100
    opt.dataZoom[0].start = startPer
    opt.dataZoom[0].end = endPer
    charts[0].setOption(<any>opt)
  }
  if (typeFlag == MinuteType.Duration) {
    let bindChart = charts[1]
    bindCharts(charts[0], bindChart)
    lastAppData = await ajax('minuteData', { beginDate, endDate, freqType: 0, isApp: true })
    let hash = {} // 记录APP的出现次数，既为分钟数
    lastAppData.forEach(x => {
      let app = x.Apps
      if (hash[app] == null) {
        hash[app] = 1
      }
      hash[app] += 1
    })
    let opt = getMinuteOption([typeFlag])[0]
    let appList = Object.keys(hash)
    let appInfoList: Array<seriesData> = []
    appList.forEach(x => {
      appInfoList.push({ name: x, value: hash[x] })
    })
    // 可以对 appInfoList 排序
    appInfoList.sort((x, y) => y.value - x.value)
    appList = appInfoList.map(x => {
      return x.name
    })
    opt.legend.data = appList
    opt.series[0].data = appInfoList
    charts[0].setOption(<any>opt)
    showAppMinute(appList[0], bindChart)
    // typeFlag == MinuteType.AppByMinute 同步处理应用分钟数据
  }
}
// 绑定 chart的事件
function bindCharts(b1, b2) {
  console.log('hasBind', b1['hasBind'])
  if (b1['hasBind'] == null) {
    // 需要防止重复绑定
    b1.on('click', function (params) {
      // 控制台打印数据的名称
      showAppMinute(params.name, b2);
    });
    b1['hasBind'] = true
  }
}
// 根据应用名操作应用使用分钟数 从 lastAppData 中获取数据 [{"Apps":"C:\\Windows\\explorer.exe","KeyCount":0,"Minute":"202312030000","MouseCount":3,"Date":"2023-12-03"},...]
function showAppMinute(appName, chart) {
  if (chart == null || lastAppData == null) {
    console.log('showAppMinute fail:', chart, lastAppData)
    return
  }
  let today = new Date();
  let now = new Date().getTime()
  // 将时分秒设置为 0
  today.setHours(0, 0, 0, 0);
  let hash = {}, lastMinute = 0;
  lastAppData.forEach(x => {
    if (appName == x.Apps) {
      let minute = dayjs(x.Minute, 'YYYYMMDDHHmm').valueOf()
      lastMinute = minute
      hash[minute] = x
    }
  })
  let base = today.getTime()
  let keyData = [], mouseData = [];
  for (let i = 0; i < 24 * 60; i++) {
    let now = +new Date(base + oneMinute * i);
    keyData.push([now, hash[now]?.KeyCount ?? 0]);
    mouseData.push([now, hash[now]?.MouseCount ?? 0]);
  }
  let opt = getMinuteOption([MinuteType.AppByMinute])[0]
  opt.series[0].data = keyData
  opt.series[1].data = mouseData
  // 当前时间的前1小时，后10分钟
  let startPer = ((lastMinute - base - 60 * oneMinute) / (24 * 60 * 60 * 1000)) * 100
  let endPer = ((lastMinute + 10 * oneMinute - base) / (24 * 60 * 60 * 1000)) * 100
  opt.dataZoom[0].start = startPer
  opt.dataZoom[0].end = endPer
  opt.title.top = 0,       // 设置标题靠顶部
    opt.title.text = appName
  opt.legend.top = 30

  chart.setOption(<any>opt, true)
}

export { getMinuteOption, setMinuteEcharts, bindCharts }