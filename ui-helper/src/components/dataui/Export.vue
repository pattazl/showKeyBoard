<template>
  <div>
    <n-h3>暂未开发，历史按键信息以 sqlite3 格式保存在 records.db 文件的 stat 表中</n-h3>
    <n-button type="primary" @click="handleShowMessage">
      导出数据
    </n-button>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { useMessage } from 'naive-ui'

export default defineComponent({
	name: 'Message',
	setup () {
		const message = useMessage()
		const handleShowMessage = () => {
			message.success('I can use message')
		}
		return {
			handleShowMessage,
		}
	},
})
</script>
