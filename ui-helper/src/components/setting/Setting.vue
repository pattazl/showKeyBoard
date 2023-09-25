<template>
  <div  ref="containerRef">
    <div style="height: 120px ;margin-right: 90px;float: right;">
      <n-anchor
        affix
        :top="80"
        :listen-to="() => containerRef"
        style="z-index: 10; font-size: 18px; "
        :bound="50"
        :show-rail="false"
        :ignore-gap="true"
        type ='block'
        position='fix'
      >
        <n-anchor-link :title="contentText?.menu?.setting1" @click.prevent ="scrollTo('General')" href="#General"/>
        <n-anchor-link :title="contentText?.menu?.setting2" @click.prevent ="scrollTo('KeyUI')" href="#KeyUI"/>
        <n-anchor-link :title="contentText?.menu?.setting3" @click.prevent ="scrollTo('StatPara')" href="#StatPara"/>
        <n-anchor-link :title="contentText?.menu?.setting4" @click.prevent ="scrollTo('KeyMap')" href="#KeyMap" />
        <n-anchor-link :title="contentText?.menu?.setting5" @click.prevent ="scrollTo('Save')" href="#Save" />
      </n-anchor>
    </div>
    <div>
    <n-space vertical >
      <h2 id="General">{{ contentText?.menu?.setting1 }}</h2>
      <n-card :style="myBorder.General?'border:1px #18a058 solid':''">
        通用设置
        <n-list hoverable v-if="allPara.common">
        <n-list-item>要忽略记录的按键  <n-dynamic-tags v-model:value="skipRecordRef" />
        </n-list-item>
        <n-list-item>
          是否忽略单独的控制键
           <div class="intro">控制键主要为 Ctrl,Alt,Shift,Win</div>
           <template #suffix>
            <n-switch :round="false" v-model:value="allPara.common.skipCtrlKey"/> 
          </template>
        </n-list-item>
        <n-list-item>是否显示和记录鼠标事件
          <template #suffix>
            <n-select v-model:value="allPara.common.showMouseEvent" :options="showRecEvent" />
          </template>
        </n-list-item>
        <n-list-item>是否记录鼠标移动距离 
          <template #suffix>
            <n-switch :round="false" v-model:value="allPara.common.recordMouseMove"/> 
          </template>
        </n-list-item>
        <n-list-item>是否显示按键
          <template #suffix>
            <n-switch :round="false" v-model:value="allPara.common.needShowKey"/> 
          </template>
        </n-list-item>
        <n-list-item>是否记录按键 
          <template #suffix>
            <n-switch :round="false" v-model:value="allPara.common.needRecordKey"/> 
          </template>
        </n-list-item>
        <n-list-item>是否显示控制键状态
          <div class="intro">控制键主要为 Ctrl,Alt,Shift,Win  出现则显示</div>
          <template #suffix>
            <n-switch :round="false" v-model:value="allPara.common.ctrlState"/> 
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
        <n-list-item>按键临时显示开关快捷键 
          <template #suffix><div class="intro">定义快捷键, !表示Alt , #表示windows, +表示shift, ^ 表示Ctrl, 比如 ^#F5 表示同时按下 ctrl+win+F5键，修改后重启程序生效</div>
            <n-input v-model:value="allPara.common.hotkey4Show" type="text" placeholder="手工输入按键" />
          </template>
        </n-list-item>
      </n-list>
      </n-card>
      <h2 id="KeyUI">{{ contentText?.menu?.setting2 }}</h2>
      <n-card :style="myBorder.KeyUI?'border:1px #18a058 solid':''">
        按键实时显示界面
        <n-list hoverable v-if="allPara.dialog">
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
              <n-color-picker v-model:value="allPara.dialog.guiBgcolor" :modes="['hex']" />
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
      <n-card :style="myBorder.StatPara?'border:1px #18a058 solid':''">
        数据统计界面的相关参数设置
        <n-list hoverable>
          <n-list-item>屏幕大小(英寸)
            <template #suffix>
              <n-input-number :value="1" :min="1" :max="65535" />
            </template>
          </n-list-item>
          <n-list-item>鼠标DPI<div class="intro">用于计算实际鼠标移动距离</div>
            <template #suffix>
              <n-input-number :value="1" :min="1" :max="65535" />
            </template>
          </n-list-item>
        </n-list>
      </n-card>
      <h2 id="KeyMap">{{ contentText?.menu?.setting4 }}</h2>
      <n-card :style="myBorder.KeyMap?'border:1px #18a058 solid':''">
        <n-dynamic-input
          v-model:value="keyMappingRef"
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
      <h2 id="Save">{{ contentText?.menu?.setting5 }}</h2>
      <n-card :style="myBorder.Save?'border:1px #18a058 solid':''">
        <h2>LoadingBar</h2>
        <n-button type="primary" @click="">
          useLoadingBar
        </n-button>
        <a >Save</a>
      <br />
      <br />
      <br />
      <br /><a >Save</a>
      <br />
      <br />
      <br />
      <br /> <a >Save</a>
      <br />
      <br />
      <br />
      <br /><a >Save</a>
      <br />
      <br />
      <br />
      <br />
      </n-card>

    </n-space>
  </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted ,PropType,ref,computed } from 'vue'
import { useRoute } from 'vue-router';
import { AnchorInst  } from 'naive-ui';
import content from '../../content.js';
import { storeToRefs } from 'pinia'
import { useAustinStore } from '../../App.vue'
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
// 布尔类型清单
const boolArr =['skipCtrlKey','recordMouseMove','needShowKey','needRecordKey','ctrlState'
]
// 转换字符串为数字或boolean
function str2Type(hash){
  for(let k in hash)
  {
    if( boolArr.indexOf(k)>-1){
      hash[k] = hash[k] == 1
    }else if(/^-?\d+$/.test(hash[k])){
      hash[k] = Number(hash[k]);
    }
  }
}
// 拉取数据
async function loadPara(allPara,allFontRef,keyMappingRef,skipRecordRef,ctrlListRef,skipShowRef,screenNum,screenInfo){
  const store= useAustinStore();
  let data = <any>store.setting;
  if(data.config==null){
  // 每次路由跳转变量会重新初始化，需要保存起来
    data = await ajax('getPara')
    store.setting = data
  }
  
  const sinfo = data.infoPC.screen; // [{Left:0, Top:0, Right:100, Bottom:200},{Left:0, Top:0, Right:100, Bottom:200}]
  screenInfo.value = sinfo
  screenNum.value = toVSelectList( [...Array(sinfo.length).keys()].map(x => x + 1) )
  
  // 对配置文件中 data.config 的数字字符串转换为数字
  str2Type(data.config.common)
  str2Type(data.config.dialog)
  
  allPara.value = data.config;
  console.log('data.config',data.config)
  allFontRef.value = toVSelectList( data.fonts.map(x => x.replace(/"/g,'')))
  keyMappingRef.value = toKVList(data.keyList)
  skipRecordRef.value = splitArr(allPara.value.common.skipRecord)
  ctrlListRef.value = splitArr(allPara.value.dialog.ctrlList)
  skipShowRef.value = splitArr(allPara.value.dialog.skipShow)

}
// ajax核心模块
async function ajax(path,data=null){
  console.log('ajax')
  // 测试环境
  let port = location.port
  if(port == '3000')
  {
    port = '9900' // 调试阶段
  }
  const headers = {
    "Content-Type": "application/json",
  };
  if(data == null){
    data = ''
  }else if( typeof data !='string'){
    data = JSON.stringify(data)
  }
  let rsp = await fetch(`http://127.0.0.1:${port}/${path}`,  {
      method: "POST",
      headers: headers,
      body: data})
  let result = await rsp.json();
  return result
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
    const allPara = ref<any>({})
    const allFontRef    = ref([]) // 字体清单
    const keyMappingRef = ref([])  // 按键匹配清单
    const skipRecordRef = ref([])
    const ctrlListRef   = ref([])
    const skipShowRef   = ref([])
    const screenInfo    = ref('')  // 屏幕数据
    const screenNum     = ref('')  // 屏幕
    // 00 高位表示显示，低位表示记录，
    const showRecEvent =[{ label:'不显示不记录',value:0 },{ label:'不显示但记录',value:1 /*01*/},{ label:'只显示不记录',value:2 /*10*/},{ label:'既显示也记录',value:3 /*10*/}]
    console.log('setup')
    loadPara(allPara,allFontRef,keyMappingRef,skipRecordRef,ctrlListRef,skipShowRef,screenNum,screenInfo)

    const handleShowMessage = () => {
      console.log('I can use message')
    }

    onMounted(() => {
      //scrollToSection();
    })
    //let preAnchor = null
    let  myBorder = ref({
      General:true,
      KeyUI:false,
      StatPara:false,
      KeyMap:false,
      Save:false,
    })
    const scrollTo = (href: string) => {
      for(let k in myBorder.value)
      {
        myBorder.value[k] =  false;
      }
      myBorder.value[href] = true
      // 原始组件会自动滚动到目标位置，先还原回去
      // if(preAnchor!=null)
      // {
      //   document.getElementById(preAnchor).scrollIntoView()  // 先滚回原来的位置
      // }
      // document.getElementById(href).scrollIntoView({ behavior: 'smooth' })
      // console.log( href + '---')
      // preAnchor = href
    }
    const containerRef = ref<HTMLElement | undefined>(undefined)
    return {
      showRecEvent,
      myBorder,
      scrollTo,
      containerRef,
      handleShowMessage,
      allPara,
      skipRecordRef,
      contentText,
      ctrlListRef,
      skipShowRef,
      allFontRef,
      screenNum,
      keyMappingRef,
      screenInfo,
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
.n-anchor-link{
  font-size: 16px;
}
body {
  scroll-behavior: smooth; /* 添加平滑滚动效果 */
}
</style>