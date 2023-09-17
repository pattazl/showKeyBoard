<template>
  <div>
    <n-h3>Message</n-h3>
    <n-button type="primary" @click="handleShowMessage">
      useMessage
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
