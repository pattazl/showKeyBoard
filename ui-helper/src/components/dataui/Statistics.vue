<template>
	<div>
		<div style="height: 120px ;margin-right: 90px;float: right;">
			<n-anchor affix :top="80" style="z-index: 10; font-size: 18px; " :bound="50" :show-rail="false"
				:ignore-gap="true" type='block' position='fix'>
				<n-anchor-link :title="contentText.intro114" href="#intro114" />
				<n-anchor-link :title="contentText.intro115" href="#intro115" />
				<n-anchor-link :title="contentText.intro116" href="#intro116" />
				<n-anchor-link :title="contentText.intro152" href="#intro152" />
				<n-anchor-link :title="contentText.intro153" href="#intro153" />
				<n-anchor-link :title="contentText.intro154" href="#intro154" />
				<n-anchor-link :title="contentText.intro211" href="#intro211" />
				<n-anchor-link :title="contentText.intro212" href="#intro212" />
			</n-anchor>
		</div>
		<n-space vertical class="fixedSelect">
			<n-space style="font-size:16px" :class="store.myTheme == 'dark' ? 'mydark' : 'mylight'">
				<span id="fixedSpan">{{ contentText.intro112 }}</span>
				<n-date-picker type="date" v-model:value="beginDate" :is-date-disabled="dateDisabled" />
				{{ contentText.intro93 }}
				<n-date-picker type="date" v-model:value="endDate" :is-date-disabled="dateDisabled" />
				<n-button type="primary" @click="handleQuery">{{ contentText?.intro113 }}</n-button>
				<n-switch :round="false" :rail-style="railStyle" v-model:value="fillDate" @update:value="handleQuery">
					<template #checked>
						{{ contentText.intro151 }}
					</template>
					<template #unchecked>
						{{ contentText.intro150 }}
					</template>
				</n-switch>
				{{ contentText.intro209 }}
				<n-select default-value="" @update:value="changeDb" :options="dbsOption" style="max-width:180px" />
			</n-space>
			<n-card id="intro114" :title="contentText.intro114">
				<div id="main0" style="height: 200px; min-width: 800px;width:95%;"></div>
			</n-card>
			<n-card id="intro115" :title="contentText.intro115">
				<div id="main1" style="height: 200px; min-width: 800px;width:95%;"></div>
				<div id="main6" style="height: 200px; min-width: 800px;width:95%;"></div>
			</n-card>
			<n-card id="intro116" :title="contentText.intro116 + ':' + topN">
				<div id="main2" style="height: 400px; min-width: 800px;width:95%;"></div>
			</n-card>
			<n-card id="intro152" :title="contentText.intro152 + ':' + appTopN">
				<div id="main3" style="height: 300px; min-width: 800px;width:95%;"></div>
			</n-card>
			<n-card id="intro153" :title="contentText.intro153 + ':' + appTopN">
				<div id="main4" style="height: 300px; min-width: 800px;width:95%;"></div>
			</n-card>
			<n-card id="intro154" :title="contentText.intro154 + ':' + appTopN">
				<div id="main5" style="height: 300px; min-width: 800px;width:95%;"></div>
			</n-card>
			<n-card id="intro211" :title="contentText.intro211 + ':' + appTopN">
				<div id="main7" style="height: 600px; min-width: 800px;width:95%;"></div>
			</n-card>
			<n-card id="intro212" :title="contentText.intro212">
				<div id="main8" style="height: 300px; min-width: 800px;width:95%;"></div>
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
import { CalendarComponent } from 'echarts/components';

import {
	TitleComponent,
	ToolboxComponent,
	TooltipComponent,
	GridComponent,
	LegendComponent
} from 'echarts/components';
import { LineChart,BarChart } from 'echarts/charts';
import { UniversalTransition } from 'echarts/features';
import { CanvasRenderer } from 'echarts/renderers';

echarts.use([
	TitleComponent,
	ToolboxComponent,
	TooltipComponent,
	GridComponent,
	LegendComponent,
	LineChart,
	BarChart,
	CanvasRenderer,
	UniversalTransition,
	CalendarComponent
]);

import { arrRemove, getHistory, ajax, railStyle, deepCopy, appPath2Name, dateFormat, addExtListener,getDbs,setDbSel,minute2Hour } from '@/common';
import content from '../../content.js';
// 大数值格式化函数：转换为万/百万/亿单位
function formatLargeNumber(value) {
	if (value >= 10000) {
		return value.toExponential(0);;
	}
	return value; // 小于1万直接显示
}
let option = []; // 用数组代替
let optionCount = 9
option[0] = {
	grid: {
		left: 50, 
		right: 50, 
	},
	xAxis: {
		type: 'category',
		boundaryGap: false,
		data: []
	},
	yAxis: {
		type: 'value',
		axisLabel: {
			// 应用格式化函数
			formatter: function(value) {
				return formatLargeNumber(value);
			}
		}
	},
	tooltip: {
		trigger: 'axis'
	},

	legend: {
		data: ['Pix Distance', 'Screen Distance(Meter)', 'Physical Distance(Meter)']
	},
	series: []
};
option[1] = {
	grid: {
		left: 50, 
		right: 50, 
	},
	tooltip: {
		trigger: 'axis'
	},
	legend: {
		data: ['Sum of mouse', 'Sum of keyboard']
	},
	xAxis: {
		type: 'category',
		boundaryGap: false,
		data: []
	},
	yAxis: {
		type: 'value'
	},
	series: [
		{
			name: 'Sum of mouse',
			type: 'line',
			data: [120, 132, 101, 134, 90, 230, 210]
		},
		{
			name: 'Sum of keyboard',
			type: 'line',
			data: [220, 182, 191, 234, 290, 330, 310]
		}

	]
};
option[2] = {
	grid: {
		left: 50, 
		right: 50, 
	},
	xAxis: {
		type: 'category',
		boundaryGap: false,
		data: []
	},
	yAxis: {
		type: 'value'
	},
	tooltip: {
		trigger: 'axis'
	},

	legend: {
		data: []
	},
	series: []
};
option[3] = deepCopy(option[2])
option[4] = deepCopy(option[2])
option[5] = deepCopy(option[2])

// 热力图，有数据差异
// function initHeatDateRange() {
// let endDate , startDate 
//   const end = new Date();
//   end.setDate(end.getDate()-1)
//   const start = new Date(end.getTime())
//   start.setFullYear(end.getFullYear() - 1);
//   endDate = end.getTime()
//   startDate = start.getTime()
// //   const dayTime = 3600 * 24 * 1000;
// //   const data = [];
// //   for (let time = startDate; time < endDate; time += dayTime) {
// //     data.push([
// //       echarts.time.format(time, '{yyyy}-{MM}-{dd}', false),
// //       Math.floor(Math.random() * 10000)
// //     ]);
// //   }
// //   return data;
// 	return [ echarts.time.format(startDate, '{yyyy}-{MM}-{dd}', false),
//     echarts.time.format(endDate, '{yyyy}-{MM}-{dd}', false) ]
// }
// 日历热力图
option[6] = {
	title: {
		top: 30,
		left: 'center',
		text: 'Daily Count'
	},
	tooltip: {
		// 自定义提示框内容
		formatter: function (params) {
			// 返回提示框HTML内容
			return `<div style="font-weight:bold;margin-bottom:5px">${params.value[0]}</div><div>${params.value[1]}</div>`;
		}
	},
	visualMap: {
		min: 0,
		max: 20000,
		type: 'piecewise',
		orient: 'horizontal',
		left: 'center',
		inRange: {
			color: ['#e0f2fe', '#bae6fd', '#7dd3fc', '#38bdf8', '#0ea5e9', '#0284c7']
		},
		top: 55
	},
	calendar: {
		top: 100,
		left: 50,
		right: 50,
		cellSize: ['auto', 13],
		range: [], // initHeatDateRange(),
		itemStyle: {
			borderWidth: 0.5
		},
		yearLabel: { show: false }
	},
	series: {
		type: 'heatmap',
		coordinateSystem: 'calendar',
		data: []
	}
}
// 堆叠面积图

option[7] = {
  grid: {
  	left: 50, 
  	right: 50, 
  },
  legend: {
    show: false
  },
  yAxis: {
    type: 'value'
  },
  xAxis: {
    type: 'category',
	// boundaryGap: false,
    data: []
  },
  title: {
    text: ''
  },
  toolbox: {
    feature: {
      saveAsImage: {}
    }
  },
  series: [],
  tooltip: {
         trigger: 'axis', // 触发类型：按坐标轴触发（堆叠图推荐）
         axisPointer: { type: 'shadow' }, // 鼠标悬浮时显示阴影指示器
		 extraCssText: 'max-width: 400px; white-space: normal; word-wrap: break-word;',
         // 自定义提示内容格式
         formatter: function (params) {
           // params 是当前柱子对应的所有堆叠系列数据数组
           let tip = `<div>${params[0].name} App Minutes</div>`; // 标题（如“周一 销售额”）
		   let list = []
           params.forEach(item => {
			if(item.value==0)return;// 跳过
			list.push(`<div style='color:${item.color}'>${item.seriesName}: ${item.value}${minute2Hour(item.value)}</div>`);
           });
		   list.reverse()
           // 计算堆叠总和并添加
           const total = params.reduce((sum, item) => sum + item.value, 0);
           tip += `${list.join('')}<div>Total: ${total.toFixed(2)} ${minute2Hour(total)}</div>`;
           return tip;
         }
       },
}
option[8] = deepCopy(option[2])
option[8].tooltip = {
	trigger: 'axis', // 触发类型：按坐标轴触发（堆叠图推荐）
	axisPointer: { type: 'shadow' }, // 鼠标悬浮时显示阴影指示器
	// 自定义提示框内容
	formatter: function (params) {
		// 返回提示框HTML内容
		let min = ~~params[0].value.toFixed(0)
		return `<div style="font-weight:bold;margin-bottom:5px">${params[0].name}</div><div>${min} ${minute2Hour(min)}</div>`;
	}
}
option[8].series[0] = {
	name: 'Daily Minutes',
	type: 'line',
	data: [120, 132, 101, 134, 90, 230, 210]
}

export default defineComponent({
	name: 'Message',
	props: {
		lang: {
			type: String as PropType<'en-US' | 'zh-CN'>,
		},
	},
	setup(props) {
		const store = useAustinStore();
		const contentText = computed(() => content[props.lang])
		const message = useMessage()
		const beginDate = ref(0);
		const endDate = ref(0);
		const historyDate = ref([]);
		const topN = ref(0)
		const appTopN = ref(0)
		const fillDate = ref(store.data.dataSetting.fillDate);
		const dbsOption = ref([]);
		const appNameListMap = JSON.parse(store.data.dataSetting.appNameList);


		topN.value = store.data.dataSetting.topN;
		appTopN.value = store.data.dataSetting.appTopN;

		const sinfo = store.data.infoPC?.screen; // [{Left:0, Top:0, Right:100, Bottom:200},{Left:0, Top:0, Right:100, Bottom:200}]
		let screenPixlSize = 0;
		if (sinfo != null && sinfo.length > 0 && sinfo[0].Right != null) {
			screenPixlSize = Math.sqrt((sinfo[0].Right - sinfo[0].Left) ** 2 + (sinfo[0].Bottom - sinfo[0].Top) ** 2)
		}
		// 从目标数组中根据byArr顺序返回符合要求的数组
		function getVal(targetArr, byArr, keyname) {
			let returnArr = []
			byArr.forEach(x => {
				returnArr.push(targetArr.find(y => y.date == x && y.keyname == keyname)?.keycount ?? 0)
			})
			return returnArr;
		}
		// 获取全部按键数据的组合，以dim为维度从arr中取数据
		function getKeyData(dim: Array<String>, arr) {
			let hashList = {}
			// 已经排序过
			arr.forEach(x => {
				if (hashList[x.keyname] == null) {
					hashList[x.keyname] = {
						name: x.keyname,
						date: { [x.date]: x.keycount },
						smooth: true,
						type: 'line'
					}
				} else {
					hashList[x.keyname].date[x.date] = x.keycount
				}
			})
			// 对 hashList 中每个key的 date 字段按 dim 进行整理，没有值则为空
			for (let k in hashList) {
				let obj = hashList[k]
				obj.data = dim.map(x => obj.date[x] ?? 0)
			}
			return hashList
		}
		let chartDom = [], myChart = [];
		async function handleQuery() {
			let b = dayjs(beginDate.value).format(dateFormat), e = dayjs(endDate.value).format(dateFormat)
			// console.log(b,e)
			if (/^\d{4}-\d{2}-\d{2}$/.test(b) && /^\d{4}-\d{2}-\d{2}$/.test(e) && e >= b) {
				let res = await ajax('statData', { "beginDate": b, "endDate": e }) //数组，3个内容
				if (res.length < 4) {
					return
				}
				let dateArr = []
				let pixHash = { name: 'Pix Distance', data: [], smooth: true, type: 'line' }
				let screenHash = { name: 'Screen Distance(Meter)', data: [], smooth: true, type: 'line' }
				let phyHash = { name: 'Physical Distance(Meter)', data: [], smooth: true, type: 'line' }
				// 需要循环补充到 x.date 这天
				if (fillDate.value) {
					let ed = dayjs(endDate.value, dateFormat).add(1, 'day');
					let currentDate = dayjs(beginDate.value, dateFormat);
					while (currentDate.isBefore(ed)) {
						dateArr.push(currentDate.format(dateFormat));
						currentDate = currentDate.add(1, 'day');
					}
				} else {
					res[0].forEach(x => {
						dateArr.push(x.date);
					})
				}
				option.forEach(x => {
					if(x.xAxis!=null){
						if( Array.isArray(x.xAxis)){
							x.xAxis[0].data = dateArr
						}else{
							x.xAxis.data = dateArr
						}
					}
				})
				// 设置	
				dateArr.forEach(x => {
					let kc = res[0].find(y => y.date == x)?.keycount ?? 0
					pixHash.data.push(kc)
					if (screenPixlSize > 0) {
						screenHash.data.push((kc * (store.data.dataSetting.screenSize * 0.0254) / screenPixlSize).toFixed(4));
					}
					if (store.data.dataSetting.mouseDPI > 0) {
						phyHash.data.push((kc * (0.0254) / store.data.dataSetting.mouseDPI).toFixed(4));
					}
				})
				option[0].series = [pixHash, screenHash, phyHash]
				let mouseArr = getVal(res[1], dateArr, 'mouse')
				let kbArr = getVal(res[1], dateArr, 'keyboard')
				option[1].series[0].data = mouseArr
				option[1].series[1].data = kbArr
				// 对鼠标键盘合计相加形成热力图数据
				let heatMax = 0 // 最大计数
				let heatRange = ['9999','']
				let heatArr = dateArr.map((x,i) => {
					let count = mouseArr[i] + kbArr[i]
					heatMax = Math.max(heatMax,count)
					heatRange[0] = x < heatRange[0]?x:heatRange[0]
					heatRange[1] = x > heatRange[1]?x:heatRange[1]
					return [x,count] 
				})

				let hash = getKeyData(dateArr, res[2])
				option[2].series = Object.keys(hash).map(x => hash[x])
				option[2].legend.data = Object.keys(hash)
				// 对应用进行统计分析 
				// 对 res[3] 分离出3个数组分别是 汇总，鼠标，键盘
				let appInfo = [[], [], []];
				res[3].forEach(x => {
					let val = '', shortName = ''
					if (x.keyname.indexOf('App-Mouse-') == 0) {
						val = x.keyname.replace(/App-Mouse-/, '')
						shortName = appPath2Name(val, appNameListMap)
						x.keyname = shortName
						appInfo[1].push(x)
					} else if (x.keyname.indexOf('App-Key-') == 0) {
						val = x.keyname.replace(/App-Key-/, '')
						shortName = appPath2Name(val, appNameListMap)
						x.keyname = shortName
						appInfo[2].push(x)
					}
					// 汇总
					if (val != '') {
						let index = appInfo[0].findIndex(y => (y.keyname == shortName && y.date == x.date))
						if (index != -1) {
							appInfo[0][index].keycount += x.keycount  // 如果有相同日期和键值的就累加
						} else {
							let xx = deepCopy(x) // 需要深拷贝，避免影响上面2个函数的数据
							appInfo[0].push(xx)  // 没有数据则插入
						}
					}
				})
				// 应用数据循环设置参数
				appInfo.forEach((xx, i) => {
					let hash = getKeyData(dateArr, xx)
					option[3 + i].series = Object.keys(hash).map(x => hash[x])
					option[3 + i].legend.data = Object.keys(hash)
				})
				// 对于 日期热力图需要额外处理
				option[6].title.text = contentText.value.intro199
				option[6].visualMap.max = heatMax
				option[6].series.data = heatArr
				option[6].calendar.range = heatRange
				// 提示说明
				option[0].legend.data = [contentText.value.intro200,contentText.value.intro201,contentText.value.intro202]
				option[0].series[0].name = contentText.value.intro200
				option[0].series[1].name = contentText.value.intro201
				option[0].series[2].name = contentText.value.intro202
				option[1].legend.data = [contentText.value.intro203,contentText.value.intro204]
				option[1].series[0].name = contentText.value.intro203
				option[1].series[1].name = contentText.value.intro204

			   // 需要查询 应用统计时长数据 [{"Apps":"msedge.exe","Date":"2025-09-09","Minutes":161.32}
			   res = await ajax('getAppMinute', { "beginDate": b, "endDate": e,"isTotal":0 }) //数组，3个内容
			   // 原始数据（假设从接口获取）
				const categories = [...new Set(res.map(item => item.Apps))];
				// 2. 动态构建series数据
				const series = categories.map(apps => {
				const data = dateArr.map(date => {
					// 查找对应日期和分类的值，没有则为0
					const item = res.find(i => i.Date === date && i.Apps === apps);
					return item ? item.Minutes : 0;
					});
					return {
						name: apps,
						type: 'bar',
						stack: 'total',
						data: data
					};
				});
				// option[7].legend.data = categories
				option[7].series = series

				// 需要查询 每日使用数据 [{"Date":"2025-09-09","Minutes":161.32}
				res = await ajax('getAppMinute', { "beginDate": b, "endDate": e,"isTotal":1 }) //数组，3个内容
				option[8].xAxis.data = res.map(x=>x.Date)
				option[8].series[0].data = res.map(x=>x.Minutes)
				//option[8].legend.data = ['total']
				// 显示全部图标数据
				myChart.forEach((v, i) => {
					v.setOption(option[i], true); // 去除缓存
				})
			} else {
				message.error(contentText.value.intro117)
			}
			
		}
		async function changeDb(db){
			setDbSel(db);
			// 设置下拉选择
			let dateArr = await ajax('getHistoryDate')
			historyDate.value = dateArr.map((x) => {
				return { label: x, value: x }
			})
			if (dateArr.length > 0) {
				let beginIndex = 6;
				if (dateArr.length < 7) {
					beginIndex = dateArr.length - 1
				}
				beginDate.value = dayjs(dateArr[beginIndex], dateFormat).valueOf();// 设置选择第一个
				endDate.value = dayjs(dateArr[0], dateFormat).valueOf();// 设置选择第一个
				handleQuery()
			}
		}
		onMounted(async () => {
			for (let i = 0; i < optionCount; i++) {
				chartDom[i] = document.getElementById('main' + i);
				myChart[i] = echarts.init(chartDom[i], store.myTheme);
			}
			addExtListener(myChart);
			// 需要增加多个数据源的选择
			let dbs = [{label:contentText.value.intro210,value:''}]
			await getDbs(dbs)
			dbsOption.value = dbs
			changeDb('')
		});
		// 监控英文切换等
		watch(() => contentText.value, (newValue, oldValue) => {
			option[6].title.text =  newValue.intro199
			option[0].legend.data = [newValue.intro200,newValue.intro201,newValue.intro202]
			option[0].series[0].name = newValue.intro200
			option[0].series[1].name = newValue.intro201
			option[0].series[2].name = newValue.intro202
			option[1].legend.data = [newValue.intro203,newValue.intro204]
			option[1].series[0].name = newValue.intro203
			option[1].series[1].name = newValue.intro204

			myChart[0].setOption(option[0], true);
			myChart[1].setOption(option[1], true);
			myChart[6].setOption(option[6], true);

			// myChart[7].setOption(option[6], true); // 需要使用 堆叠面积图
		});
		watch(() => store.myTheme, (newValue, oldValue) => {
			myChart.forEach((v, i) => {
				v.dispose()
				v = echarts.init(chartDom[i], newValue);
				v.setOption(option[i]);
			})
		});
		function dateDisabled(ts: number) {
			const date = dayjs(ts).format(dateFormat)
			return !historyDate.value.some(x => {
				return x.value == date
			})
		}
		return {
			contentText,
			beginDate,
			endDate,
			historyDate,
			handleQuery,
			topN,
			appTopN,
			railStyle,
			fillDate,
			store,
			dateDisabled,
			dbsOption,
			changeDb,
		}
	},
})
</script>
<style scoped>
@import "@/res/common.css";
</style>