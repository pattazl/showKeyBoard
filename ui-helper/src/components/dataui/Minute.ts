
import { MinuteType } from '../../myType.d'
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
import { setWS, arrRemove, getHistory, showLeftKey, railStyle, showAppChart, appPath2Name, closeWS, ajax } from '@/common';

let base = +new Date(1988, 9, 3);
let oneDay = 60 * 1000;

let data = [[base, Math.random() * 300]];
let data1 = [[base, Math.random() * 30000]];
for (let i = 1; i < 2000; i++) {
  let now = new Date((base += oneDay));
  data.push([+now, Math.round((Math.random() - 0.5) * 20 + data[i - 1][1])]);
  data1.push([+now, 100*Math.round((Math.random() - 0.5) * 20 + data[i - 1][1])]);
}

// 返回Option
function getMinuteOption(typeFlag: Array<MinuteType>): Array<any> {
  let optByMinute = {
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
        type: 'value',
        boundaryGap: [0, '100%'],
      },
      {
        // 右侧 Y 轴配置
        type: 'value',
        name: 'Distance(Pixels)',
      },
      ],
    dataZoom: [
      {
        type: 'inside',
        start: 0,
        end: 100
      },
      {
        start: 0,
        end: 100
      }
    ],
    series: [
      {
        name: 'Fake Data',
        type: 'line',
        smooth: true,
        symbol: 'none',
        areaStyle: {},
        data: data,
        yAxisIndex: 0, // 使用左侧 Y 轴
      },
      {
        name: 'Fake Data2',
        type: 'line',
        smooth: true,
        symbol: 'none',
        areaStyle: {},
        data: data1,
        yAxisIndex: 1, // 使用左侧 Y 轴
      }
    ]
  }
  let optByhour = {}
  let optDuration = {}
  let optDistribution = {}
  let optAppByMinute = {}
  let optAppByHour = {}

  let hashMap = {
    [MinuteType.ByMinute] : optByMinute,
    [MinuteType.ByHour] : optByhour,
    [MinuteType.Duration] : optDuration,
    [MinuteType.Distribution] : optDistribution,
    [MinuteType.AppByMinute] : optAppByMinute,
    [MinuteType.AppByHour] : optAppByHour,
  }
  let optionArr = []
  typeFlag.forEach(x=>{
    optionArr.push( hashMap[x] )
  })
  return optionArr
}
async function setMinuteEcharts(beginDate: string, endDate: string, typeFlag: MinuteType, chart: echarts.ECharts) {
  if (typeFlag == MinuteType.ByMinute) {
    let data = await ajax('minuteData', { beginDate, endDate, freqType: 0, isApp: false })
    // let data3 = await ajax('minuteData',{beginDate:strDay, endDate:strDay,freqType:0,isApp:true})
    let opt = getMinuteOption([typeFlag])[0]
    chart.setOption(<any>opt)
  }
}

export { getMinuteOption, setMinuteEcharts }