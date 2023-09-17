<template>
  <div>
    <n-space vertical>
      <n-card>此处填写使用说明
	
      </n-card>
    </n-space>
  </div>
</template>

<script lang="ts">
import { defineComponent ,onMounted} from 'vue'
import { useRoute } from 'vue-router';
export default defineComponent({
	name: 'Home',
	setup () {
		const handleShowMessage = () => {
			console.log('I can use message')
		}
		const handleShowDialog = () => {
			
		}
		const handleShowNotification = () => {
			
		}
		const handleShowLoadingBar = () => {
			//window.$loadingBar.start()
		}

		return {
			handleShowMessage,
			handleShowDialog,
			handleShowNotification,
			handleShowLoadingBar,
		}
	},
})
</script>
