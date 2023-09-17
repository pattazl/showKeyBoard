<template>
  <div>
    <n-space vertical>
      <n-card>
        <a id="General">{{ contentText?.menu?.setting1 }}</a>
        <br />
        <br />
        <br />
        <br />
        <h3>Message</h3>
        <n-button type="primary" @click="handleShowMessage">
          useMessage
        </n-button>
      </n-card>
      <n-card>
        <h3>Dialog</h3>
        <n-button type="primary" @click="">
          useDialog
        </n-button>
      </n-card>
      <n-card>
        <h3>Nofitication</h3>
        <n-button type="primary" @click="">
          useNotification
        </n-button>
      </n-card>
      <a id="KeyUI">KeyUI</a>
      <br />
      <br />
      <br />
      <br />
      <n-card>
        <h3>LoadingBar</h3>
        <n-button type="primary" @click="">
          useLoadingBar
        </n-button>
      </n-card>
      <a id="StatPara">StatPara</a>
      <n-card>
        <br />
        <br />
        <br />
        <br />
        <h3>LoadingBar</h3>
        <n-button type="primary" @click="">
          useLoadingBar
        </n-button>
      </n-card>
      <a id="KeyMap">KeyMap</a>
      <br />
      <br />
      <br />
      <br />
      <n-card>

        <h3>LoadingBar</h3>
        <n-button type="primary" @click="">
          useLoadingBar
        </n-button>
      </n-card>
      <a id="Save">Save</a>
      <br />
      <br />
      <br />
      <br />
      <n-card>

        <h3>LoadingBar</h3>
        <n-button type="primary" @click="">
          useLoadingBar
        </n-button>
      </n-card>
    </n-space>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted } from 'vue'
import { useRoute } from 'vue-router';
import content from '../../content.js';
import { useAustinStore } from '../../App.vue'
export default defineComponent({
  name: 'Setting',
  setup() {
    const store= useAustinStore();
    const contentText = computed(()=>content[store.lang]) 
    //watch(() => store.lang, (newValue, oldValue) => {
    //  console.log(` lang New value: ${newValue}, old value: ${oldValue}`);
    //});
    const allPara = toRef({
      "common": {
        "skipRecord": "",
        "skipCtrlKey": "0",
        "needShowKey": "1",
        "needRecordKey": "1",
        "ctrlState": "1",
        "serverPort": "9900",
        "showMouseEvent": "3",
        "recordMouseMove": "1",
        "activeWindowProc": ""
      },
      "dialog": {
        "guiWidth": "240",
        "guiHeigth": "0",
        "guiBgcolor": "11AA99",
        "guiBgTrans": "0",
        "guiTrans": "1",
        "guiOpacity": "150",
        "guiTextFont": "Verdana",
        "guiTextSize": "26",
        "guiTextWeight": "bold",
        "guiTextColor": "FF0000",
        "guiLife": "7000",
        "guiInterval": "1000",
        "guiPos": "BR",
        "guiPosXY": "Y",
        "guiPosOffsetX": "0",
        "guiPosOffsetY": "-50",
        "guiDpiscale": "0",
        "guiMonitorNum": "2",
        "guiMargin": "5",
        "guiEdge": "1",
        "txtSplit": " ",
        "ctrlX": "10",
        "ctrlY": "5",
        "ctrlTextSize": "20",
        "ctrlList": "Ctrl|Alt|LWin|Shift|RWin|CapsLock",
        "skipShow": "<^<+a|PrintScreen"
      }
    })
    const handleShowMessage = () => {
      console.log('I can use message')
    }

    const route = useRoute()
    const scrollToSection = () => {
      const section = route.params.section;
      console.log('Setting:' + section)
      if (section) {
        // 滚动到对应的锚点位置
        const element = document.getElementById(section.toString());
        if (element) {
          element.scrollIntoView({ behavior: 'smooth' });
        }
      }
    }
    onMounted(() => {
      scrollToSection();
    })
    return {
      handleShowMessage,
      allPara,
      contentText,
    }
  },
})
</script>
