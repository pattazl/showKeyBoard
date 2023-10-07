<template>
	<div style="font-size:16px">
		<n-space vertical>
			<n-space>
				{{ contentText.intro120 }}
				<n-select v-model:value="deleteTick" multiple clearable :options="todayList" />
				<n-button type="error" @click="handleDeleteTick">
					{{ contentText.intro106 }}
				</n-button>
			</n-space>
			<n-space>
				{{ contentText.intro121 }}
				<n-select v-model:value="deleteDate" multiple clearable :options="historyDate" />
				<n-button type="error" @click="handleDeleteDate">
					{{ contentText.intro106 }}
				</n-button>
			</n-space>
		</n-space>
		<n-h3>暂未开发导出导入功能，历史按键信息以 sqlite3 格式保存在 records.db 文件的 stat 表中</n-h3>
	</div>
</template>

<script lang="ts">
import { defineComponent, onMounted, PropType, ref, computed, h, watch } from 'vue'
import content from '../../content.js';
import { useMessage, useDialog } from 'naive-ui'
import { getHistory, ajax } from '@/common';
import { useAustinStore } from '../../App.vue'
import dayjs from 'dayjs'

// 获取今天的全部启动次数信息
async function getTodayData(todayList) {
	let str = dayjs(new Date()).format('YYYY-MM-DD')
	let historyData = await getHistory(str, str)
	let tickSet = new Set();
	historyData.forEach(x => tickSet.add(x.tick))
	tickSet.forEach(x => {
		todayList.push({ label: dayjs(new Date(<number>x)).format('YYYY-MM-DD HH:mm:ss.SSS'), value: x })
	})
}
async function getHistoryDate(historyDate) {
	let dateArr = await ajax('getHistoryDate')
	historyDate.value = dateArr.map((x) => {
		return { label: x, value: x }
	})
}
export default defineComponent({
	name: 'Export',
	props: {
		lang: {
			type: String as PropType<'en-US' | 'zh-CN'>,
		},
	},
	setup(props) {
		const contentText = computed(() => content[props.lang])
		const deleteTick = ref()
		const deleteDate = ref()
		const todayList = ref([])
		const historyDate = ref([])
		const message = useMessage()
		const myDialog = useDialog()
		getTodayData(todayList.value)
		getHistoryDate(historyDate)

		async function handleDeleteTick() {
			myDialog.warning({
				title: contentText.value.intro122+ deleteTick.value.map(x=> dayjs(new Date(x)).format('YYYY-MM-DD HH:mm:ss.SSS')),
				positiveText: contentText.value.intro109,
				negativeText: contentText.value.intro110,
				maskClosable: false,
				onPositiveClick: async () => {
					await ajax('deleteData', { date: deleteTick.value, flag: 0 })
					todayList.value = []
					deleteTick.value = ''
					getTodayData(todayList.value)
					message.success(contentText.value.intro76)

				}
			})
		}
		async function handleDeleteDate() {
			myDialog.warning({
				title: contentText.value.intro122+ deleteDate.value,
				positiveText: contentText.value.intro109,
				negativeText: contentText.value.intro110,
				maskClosable: false,
				onPositiveClick: async () => {
					await ajax('deleteData', { date: deleteDate.value, flag: 1 })
					historyDate.value = []
					deleteDate.value = ''
					getHistoryDate(historyDate)
					message.success(contentText.value.intro76)
				}
			})

		}

		return {
			deleteTick,
			handleDeleteTick,
			handleDeleteDate,
			historyDate,
			todayList,
			deleteDate,
			contentText,
		}
	},
})
</script>
<style scoped>
.n-select {
	width: 500px;
}
</style>