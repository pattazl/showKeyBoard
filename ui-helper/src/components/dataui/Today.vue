<template>
	<div>
		<n-h3>Message</n-h3>
		<n-button type="primary" @click="handleShowMessage">
			今天数据
		</n-button>
		<div id="main" style="height: 500px; min-width:  800px;width:95%; "></div>
    <div>剩余未定义数据</div>
    <n-input
      :value="strLeftKeyVal"
      type="textarea"
      placeholder="未定义数据"
      show-count
      size="large"
      rows="10"
      style="width: 400px;"
    />
	</div>
</template>

<script lang="ts">
import { useAustinStore } from '../../App.vue'
import { defineComponent, onMounted, PropType, ref, computed } from 'vue'
import { useMessage } from 'naive-ui'
// 引入 echarts 核心模块，核心模块提供了 echarts 使用必须要的接口。
import * as echarts from 'echarts/core';
// 引入柱状图图表，图表后缀都为 Chart
import { BarChart,HeatmapChart,HeatmapSeriesOption } from 'echarts/charts';
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
import { setWS,arrRemove } from '@/common';

// 注册必须的组件
echarts.use([
  TitleComponent,
  TooltipComponent,
  GridComponent,
  DatasetComponent,
  TransformComponent,
  BarChart,
  LabelLayout,
  UniversalTransition,
  CanvasRenderer,
  HeatmapChart,
  VisualMapComponent
]);


var option;
// prettier-ignore
const hours = [

];
// prettier-ignore
const days = [
    
];
// prettier-ignore
let hashTxtData = {}; // 按键上显示的内容
let hashOriData = {}; // 原始定义的内容，提示框上显示
const keyData = [
[0,0,"LControl"],
[1,0,"LWin"],
[2,0,"LAlt"],
[3,0,"Space"],
[4,0,"Space"],
[5,0,"Space"],
[6,0,"Space"],
[7,0,"Space"],
[8,0,"Space"],
[9,0,"Space"],
[10,0,"Space"],
[11,0,"RAlt"],
[12,0,"RWin"],
[13,0,"RControl"],
[14,0,],
[15,0,"Left"],
[16,0,"Down"],
[17,0,"Right"],

[0,1,"LShift"],
[1,1,"LShift"],
[2,1,"Z"],
[3,1,"X"],
[4,1,"C"],
[5,1,"V"],
[6,1,"B"],
[7,1,"N"],
[8,1,"M"],
[9,1,"<,"],
[10,1,">."],
[11,1,"?/"],
[12,1,"RShift"],
[13,1,"RShift"],
[14,1,],
[15,1,],
[16,1,"Up"],
[17,1,],

[0,2,"CapsLock"],
[1,2,"CapsLock"],
[2,2,"A"],
[3,2,"S"],
[4,2,"D"],
[5,2,"F"],
[6,2,"G"],
[7,2,"H"],
[8,2,"J"],
[9,2,"K"],
[10,2,"L"],
[11,2,":;"],
[12,2,"\"'"],
[13,2,"Enter"],
[14,2,],
[15,2,],
[16,2,],
[17,2,],

[0,3,"Tab"],
[1,3,"Q"],
[2,3,"W"],
[3,3,"E"],
[4,3,"R"],
[5,3,"T"],
[6,3,"Y"],
[7,3,"U"],
[8,3,"I"],
[9,3,"O"],
[10,3,"P"],
[11,3,"{["],
[12,3,"}]"],
[13,3,"|\\"],
[14,3,],
[15,3,"Delete"],
[16,3,"End"],
[17,3,"Pgdn"],

[0,4,"~`"],
[1,4,"!1"],
[2,4,"@2"],
[3,4,"#3"],
[4,4,"$4"],
[5,4,"%5"],
[6,4,"^6"],
[7,4,"&7"],
[8,4,"*8"],
[9,4,"(9"],
[10,4,")0"],
[11,4,"_-"],
[12,4,"+="],
[13,4,"Backspace"],
[14,4,],
[15,4,"Insert"],
[16,4,"Home"],
[17,4,"Pgup"],

[0,5,"Escape"],
[1,5,],
[2,5,"F1"],
[3,5,"F2"],
[4,5,"F3"],
[5,5,"F4"],
[6,5,"F5"],
[7,5,"F6"],
[8,5,"F7"],
[9,5,"F8"],
[10,5,"F9"],
[11,5,"F10"],
[12,5,"F11"],
[13,5,"F12"],
[14,5,],
[15,5,"PrintScreen"],
[16,5,"Insert"],
[17,5,"Pause"],

[2,6,"LButton"],
[4,6,"RButton"],
[6,6,"MButton"],
[8,6,"WheelDown"],
[10,6,"WheelUp"],
];

//console.log(data);
option = {
  tooltip: {
    position: 'top'
  },
  grid: {
    height: '50%',
    top: '10%'
  },
  xAxis: {
    type: 'category',
    data: hours,
    splitArea: {
      show: true
    }
  },
  yAxis: {
    type: 'category',
    data: days,
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
    bottom: '15%'
  },
  series: [
    {
      tooltip: {
        trigger: 'item',
        confine: true,
        formatter: (p) => {
          //自定义提示信息
          //console.log(p);
          let dataCon = p.data;
          let key = dataCon[0]+','+dataCon[1]
          let txtCon = hashOriData[key]+'<hr> '+dataCon[2] ;
          return txtCon;
        }
      },
      name: 'Keyboard',
      type: 'heatmap',
      data: {},
      label: {
        show: true,
        formatter: function (params) {
          let txt = hashTxtData[params.data[0]+','+params.data[1]]
          if (txt==null) {
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
function getKeyVal(key, keyStatHash, leftKey) {
  let val;
  do {
    val = keyStatHash[key];
    if (val != null) break;
    // 如果1个字母的，没匹配到，尝试匹配小写
    if (key.length == 1) {
      key = key.toLowerCase()
      val = keyStatHash[key]
      if (val != null) break;
    }
    // 如果还没匹配到，查看是否非字母
    if (key.length == 2) {
      key = key.substring(1, 2)
      val = keyStatHash[key]
    }
    if (val != null) break;
  }while(false)  // 只循环一次
  if (val != null){
    arrRemove(leftKey, key)
  }else{
    val = 0;    // 默认没有找到匹配，数据 0
  }
  return val
}
export default defineComponent({
	name: 'Today',
	setup() {
    const store = useAustinStore();
    const keyList = (<any>store.preData).keyList;
    let chartDom,myChart;
    let lastUpdateTick = 0
    let leftKeyVal = []
    let strLeftKeyVal= ref('') ;
    function updateKeyData(msg){
      if(msg.indexOf('{"') != 0){
        // 不是 JSON，直接退出
        return;
      }
      // 不必每次都刷新数据，可以时间间隔可以为2秒
      let nowTick = new Date().getTime();
      if( (nowTick-lastUpdateTick) < 1000)
      {
        return;
      }else{
        lastUpdateTick = nowTick;
      }
      let keyStatHash = JSON.parse(msg)
      let keyArr = []  // 已经统计的数据清单
      let leftKey = Object.keys(keyStatHash)  // 剩余的匹配清单
      option.series[0].data =  keyData.map(function (item) {
        let val:string|number = 0 ,key:string;
        if(item[2] ==null)
        {
          val = '-'
        }else{
          key = item[2].toString()
          val = getKeyVal(key,keyStatHash,leftKey)
        }
        // 用于产生显示在界面的文字内容
        hashTxtData[item[0] +','+item[1]] = keyList[key]??item[2]
        hashOriData[item[0] +','+item[1]] = item[2]
        // 将val 数据全部放到数组中，同于统计 max值
        keyArr.push(val)
        return [item[0], item[1], val];
      });
      if(keyArr.length>5){
        let arr = keyArr.sort((a, b) => b - a)
        option.visualMap.max = Math.max(arr[3],10) // 第4个
      }
			option && myChart.setOption(option);
      // 显示未统计进去的数据 leftKey
      let leftHash ={};
      arrRemove(leftKey,'tick') ; // 去掉
      leftKey.sort( (a, b) => keyStatHash[b] - keyStatHash[a])  // 排序
      leftKeyVal = []
      leftKey.forEach( k => leftKeyVal.push(k + ' : ' + keyStatHash[k] ))
      console.log(leftKeyVal)
      strLeftKeyVal.value = leftKeyVal.join('\n')
    }
		onMounted(() => {
			chartDom = document.getElementById('main');
			myChart = echarts.init(chartDom);
      //console.log(keyList)
      setWS(updateKeyData)
		})

		const message = useMessage()
		const handleShowMessage = () => {
      console.log(option.series.data)
      //option.series[0].data = data2
      //myChart.setOption(option);
			message.success('I can use message')
		}
		return {
			handleShowMessage,
      strLeftKeyVal,
		}
	},
})
</script>
<style scoped>

</style>