<template>
	<div>
		<n-space vertical>
			<n-card v-html="contentText.intro119" style="white-space: pre-line;">
			</n-card>
		</n-space>
	</div>
</template>

<script lang="ts">
import { defineComponent, onMounted, PropType, ref, computed, h, watch } from 'vue'
import content from '../../content.js';

export default defineComponent({
	name: 'Home',
	props: {
		lang: {
			type: String as PropType<'en-US' | 'zh-CN'>,
			default: 'zh-CN'
		},
	},
	setup(props) {
		const contentText = computed(() => content[props.lang])
		return {
			contentText,
		}
	},
})
</script>
