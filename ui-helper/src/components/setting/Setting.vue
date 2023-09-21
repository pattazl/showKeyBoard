<template>
  <div>
    <n-space vertical>
      <h2 id="General">{{ contentText?.menu?.setting1 }}</h2>
      <n-card title="通用设置">
        <n-list hoverable>
        <n-list-item>要忽略记录的按键  <n-dynamic-tags v-model:value="skipRecordRef" />
        </n-list-item>
        <n-list-item>
          是否忽略单独的控制键
           <div class="intro">控制键主要为 Ctrl,Alt,Shift,Win</div>
           <template #suffix>
            <n-switch :round="false" /> 
          </template>
        </n-list-item>
        <n-list-item>是否显示和记录鼠标事件
          <template #suffix>
            <n-switch :round="false" /> 
          </template>
        </n-list-item>
        <n-list-item>是否记录鼠标移动距离 
          <template #suffix>
            <n-switch :round="false" /> 
          </template>
        </n-list-item>
        <n-list-item>是否显示按键  <div class="intro">临时显示和关闭快捷键为。。。</div>
          <template #suffix>
            <n-switch :round="false" /> 
          </template>
        </n-list-item>
        <n-list-item>是否记录按键 
          <template #suffix>
            <n-switch :round="false" /> 
          </template>
        </n-list-item>
        <n-list-item>是否显示控制键状态
          <div class="intro">控制键主要为 Ctrl,Alt,Shift,Win  出现则显示</div>
          <template #suffix>
            <n-switch :round="false" /> 
          </template>
        </n-list-item>
        <n-list-item>后端服务端口号
          <template #suffix >
            <n-input-number v-model:value="allPara.common.serverPort" :min="80" :max="65535" />
          </template>
        </n-list-item>
        <n-list-item>按键显示仅针对活跃窗口 
          <template #suffix>
            <n-input v-model:value="allPara.common.activeWindowProc" type="text" placeholder="使用正则匹配窗口进程名" />
          </template>
        </n-list-item>
      </n-list>
      </n-card>
      <h2  id="KeyUI">{{ contentText?.menu?.setting2 }}</h2>
      <n-card title="按键实时显示界面">
        <n-list hoverable>
          <n-list-item>宽度
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiWidth" :min="1" :max="65535" />
            </template>
          </n-list-item>
          <n-list-item>高度<div class="intro">为0表示自适应</div>
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiHeigth" :min="0" :max="65535" />
            </template>
          </n-list-item>
          <n-list-item>背景色和透明度
            <template #suffix>
              <n-color-picker v-model:value="allPara.dialog.guiWidth" :modes="['hex']" />
            </template>
          </n-list-item>
          <n-list-item>鼠标穿透<div class="intro">可以点击到提示框下方的窗体</div>
            <template #suffix>
              <n-switch v-model:value="allPara.dialog.guiTrans" :round="false" /> 
            </template>
          </n-list-item>
          <n-list-item>字体<div class="intro">选择系统的字体</div>
            <template #suffix>
                <n-select v-model:value="allPara.dialog.guiTextFont" :options="allFontRef" />
            </template>
          </n-list-item>
          <n-list-item>字体大小
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiTextSize" :min="1" :max="100" />
            </template>
          </n-list-item>
          <n-list-item>是否粗体
            <template #suffix>
              <n-switch v-model:value="allPara.dialog.guiTextWeight" :round="false" /> 
            </template>
          </n-list-item>
          <n-list-item>字体颜色
            <template #suffix>
              <n-color-picker v-model:value="allPara.dialog.guiTextColor" :modes="['hex']" />
            </template>
          </n-list-item>
          <n-list-item>显示多少毫秒消失
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiLife" :min="10" :max="100000" />
            </template>
          </n-list-item>
          <n-list-item>窗体间隔毫秒
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiInterval" :min="10" :max="100000" />
            </template>
          </n-list-item>
          <n-list-item>上下左右位置<div class="intro">提示框出现的屏幕位置</div>
            <template #suffix>
                <n-select v-model:value="allPara.dialog.guiPos"
                :options="[{label:'左上',value:'TL'},{label:'右上',value:'TR'},{label:'左下',value:'BL'},{label:'右下',value:'BR'}]" />
            </template>
          </n-list-item>
          <n-list-item>窗口移动位置<div class="intro">横向或垂直移动</div>
            <template #suffix>
                <n-select v-model:value="allPara.dialog.guiPosXY" :options="[{label:'X'},{label:'Y'}]" />
            </template>
          </n-list-item>
          <n-list-item>位置X调整<div class="intro">横向位置调整，负数表示往左，正数表示往右</div>
            <template #suffix>
                <n-input-number v-model:value="allPara.dialog.guiPosOffsetX" />
            </template>
          </n-list-item>
          <n-list-item>位置Y调整<div class="intro">纵向位置调整，负数表示往上，正数表示往下</div>
            <template #suffix>
                <n-input-number v-model:value="allPara.dialog.guiPosOffsetY" />
            </template>
          </n-list-item>
          <n-list-item>是否进行DPI
            <template #suffix>
                <n-switch v-model:value="allPara.dialog.guiDpiscale" :round="false" />
            </template>
          </n-list-item>
          <n-list-item>第几个屏幕<div class="intro">分辨率为 {{  }}</div>
            <template #suffix>
              <n-select v-model:value="allPara.dialog.guiMonitorNum"
                :options="screenNum" />
            </template>
          </n-list-item>
          <n-list-item>提示窗口间的间隔
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiMargin" />
            </template>
          </n-list-item>
          <n-list-item>是否有细边框
            <template #suffix>
                <n-switch v-model:value="allPara.dialog.guiEdge" :round="false" />
            </template>
          </n-list-item>
          <n-list-item>按键分隔符,用于按键间的显示分割
            <template #suffix>
              <n-input v-model:value="allPara.dialog.txtSplit" type="text" placeholder="推荐用空格" />
            </template>
          </n-list-item>
          <h4>长按显示,主要用于控制键,如Ctrl、Alt、LWin、Shift、RWin、CapsLock等</h4>
          <n-list-item>控制键X位置
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.ctrlX" />
            </template>
          </n-list-item>
          <n-list-item>控制键Y位置
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.ctrlY" />
            </template>
          </n-list-item>
          <n-list-item>控制键字体大小
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.ctrlTextSize" :min="1" :max="100" />
            </template>
          </n-list-item>
          <n-list-item>控制键清单
              <n-dynamic-tags v-model:value="ctrlListRef" />
          </n-list-item>
          <n-list-item>哪些按键不会显示，但会记录
              <n-dynamic-tags v-model:value="skipShowRef" />
          </n-list-item>

        </n-list>
      </n-card>
      <h2 id="StatPara">{{ contentText?.menu?.setting3 }}</h2>
      <n-card title="数据统计界面的相关参数设置">
        <n-list hoverable>
          <n-list-item>屏幕大小(英寸)
            <template #suffix>
              <n-input-number :value="1" :min="1" :max="65535" />
            </template>
          </n-list-item>
        </n-list>
      </n-card>
      <h2 id="KeyMap">{{ contentText?.menu?.setting4 }}</h2>
      <n-card>
        <n-dynamic-input
          v-model:value="keyMappintRef"
          preset="pair"
          key-placeholder="按键名"
          value-placeholder="显示的按键符号"
        />
      </n-card>
      <n-card>
        <br />
        <br />
        <br />
        <br />
        <h2>LoadingBar</h2>
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

        <h2>LoadingBar</h2>
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

        <h2>LoadingBar</h2>
        <n-button type="primary" @click="">
          useLoadingBar
        </n-button>
      </n-card>
    </n-space>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted ,PropType,ref,computed } from 'vue'
import { useRoute } from 'vue-router';
import content from '../../content.js';
// import { useAustinStore } from '../../App.vue'

function splitArr(str){
  let arr =[]
  if( str.length > 0){
    arr = str.split('|')
  }
  return arr;
}
// 生成界面上select的数组
function toVSelectList(arr:Array<string|number>){
  let resArr = [];
  for(let v of arr)
  {
    resArr.push({ label:v,value:v })
  }
  return resArr
}
// 生成界面上key,value数组
function toKVList(obj:{}){
  let resArr = [];
  for(let k in obj)
  {
    resArr.push({ key:k,value:obj[k] })
  }
  return resArr
}
export default defineComponent({
  name: 'Setting',
  props:{
    lang: {
      type: String as PropType<'en-US' | 'zh-CN'>,
    },
  },
  setup(props) {
    // const store= useAustinStore(); 可通过 属性传递，也可通过pinia传递
    // const lang = computed(()=>store.lang) 
    const contentText = computed(()=>content[props.lang]) 
    //watch(() => store.lang, (newValue, oldValue) => {
    //  console.log(` lang New value: ${newValue}, old value: ${oldValue}`);
    //});
    // 各类数据模拟
    const allPara = ref({
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
    // 字体清单
    const allFont = ['宋体','雅黑','Verdana'];
    // 按键匹配清单
    const keyMappint = {
      'Space' : ' ␣',
'<^' : ' ^+',
'>^' : ' ^+',
'<!' : ' ⎇+',
'>!' : ' ⎇+',
'<+' : ' ⇧+',
'>+' : ' ⇧+',
'<#' : ' ⊞+',
'>#' : ' ⊞+',
'Enter' : ' ↩',
'Left' : ' ⬅ ',
'Up' : '⬆︎ ',
'Down' : '⬇ ',
'Right' : '➞',
'LControl' : ' ^',
'RControl' : ' ^',
'LWin' : ' ⊞',
'RWin' : ' ⊞',
'LAlt' : ' ⎇',
'RAlt' : ' ⎇',
'LShift' : ' ⇧',
'RShift' : ' ⇧',
'Backspace' : ' BS',
'Escape' : ' Esc',
'CapsLock' : 'Caps',
'Delete' : 'Del ',
'Insert' : 'Ins',
'PrintScreen' : 'PrtSc',
'ScrollLock' : 'Scroll',
'Insert' : 'Ins',
'PgUp' : ' PgUp',
'AppsKey' : ' Menu ',
'Ctrl' : ' ^',
'LWin' : ' ⊞',
'RWin' : ' ⊞',
'Alt' : ' ⎇',
'Shift' : ' ⇧',
'WheelUp' : ' ⇡',
'WheelDown' : ' ⇣',
'MButton' : ' 🖲️',
'LButton' : ' 🖱️',
'RButton' : ' 🖰',
    }
    // 屏幕数据
    const screenInfo = [{Left:0, Top:0, Right:100, Bottom:200},{Left:0, Top:0, Right:100, Bottom:200}]
    let a = [...Array(screenInfo.length).keys()]
    const screenNum = toVSelectList( [...Array(screenInfo.length).keys()].map(x => x + 1) )

    // 数据处理
    const allFontRef = toVSelectList(allFont)

    const keyMappintRef = ref(toKVList(keyMappint));
    const skipRecordRef = ref(splitArr(allPara.value.common.skipRecord))
    const ctrlListRef = ref(splitArr(allPara.value.dialog.ctrlList)) 
    const skipShowRef = ref(splitArr(allPara.value.dialog.skipShow))

    const handleShowMessage = () => {
      console.log('I can use message')
    }
    // 自动路由识别和滚动
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
      skipRecordRef,
      contentText,
      ctrlListRef,
      skipShowRef,
      allFontRef,
      screenNum,
      keyMappintRef,
    }
  },
})
</script>
<style scoped>
.intro {
  font-size:smaller;
  color:#2080f0;;
}
.n-input-number{
  width:150px
}
.n-color-picker , .n-select {
  width:180px;
}
.n-input{
  width:300px;
}
</style>