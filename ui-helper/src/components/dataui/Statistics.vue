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
			</n-anchor>
		</div>
		<n-space vertical class="fixedSelect">
			<n-space style="font-size:16px">
				{{ contentText.intro112 }}
				<n-select v-model:value="beginDate" filterable :options="historyDate" />
				{{ contentText.intro93 }}
				<n-select v-model:value="endDate" filterable :options="historyDate" />
				<n-button type="primary" @click="handleQuery">{{ contentText?.intro113 }}</n-button>
				<n-switch :round="false" :rail-style="railStyle" v-model:value="fillDate" @update:value="handleQuery">
					<template #checked>
						{{ contentText.intro151 }}
					</template>
					<template #unchecked>
						{{ contentText.intro150 }}
					</template>
				</n-switch>
			</n-space>
			<n-card id="intro114" :title="contentText.intro114">
				<div id="main0" style="height: 200px; min-width: 800px;width:95%;"></div>
			</n-card>
			<n-card id="intro115" :title="contentText.intro115">
				<div id="main1" style="height: 200px; min-width: 800px;width:95%;"></div>
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
import {
	TitleComponent,
	ToolboxComponent,
	TooltipComponent,
	GridComponent,
	LegendComponent
} from 'echarts/components';
import { LineChart } from 'echarts/charts';
import { UniversalTransition } from 'echarts/features';
import { CanvasRenderer } from 'echarts/renderers';

echarts.use([
	TitleComponent,
	ToolboxComponent,
	TooltipComponent,
	GridComponent,
	LegendComponent,
	LineChart,
	CanvasRenderer,
	UniversalTransition
]);

import { arrRemove, getHistory, ajax, railStyle, deepCopy, appPath2Name } from '@/common';
import content from '../../content.js';
let option = []; // 用数组代替
option[0] = {
	xAxis: {
		type: 'category',
		data: []
	},
	yAxis: {
		type: 'value'
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
	tooltip: {
		trigger: 'axis'
	},
	legend: {
		data: ['Sum of mouse', 'Sum of keyboard']
	},
	grid: {
		left: '3%',
		right: '4%',
		bottom: '3%',
		containLabel: true
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
	xAxis: {
		type: 'category',
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
		const beginDate = ref('');
		const endDate = ref('');
		const historyDate = ref([]);
		const topN = ref(0)
		const appTopN = ref(0)
		const fillDate = ref(store.data.dataSetting.fillDate);
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
			let b = beginDate.value, e = endDate.value
			if (/^\d{4}-\d{2}-\d{2}$/.test(b) && /^\d{4}-\d{2}-\d{2}$/.test(e) && e >= b) {
				let res = await ajax('statData', { "beginDate": b, "endDate": e }) //数组，3个内容
				if (res.length < 4) {
					return
				}
				let dateArr = []
				let pixHash = { name: 'Pix Distance', data: [], smooth: true, type: 'line' }
				let screenHash = { name: 'Screen Distance(Meter)', data: [], smooth: true, type: 'line' }
				let phyHash = { name: 'Physical Distance(Meter)', data: [], smooth: true, type: 'line' }
				const df = 'YYYY-MM-DD'
				// 需要循环补充到 x.date 这天
				if (fillDate.value) {
					let ed = dayjs(endDate.value, df).add(1, 'day');
					let currentDate = dayjs(beginDate.value, df);
					while (currentDate.isBefore(ed)) {
						dateArr.push(currentDate.format('YYYY-MM-DD'));
						currentDate = currentDate.add(1, 'day');
					}
				} else {
					res[0].forEach(x => {
						dateArr.push(x.date);
					})
				}
				option.forEach(x => {
					x.xAxis.data = dateArr
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
				option[1].series[0].data = getVal(res[1], dateArr, 'mouse')
				option[1].series[1].data = getVal(res[1], dateArr, 'keyboard')

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
				// 显示全部图标数据
				myChart.forEach((v, i) => {
					v.setOption(option[i], true); // 去除缓存
				})
			} else {
				message.error(contentText.value.intro117)
			}
		}
		onMounted(async () => {
			for (let i = 0; i < 6; i++) {
				chartDom[i] = document.getElementById('main' + i);
				myChart[i] = echarts.init(chartDom[i], store.myTheme);
			}

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
				beginDate.value = dateArr[beginIndex];// 设置选择第一个
				endDate.value = dateArr[0];// 设置选择第一个
				handleQuery()
			}
		})
		watch(() => store.myTheme, (newValue, oldValue) => {
			myChart.forEach((v, i) => {
				v.dispose()
				v = echarts.init(chartDom[i], newValue);
				v.setOption(option[i]);
			})
		});
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
		}
	},
})
</script>
<style scoped>
@import "@/res/common.css";
</style>