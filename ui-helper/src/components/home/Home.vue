<template>
  <div>
    <n-space vertical>
      <n-card>
		<a id="test1">此处填写使用说明</a>
		<br/>
		<br/>
		<br/>
		<br/>
        <h3>Message</h3>
        <n-button type="primary" @click="handleShowMessage">
          useMessage
        </n-button>
      </n-card>
      <n-card>
        <h3>Dialog</h3>
        <n-button type="primary" @click="handleShowDialog">
          useDialog
        </n-button>
      </n-card>
      <n-card>
        <h3>Nofitication</h3>
        <n-button type="primary" @click="handleShowNotification">
          useNotification
        </n-button>
      </n-card>
      <n-card>
		<a id="test">此处为跳转测试</a>
        <h3>LoadingBar</h3>
        <n-button type="primary" @click="handleShowLoadingBar">
          useLoadingBar
        </n-button>
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
    const route = useRoute()
    const scrollToSection =() => {
      const section = route.params.section;
      console.log('home:'+section)
      if (section) {
        // 滚动到对应的锚点位置
        const element = document.getElementById(section.toString());
        if (element) {
          element.scrollIntoView({ behavior: 'smooth' });
        }
      }
    }
    onMounted(()=>{
      scrollToSection();
    })
		return {
			handleShowMessage,
			handleShowDialog,
			handleShowNotification,
			handleShowLoadingBar,
		}
	},
})
</script>
