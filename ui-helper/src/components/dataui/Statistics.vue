<template>
	<div>
		<n-space vertical>
			<n-space style="font-size:16px">
				{{ contentText.intro112 }} 
				<n-select v-model:value="beginDate" filterable :options="historyDate" />
				{{ contentText.intro93 }} 
				<n-select v-model:value="endDate" filterable :options="historyDate" />
				<n-button type="primary" @click="handleUpdateValue">{{ contentText?.intro113 }}</n-button>
			</n-space>
			<n-card :title="contentText.intro114">
				<div id="main1" style="height: 300px; min-width: 800px;width:95%;"></div>
			</n-card>
			<n-card :title="contentText.intro115">
				<div id="main2" style="height: 300px; min-width: 800px;width:95%;"></div>
			</n-card>
			<n-card :title="contentText.intro116">
				<div id="main3" style="height: 300px; min-width: 800px;width:95%;"></div>
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
import { arrRemove, getHistory, ajax } from '@/common';
import content from '../../content.js';

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
		
		let chartDom1, myChart1;
		let chartDom2, myChart2;
		let chartDom3, myChart3;


		async function handleUpdateValue(value) {
			//historyData = await getHistory(value, value)
			//let keyStatHash = getHash('')
			//showHash(keyStatHash)
		}
		onMounted(async () => {
			chartDom1 = document.getElementById('main1');
			myChart1 = echarts.init(chartDom1, store.myTheme);
			// 设置下拉选择
			let dateArr = await ajax('getHistoryDate')
			historyDate.value = dateArr.map((x) => {
				return { label: x, value: x }
			})
			if (dateArr.length > 0) {
				let beginIndex = 6;
				if(dateArr.length<7){
					beginIndex = dateArr.length-1
				}
				beginDate.value = dateArr[beginIndex];// 设置选择第一个
				endDate.value = dateArr[0];// 设置选择第一个

			}

		})
		return {
			contentText,
			beginDate,
			endDate,
			historyDate,
			handleUpdateValue,
		}
	},
})
</script>
