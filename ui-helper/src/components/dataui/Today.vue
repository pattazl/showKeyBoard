<template>
	<div>
		<n-h3>Message</n-h3>
		<n-button type="primary" @click="handleShowMessage">
			今天数据
		</n-button>
		<div id="main" style="height: 400px; width: 800px; border: 1px red solid;"></div>
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
import { setWS } from '@/common';

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
let hashTxtData = {};
const keyData = [[0,0,"<^"],
[1,0,"<#"],
[2,0,"<!"],
[3,0,"Space"],
[4,0,"Space"],
[5,0,"Space"],
[6,0,"Space"],
[7,0,"Space"],
[8,0,"Space"],
[9,0,"Space"],
[10,0,"Space"],
[11,0,">!"],
[12,0,">#"],
[13,0,">^"],
[14,0,],
[15,0,"Left"],
[16,0,"Down"],
[17,0,"Right"],

[0,1,"<+"],
[1,1,"<+"],
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
[12,1,">+"],
[13,1,">+"],
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
[17,5,"Delete"]
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
    max: 10,
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
          let txtCon = dataCon[2] + ':';
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

export default defineComponent({
	name: 'Today',
	setup() {
    let chartDom,myChart;
    function updateKeyData(msg){
      console.log(msg)
    }
		onMounted(() => {
			chartDom = document.getElementById('main');
			myChart = echarts.init(chartDom);
      const store = useAustinStore();
      const keyList = (<any>store.preData).keyList;
      console.log(keyList)
      option.series[0].data =  keyData.map(function (item) {
        let val:string|number = 0 
          if(item[2] ==null)
          {
            val = '-'
          }
          hashTxtData[item[0] +','+item[1]] = keyList[item[2]]??item[2]
          return [item[0], item[1], val];
      });
			option && myChart.setOption(option);

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
		}
	},
})
</script>
