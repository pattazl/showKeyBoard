<template>
  <div>
    <div style="height: 120px ;margin-right: 90px;float: right;">
      <n-anchor affix :top="80" style="z-index: 10; font-size: 18px; " :bound="50" :show-rail="false" :ignore-gap="true"
        type='block' position='fix'>
        <n-anchor-link :title="contentText.intro86" href="#intro86" />
        <n-anchor-link :title="contentText.intro97" href="#intro97" />
        <n-anchor-link :title="contentText.intro149" href="#intro149" />
        <n-anchor-link :title="contentText.intro149" href="#intro149_" />
        <n-anchor-link :title="contentText.intro87" href="#intro87" />
        <n-anchor-link :title="contentText.intro165" href="#intro165" />
        <n-anchor-link :title="contentText.intro166" href="#intro166" />
        <n-anchor-link :title="contentText.intro167" href="#intro167" />
        <n-anchor-link :title="contentText.intro170" href="#intro170" />
        <n-anchor-link :title="contentText.intro171" href="#intro171" />
      </n-anchor>
    </div>
    <n-space vertical class="fixedSelect">
      <n-space style="font-size:16px" :class="store.myTheme == 'dark' ? 'mydark' : 'mylight'">
        {{ contentText.intro112 }}
        <n-date-picker type="date" v-model:value="beginDate" :is-date-disabled="dateDisabled"
          @update:value="handleUpdateValue" />
        <n-switch :round="false" :rail-style="railStyle" v-model:value="showEndDate" @update:value="endDate = beginDate">
          <template #unchecked>
            {{ contentText.intro159 }}
          </template>
          <template #checked>
            {{ contentText.intro160 }}
          </template>
        </n-switch>
        <n-date-picker type="date" v-show="showEndDate" v-model:value="endDate" :is-date-disabled="dateDisabled"
          @update:value="handleUpdateValue" />
      </n-space>
      <n-card id="intro86" :title="contentText.intro86">
        <div id="main1" style="height: 500px; min-width: 800px;width:95%;"></div>
      </n-card>
      <n-card id="intro97" :title="contentText.intro97">
        <n-data-table :columns="columns0" :data="mouseTable" />
      </n-card>
      <n-card id="intro149" :title="contentText.intro149">
        <div id="main2" style="height: 300px; min-width: 800px;width:95%;"></div>
      </n-card>
      <n-card id="intro149_" :title="contentText.intro149">
        <n-data-table :columns="columns2" :data="appListData" />
      </n-card>
      <n-card id="intro87" :title="contentText.intro87">
        <template #header-extra>
          <n-switch :round="false" :rail-style="railStyle" v-model:value="leftKeySwitch" @update:value="showLeftKeyRef">
            <template #checked>
              {{ contentText.intro143 }}
            </template>
            <template #unchecked>
              {{ contentText.intro144 }}
            </template>
          </n-switch>
        </template>
        <n-data-table :columns="columns" :data="dataTable" />
      </n-card>
      <n-card id="intro165"
        :title="contentText.intro165 + ',' + contentText.intro169 + ':' + store.data.dataSetting?.minuteKeepDays">
        <div id="main3" style="height: 500px; min-width: 800px;width:95%;"></div>
      </n-card>
      <n-card id="intro166" :title="contentText.intro166">
        <div id="main4" style="height: 500px; min-width: 800px;width:95%;"></div>
      </n-card>
      <n-card id="intro167" :title="contentText.intro167">
        <n-slider v-model:value="appRanger" range :step="1" :min="1" :max="maxAppLen" @update:value="slideApp"
          :marks="appMarks" />
        <div id="main5" style="height: 500px; min-width: 800px;width:95%;"></div>
      </n-card>
      <n-card id="intro170"
        :title="contentText.intro170 + ',' + contentText.intro169 + ':' + store.data.dataSetting?.minuteKeepDays + contentText.intro168">
        <div id="main6" style="height: 250px; min-width: 800px;width:95%;"></div>
      </n-card>
      <n-card id="intro171" :title="contentText.intro171">
        <div id="main7" style="height: 250px; min-width: 800px;width:95%;"></div>
      </n-card>
    </n-space>
  </div>
</template>

<script lang="ts">
import { useAustinStore } from '../../App.vue'
import dayjs from 'dayjs'
import { defineComponent, onMounted, PropType, ref, computed, h, watch } from 'vue'
import { useMessage, NTag } from 'naive-ui'
// 引入 echarts 核心模块，核心模块提供了 echarts 使用必须要的接口。
import * as echarts from 'echarts/core';
// 引入柱状图图表，图表后缀都为 Chart
import { HeatmapChart, BarChart, PieChart } from 'echarts/charts';
// 引入提示框，标题，直角坐标系，数据集，内置数据转换器组件，组件后缀都为 Component
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  DatasetComponent,
  TransformComponent,
  VisualMapComponent,
  DataZoomComponent,
} from 'echarts/components';
// 标签自动布局、全局过渡动画等特性
import { LabelLayout, UniversalTransition } from 'echarts/features';
import { MinuteType } from '../../myType.d'
// 引入 Canvas 渲染器，注意引入 CanvasRenderer 或者 SVGRenderer 是必须的一步
import { CanvasRenderer } from 'echarts/renderers';
import { arrRemove, getHistory, ajax, showLeftKey, railStyle, showAppChart, appPath2Name, deepCopy, dateFormat } from '@/common';
import content from '../../content.js';
import { setMinuteEcharts, getMinuteOption, appInfoList, showAppDuration } from './Minute';
// 注册必须的组件
echarts.use([
  TitleComponent,
  TooltipComponent,
  GridComponent,
  DatasetComponent,
  TransformComponent,
  LabelLayout,
  UniversalTransition,
  CanvasRenderer,
  HeatmapChart,
  VisualMapComponent,
  DataZoomComponent,
  BarChart,
  PieChart,
]);

// prettier-ignore
let hashTxtData = {}; // 按键上显示的内容
let hashOriData = {}; // 原始定义的内容，提示框上显示
let historyData = []
let keyData = [];
let appNameListMap = {};

var option = {
  textStyle: {
    fontSize: 16
  },
  tooltip: {
    position: 'top'
  },
  grid: {
    height: '80%',
    top: '0%',
  },
  xAxis: {
    type: 'category',
    show: true,
    splitArea: {
      show: true
    }
  },
  yAxis: {
    type: 'category',
    show: true,
    splitArea: {
      show: true
    }
  },
  visualMap: {
    min: 0,
    max: 100,
    calculable: true,
    orient: 'horizontal',
    left: 'center',
    bottom: '2%'
  },
  series: [
    {
      tooltip: {
        trigger: 'item',
        textStyle: {
          fontSize: 18
        },
        confine: true,
        formatter: (p) => {
          //自定义提示信息
          //console.log(p);
          let dataCon = p.data;
          let key = dataCon[0] + ',' + dataCon[1]
          let txtCon = hashOriData[key] + '<hr> ' + dataCon[2];
          return txtCon;
        }
      },
      name: 'Keyboard',
      type: 'heatmap',
      data: {},
      label: {
        show: true,
        formatter: function (params) {
          let txt = hashTxtData[params.data[0] + ',' + params.data[1]]
          if (txt == null) {
            return 'N/A';
          } else {
            return txt;
          }
        }
      },
      emphasis: {
        itemStyle: {
          shadowBlur: 10,
          shadowColor: 'rgba(0, 0, 0, 0.5)'
        }
      }
    }
  ]
};
let option2 = {
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
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true
  },
  xAxis: [
    {
      type: 'category',
      data: ['Mon'],
      axisLabel: {
        interval: 0,
        rotate: 45,
        show: true,
        formatter: function (value) {
          return appPath2Name(value, appNameListMap)
        },
      }
    }
  ],
  yAxis: [
    {
      type: 'value'
    }
  ],
  dataZoom: [
    {
      show: true,
      startValue: 0,
      endValue: 14  // 前15个数据
    }
  ],
  series: [
    {
      name: 'Mouse',
      type: 'bar',
      stack: 'apps',
      emphasis: {
        focus: 'series'
      },
      data: [320]
    },
    {
      name: 'Keyboard',
      type: 'bar',
      stack: 'apps',
      emphasis: {
        focus: 'series'
      },
      data: [120]
    }
  ]
};
// 合并最匹配的键盘统计数据，并整理遗留的数据信息
function getKeyVal(key, mapkey, keyStatHash, leftKey) {
  let val, matchKey;
  do {
    if (mapkey != null) { // 需要支持多个key，用空格分割
      matchKey = mapkey.split(' ')
      let isMatch = false
      val = matchKey.reduce((accumulator, k) => {
        let v = keyStatHash[k];
        if (v != null) {
          isMatch = true;
        } else {
          v = 0
        }
        return accumulator + v
      }, 0)
      if (isMatch) break;
      val = null; //没有匹配上需要清空
    } else {
      // 尝试用 key 
      matchKey = key
      val = keyStatHash[matchKey]
      if (val != null) break;
    }
  } while (false)  // 只循环一次
  if (val != null) {
    arrRemove(leftKey, matchKey)
  } else {
    val = 0;    // 默认没有找到匹配，数据 0
  }
  return val
}

// 获取对应时间的Hash
function getHash(tick) {
  let hash = {}
  for (let v of historyData) {
    if (v.tick == tick) {
      hash[v.keyname] = v.keycount
    }
  }
  return hash
}
export default defineComponent({
  name: 'History',
  props: {
    lang: {
      type: String as PropType<'en-US' | 'zh-CN'>,
    },
  },
  setup(props) {
    // reset
    hashTxtData = {}; // 按键上显示的内容
    hashOriData = {}; // 原始定义的内容，提示框上显示
    historyData = []

    const contentText = computed(() => content[props.lang])
    const store = useAustinStore();
    const keyList = (<any>store.preData).keyList;
    keyData = JSON.parse((<any>store.preData).dataSetting.mapDetail);
    appNameListMap = JSON.parse((<any>store.preData).dataSetting.appNameList);
    let optionArr = [], myChartArr: Array<echarts.ECharts> = [], chartDomArr = [];
    let domNameArr = ['main1', 'main2', 'main3', 'main4', 'main5', 'main6', 'main7'] // ,'main6','main7'
    let strLeftKeyVal = ref('');
    let dataTable = ref([])
    let mouseTable = ref([])
    const columns = ref([]);
    const columns0 = ref([]);
    const historyDate = ref([]);
    const beginDate = ref(0);
    const endDate = ref(0);
    const columns2 = ref([]);
    const appListData = ref([]);
    const showEndDate = ref(0);
    // 显示剩余按键
    const leftKeySwitch = ref(store.data.dataSetting.mergeControl);

    // 获取屏幕像素对角线距离
    const sinfo = store.data.infoPC?.screen; // [{Left:0, Top:0, Right:100, Bottom:200},{Left:0, Top:0, Right:100, Bottom:200}]
    let screenPixlSize = 0;
    if (sinfo != null && sinfo.length > 0 && sinfo[0].Right != null) {
      screenPixlSize = Math.sqrt((sinfo[0].Right - sinfo[0].Left) ** 2 + (sinfo[0].Bottom - sinfo[0].Top) ** 2)
    }

    setColumn(props.lang)
    watch(() => store.lang, (newValue, oldValue) => {
      setColumn(newValue)
    });
    function setColumn(lang) {
      columns0.value = [
        {
          title: content[lang].intro88,
          key: 'keyName',
        },
        {
          title: content[lang].intro98,
          key: 'desc',
        },
        {
          title: content[lang].intro90,
          key: 'count',
          defaultSortOrder: 'descend',
          sorter: 'default',
        }
      ];
      columns.value = [
        {
          title: content[lang].intro88,
          key: 'keyName',
          sorter: 'default',
        },
        {
          title: content[lang].intro89,
          key: 'desc',
          render(row) {
            let arr = row.desc.split(' ');
            let lastChar = arr[arr.length - 1];
            if (lastChar.length == 1) {
              arr[arr.length - 1] = lastChar.toUpperCase()
            }
            const tags = arr.map((tagKey) => {
              return h(
                NTag,
                {
                  style: {
                    marginRight: '6px'
                  },
                  type: 'info',
                  bordered: false
                },
                {
                  default: () => tagKey
                }
              )
            })
            return tags
          }
        },
        {
          title: content[lang].intro90,
          key: 'count',
          defaultSortOrder: 'descend',
          sorter: 'default',
        }
      ];
      // 应用列表数据
      columns2.value = [
        {
          title: content[lang].intro156, // 'AppPath',
          key: 'appPath',
          sorter: 'default'
        },
        {
          title: content[lang].intro90, //'keyCount',
          key: 'keyCount',
          defaultSortOrder: 'descend',
          sorter: (row1, row2) => row1.keyCount - row2.keyCount
        },
        {
          title: content[lang].intro157, // 'keyType',
          key: 'keyType',
          defaultFilterOptionValues: ['Mouse', 'Keyboard'],
          filterOptions: [
            {
              label: 'Mouse',
              value: 'Mouse'
            },
            {
              label: 'Keyboard',
              value: 'Keyboard'
            }
          ],
          filter(value, row) {
            return row.keyType == value.toString()
          }
        }
      ]
    }
    async function handleUpdateValue(value) {
      // console.log(value, beginDate.value, endDate.value)
      let b = beginDate.value, e = b;
      if (showEndDate.value) {
        if (beginDate.value > endDate.value) {
          if (value == endDate.value) {
            endDate.value = beginDate.value
          } else {
            beginDate.value = endDate.value
          }
        }
        b = beginDate.value
        e = endDate.value
      }
      let bs = dayjs(b).format(dateFormat)
      let es = dayjs(e).format(dateFormat)
      historyData = await getHistory(bs, es)
      let keyStatHash = getHash('')
      showHash(keyStatHash)
      updateMinuteData()
    }
    let lastLeftKey = [], LastKeyStatHash = {};
    // 显示数据
    function showHash(keyStatHash) {
      let keyArr = []  // 已经统计的数据清单
      let leftKey = Object.keys(keyStatHash)  // 剩余的匹配清单
      option.series[0].data = keyData.map(function (item) {
        let val: string | number = 0, key: string, keyMap;
        keyMap = item[3]
        if (item[2] == null) {
          val = '-'
        } else {
          key = item[2].toString()  // 界面显示的
          val = getKeyVal(key, keyMap, keyStatHash, leftKey)
        }
        // 用于产生显示在界面的文字内容
        let strKeyMap = '' // 配置了匹配键的
        if (keyMap != null) strKeyMap = ' (' + keyMap + ')'
        hashTxtData[item[0] + ',' + item[1]] = keyList[key] ?? key
        hashOriData[item[0] + ',' + item[1]] = key + strKeyMap
        // 将val 数据全部放到数组中，同于统计 max值
        keyArr.push(val)
        return [item[0], item[1], val];
      });
      if (keyArr.length > 5) {
        let arr = keyArr.sort((a, b) => b - a)
        option.visualMap.max = Math.max(arr[3], 10) // 第4个
      }
      option && myChartArr[0].setOption(option);
      // 显示未统计进去的数据 leftKey
      //let leftHash = {};
      arrRemove(leftKey, 'tick'); // 去掉
      arrRemove(leftKey, 'mouseDistance'); // 去掉
      // 显示 chart2
      appListData.value = showAppChart(leftKey, keyStatHash, option2, myChartArr[1]
        , store.data.dataSetting.mergeAppName ? appNameListMap : null);
      //leftKey.sort((a, b) => keyStatHash[b] - keyStatHash[a])  // 排序
      //let leftKeyVal = []
      //leftKey.forEach(k => leftKeyVal.push(k + ' : ' + keyStatHash[k]))
      //strLeftKeyVal.value = leftKeyVal.join('\n')
      lastLeftKey = leftKey, LastKeyStatHash = keyStatHash;
      dataTable.value = showLeftKey(leftKeySwitch.value, leftKey, keyStatHash)
      // 需要添加2个，鼠标屏幕移动距离和鼠标物理移动距离 ，每英寸为25.4mm,约 0.0254米
      mouseTable.value = []
      if (keyStatHash['mouseDistance'] > 0) {
        let pixel = keyStatHash['mouseDistance']; //获取像素移动距离
        mouseTable.value.push({ keyName: 'mouseDistance', count: Number(pixel), desc: contentText.value.intro94 })
        // 屏幕移动距离 mouseScreenDistance
        let realScreen = 0
        if (screenPixlSize > 0) {
          realScreen = pixel * (store.data.dataSetting.screenSize * 0.0254) / screenPixlSize;
        }
        mouseTable.value.push({ keyName: 'mouseScreenDistance', count: Number(realScreen.toFixed(4)), desc: contentText.value.intro95 })

        // 屏幕移动距离 mousePhysicalDistance
        let realPhysical = 0;
        if (store.data.dataSetting.mouseDPI > 0) {
          realPhysical = pixel * (0.0254) / store.data.dataSetting.mouseDPI;
        }
        mouseTable.value.push({ keyName: 'mousePhysicalDistance', count: Number(realPhysical.toFixed(4)), desc: contentText.value.intro96 })
      }
    }
    function showLeftKeyRef() {
      dataTable.value = showLeftKey(leftKeySwitch.value, lastLeftKey, LastKeyStatHash)
    }
    async function updateMinuteData() {
      let b = beginDate.value, e = endDate.value;
      if (beginDate.value > endDate.value) {
        b = beginDate.value
        e = beginDate.value
      }
      let bs = dayjs(b).format(dateFormat)
      let es = dayjs(e).format(dateFormat)
      // 需要渲染 main1 图表
      setMinuteEcharts(bs, es, MinuteType.ByMinute, [myChartArr[2]], appNameListMap) // main3
      setMinuteEcharts(bs, es, MinuteType.ByHour, [myChartArr[3]], appNameListMap) // main4
      await setMinuteEcharts(bs, es, MinuteType.Duration, [myChartArr[4], myChartArr[5], myChartArr[6]], appNameListMap) // main5 6
      updateAppLenInfo()
    }
    onMounted(async () => {
      myChartArr = []
      chartDomArr = []
      domNameArr.forEach(x => {
        let chartDom = document.getElementById(x);
        let myChart = echarts.init(chartDom, store.myTheme);
        chartDomArr.push(chartDom)
        myChartArr.push(myChart)
      })
      let arr: Array<any> = getMinuteOption([MinuteType.ByMinute, MinuteType.Duration, MinuteType.AppByMinute])
      optionArr = [option, option2].concat(arr)
      // 设置下拉选择
      let dateArr = await ajax('getHistoryDate')
      historyDate.value = dateArr.map((x) => {
        return { label: x, value: x }
      })
      if (dateArr.length > 0) {
        beginDate.value = dayjs(dateArr[0], dateFormat).valueOf();// 设置选择第一个
        handleUpdateValue(dateArr[0])
      }

    })
    // let myTheme = ref()  
    watch(() => store.myTheme, (newValue, oldValue) => {
      domNameArr.forEach((x, i) => {
        let dom = chartDomArr[i]
        myChartArr[i].dispose()
        myChartArr[i] = echarts.init(dom, newValue);
        let opt = optionArr[i]
        if (opt != null) myChartArr[i].setOption(opt);
      })
      updateMinuteData()
    });
    // 更新应用时间区间数据
    let appRanger = ref([]) // 获取的数据
    let appMarks = ref({})
    let maxAppLen = ref(0)
    function updateAppLenInfo() {
      maxAppLen.value = appInfoList.length
      appRanger.value = [1, maxAppLen.value];
      appMarks.value = {
        1: 'Top 1',
        [maxAppLen.value]: 'Top ' + maxAppLen.value
      }
    }
    function slideApp() {
      let arr = deepCopy(appRanger.value);
      if (arr[0] > arr[1]) {
        appRanger.value = [arr[1], arr[0]]
      }
      showAppDuration(appRanger.value)
    }
    function dateDisabled(ts: number) {
      const date = dayjs(ts).format(dateFormat)
      return !historyDate.value.some(x => {
        return x.value == date
      })
    }
    return {
      slideApp,
      appRanger,
      appMarks,
      maxAppLen,
      strLeftKeyVal,
      columns,
      columns0,
      columns2,
      dataTable,
      contentText,
      historyDate,
      beginDate,
      endDate,
      handleUpdateValue,
      mouseTable,
      leftKeySwitch,
      showLeftKeyRef,
      railStyle,
      appListData,
      showEndDate,
      store,
      dateDisabled,
    }
  },
})
</script>
<style scoped>
@import "@/res/common.css";
</style>
