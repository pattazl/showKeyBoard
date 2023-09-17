<template>
  <div>
    <n-h3>Dialog</n-h3>
    <n-button type="primary" @click="handleShowDialog">
      useDialog
    </n-button>

  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { useDialog } from 'naive-ui'



export default defineComponent({
	name: 'Dialog',
	setup () {
		const dialog = useDialog()
		const handleShowDialog = () => {
			dialog.success({
				title: 'Success use dialog',
				content: 'Cool',
				positiveText: 'Wow!',
				onPositiveClick: () => {
					//window.$message.success('Great!')
				},
			})
		}
		return {
			handleShowDialog,
		}
	},
})
</script>
