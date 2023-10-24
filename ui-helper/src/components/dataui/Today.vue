<template>
  <div>
    <n-space vertical>
      <n-space style="font-size:16px">
        {{ contentText.intro92 }}
        <n-select v-model:value="beginDate" :options="historyDate" @update:value="handleUpdateValue" /> {{
          contentText.intro93 }}
        <n-select v-model:value="endDate" :options="historyDate" @update:value="handleUpdateValue" />
      </n-space>
      <n-card :title="contentText.intro86 + contentText.intro142 + updateTime">
        <div id="main" style="height: 500px; min-width: 800px;width:95%;"></div>
      </n-card>
      <n-card :title="contentText.intro97 + contentText.intro142 + updateTime">
        <n-data-table :columns="columns0" :data="mouseTable" />
      </n-card>
      <n-card :title="contentText.intro87 + contentText.intro142 + updateTime">
        <template #header-extra>
          <n-switch :round="false" :rail-style="railStyle" v-model:value="leftKeySwitch" @update:value="showLeftKey">
            <template #checked>
              合并左右控制键
            </template>
            <template #unchecked>
              左右控制键分别显示
            </template>
          </n-switch>
        </template>
        <n-data-table :columns="columns" :data="dataTable" />
      </n-card>
    </n-space>
  </div>
</template>

<script lang="ts">
import { useAustinStore } from '../../App.vue'
import dayjs from 'dayjs'
import { defineComponent, onMounted, PropType, ref, computed, h, watch, CSSProperties } from 'vue'
import { useMessage, NTag } from 'naive-ui'
// 引入 echarts 核心模块，核心模块提供了 echarts 使用必须要的接口。
import * as echarts from 'echarts/core';
// 引入柱状图图表，图表后缀都为 Chart
import { HeatmapChart, HeatmapSeriesOption } from 'echarts/charts';
// 引入提示框，标题，直角坐标系，数据集，内置数据转换器组件，组件后缀都为 Component
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  DatasetComponent,
  TransformComponent,
  VisualMapComponent
} from 'echarts/components';
// 标签自动布局、全局过渡动画等特性
import { LabelLayout, UniversalTransition } from 'echarts/features';
// 引入 Canvas 渲染器，注意引入 CanvasRenderer 或者 SVGRenderer 是必须的一步
import { CanvasRenderer } from 'echarts/renderers';
import { setWS, arrRemove, getHistory, deepCopy,getKeyDesc } from '@/common';
import content from '../../content.js';
import { Push } from '@vicons/ionicons5';
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
  VisualMapComponent
]);

// prettier-ignore
let hashTxtData = {}; // 按键上显示的内容
let hashOriData = {}; // 原始定义的内容，提示框上显示
let currTick = 0; // 当前msg中的时间戳
let historyData = []
let tickSet = new Set();
let keyData = [];

var option = {
  dark: true,
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
// 获取今天的全部启动次数信息
async function getTodayData(historyDate, contentText) {
  let str = dayjs(new Date()).format('YYYY-MM-DD')
  historyData = await getHistory(str, str)
  historyData.forEach(x => tickSet.add(x.tick))
  let hasNow = false
  tickSet.forEach(x => {
    let mark = ''
    if (x == currTick) {
      mark = '(' + contentText.intro91 + ')'
      hasNow = true
    }
    historyDate.push({ label: dayjs(new Date(<number>x)).format('YYYY-MM-DD HH:mm:ss.SSS') + mark, value: x })
  })
  // 如果数据中没有当前数据，需要强行插入一个
  if (!hasNow && currTick > 0) {
    historyDate.push({ label: dayjs(new Date(<number>currTick)).format('YYYY-MM-DD HH:mm:ss.SSS') + '(' + contentText.intro91 + ')', value: currTick })
  }
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
// 合并 hash统计数据,返回 targetHash
function mergeHash(targetHash/**out */, srcHash) {
  for (let k in targetHash) {
    if (srcHash[k] != null) {
      //需要相加合并
      targetHash[k] += srcHash[k]
    }
  }
  // target 中不存在的
  for (let k in srcHash) {
    if (targetHash[k] == null) {
      targetHash[k] = srcHash[k]
    }
  }
}
// 根据选择的时间进行计算
function getRealStatHash(oriHash, begin, end) {
  if (begin > end || tickSet.size == 0) return oriHash; // 时间不对直接返回原始的
  let hash = oriHash
  tickSet.forEach(x => {
    if (x >= begin && x <= end) {
      if (x == oriHash.tick) {
        // 表示是当前的，直接跳过无需再次合并
      } else {
        // 进行数据合并
        mergeHash(hash, getHash(x))
      }
    }
  })
  return hash;
}
export default defineComponent({
  name: 'Today',
  props: {
    lang: {
      type: String as PropType<'en-US' | 'zh-CN'>,
    },
  },
  setup(props) {
    // reset
    hashTxtData = {}; // 按键上显示的内容
    hashOriData = {}; // 原始定义的内容，提示框上显示
    currTick = 0; // 当前msg中的时间戳
    historyData = []
    tickSet = new Set();

    const contentText = computed(() => content[props.lang])
    const store = useAustinStore();
    const keyList = (<any>store.preData).keyList;
    keyData = JSON.parse((<any>store.preData).dataSetting.mapDetail);
    let chartDom, myChart;
    let lastUpdateTick = 0
    let strLeftKeyVal = ref('');
    let dataTable = ref([])
    let mouseTable = ref([])
    const columns = ref([]);
    const columns0 = ref([]);
    const historyDate = ref([]);
    const beginDate = ref(0);
    const endDate = ref(0);
    const updateTime = ref('');

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
      ],
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
        ]
    }
    function handleUpdateValue() {
      let keyStatHash = getRealStatHash({}, beginDate.value, endDate.value)
      showHash(keyStatHash)
    }
    function updateKeyData(msg) {
      if (msg.indexOf('{"') != 0) {
        // 不是 JSON，直接退出
        return;
      }
      // 有手工选择
      if (endDate.value != currTick && currTick > 0) {
        // 为固定值，直接退出 WS的响应
        return;
      }
      // 不必每次都刷新数据，可以时间间隔可以为2秒
      let nowTick = new Date().getTime();
      if ((nowTick - lastUpdateTick) < 1000) {
        return;
      } else {
        lastUpdateTick = nowTick;
      }
      let keyStatHash = JSON.parse(msg)
      // 设置下拉选择
      if (currTick == 0) {
        currTick = keyStatHash.tick; // 更新当前的tick
        getTodayData(historyDate.value, contentText.value)
        beginDate.value = currTick
        endDate.value = currTick
      }
      keyStatHash = getRealStatHash(keyStatHash, beginDate.value, endDate.value)
      if (keyStatHash['updateTime'] != null) {
        try {
          updateTime.value = dayjs(keyStatHash['updateTime'], 'YYYYMMDDHHmmSS').format('YYYY-MM-DD HH:mm:ss')
        } catch (e) {
          console.log(e)
        }
        delete keyStatHash['updateTime']
      }
      showHash(keyStatHash)
    }
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
      option && myChart.setOption(option);
      // 显示未统计进去的数据 leftKey
      //let leftHash = {};
      arrRemove(leftKey, 'tick'); // 去掉
      arrRemove(leftKey, 'mouseDistance'); // 去掉
      //leftKey.sort((a, b) => keyStatHash[b] - keyStatHash[a])  // 排序
      //let leftKeyVal = []
      //leftKey.forEach(k => leftKeyVal.push(k + ' : ' + keyStatHash[k]))
      //strLeftKeyVal.value = leftKeyVal.join('\n')
      // 需要添加2个，鼠标屏幕移动距离和鼠标物理移动距离 ，每英寸为25.4mm,约 0.0254米
      showLeftKey(leftKey,keyStatHash)
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
    // 显示剩余按键
    let lastLeftKey = [] ,LastKeyStatHash = {} , leftKeySwitch = ref(0);
    function showLeftKey(leftKey,keyStatHash) {
      if(leftKey !=null && keyStatHash!=null){
        lastLeftKey = leftKey
        LastKeyStatHash = keyStatHash
      }else{ // 使用上一次的
        leftKey = lastLeftKey
        keyStatHash = LastKeyStatHash
      }
      // 需要合并 左右控制键
      let tempLeftKey = deepCopy(leftKey)
      let tempKeyStatHash = deepCopy(keyStatHash)
      if(leftKeySwitch.value){ // 合并

      }
      dataTable.value = tempLeftKey.map(function (item) {
        return { keyName: item, count: tempKeyStatHash[item], desc: getKeyDesc(item, contentText.value) }
      })
    }
    onMounted(() => {
      chartDom = document.getElementById('main');
      myChart = echarts.init(chartDom, store.myTheme);
      //console.log(keyList)
      setWS(updateKeyData)
    })
    watch(() => store.myTheme, (newValue, oldValue) => {
      myChart.dispose()
      myChart = echarts.init(chartDom, newValue);
      myChart.setOption(option);
    });
    return {
      strLeftKeyVal,
      columns,
      columns0,
      dataTable,
      contentText,
      historyDate,
      beginDate,
      endDate,
      handleUpdateValue,
      mouseTable,
      updateTime,
      leftKeySwitch,
      showLeftKey,
      railStyle: ({
        focused,
        checked
      }: {
        focused: boolean
        checked: boolean
      }) => {
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
    }
  },
})
</script>
<style scoped>
.n-select {
  width: 300px;
}
</style>