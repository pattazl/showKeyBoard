<template>
  <div>
    <div style="height: 120px ;margin-right: 90px;float: right;">
      <n-anchor
        affix
        :top="80"
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
        {{ contentText.intro1 }}
        <n-list hoverable v-if="allPara.common">
        <n-list-item> {{ contentText.intro2 }}<n-dynamic-tags v-model:value="skipRecordRef" />
        </n-list-item>
        <n-list-item>
          {{ contentText.intro3 }}
           <div class="intro">{{ contentText.intro4 }}</div>
           <template #suffix>
            <n-switch :round="false" v-model:value="allPara.common.skipCtrlKey"/> 
          </template>
        </n-list-item>
        <n-list-item>{{ contentText.intro5 }}
          <template #suffix>
            <n-select v-model:value="allPara.common.showMouseEvent" 
            :options="[{ label:contentText.intro63,value:0 /* 00 高位表示显示，低位表示记录 */ },{ label:contentText.intro64,value:1 /*01*/},
            { label:contentText.intro65,value:2 /*10*/},{ label:contentText.intro66,value:3 /*10*/}]" />
          </template>
        </n-list-item>
        <n-list-item>{{ contentText.intro6 }} 
          <template #suffix>
            <n-switch :round="false" v-model:value="allPara.common.recordMouseMove"/> 
          </template>
        </n-list-item>
        <n-list-item>{{ contentText.intro7 }} 
          <template #suffix>
            <n-switch :round="false" v-model:value="allPara.common.needShowKey"/> 
          </template>
        </n-list-item>
        <n-list-item>{{ contentText.intro8 }}  
          <template #suffix>
            <n-switch :round="false" v-model:value="allPara.common.needRecordKey"/> 
          </template>
        </n-list-item>
        <n-list-item>{{ contentText.intro9 }} 
          <div class="intro">{{ contentText.intro10 }}</div>
          <template #suffix>
            <n-switch :round="false" v-model:value="allPara.common.ctrlState"/> 
          </template>
        </n-list-item>
        <n-list-item>{{ contentText.intro11 }}
          <template #suffix>
            <n-input-number v-model:value="allPara.common.serverPort" :min="80" :max="65535" />
          </template>
        </n-list-item>
        <n-list-item>{{ contentText.intro12 }} 
          <template #suffix>
            <n-input v-model:value="allPara.common.activeWindowProc" type="text" :placeholder="contentText.intro13" />
          </template>
        </n-list-item>
        <n-list-item>{{ contentText.intro14 }}  
          <template #suffix>
            <div class="intro">{{ contentText.intro15 }}{{ contentText.keyIntro }}</div>
            <n-input v-model:value="allPara.common.hotkey4Show" type="text" :placeholder="contentText.intro16" />
          </template>
        </n-list-item>
      </n-list>
      </n-card>
      <h2 id="KeyUI">{{ contentText?.menu?.setting2 }}</h2>
      <n-card :style="myBorder.KeyUI?'border:1px #18a058 solid':''">
        {{ contentText.intro17 }}
        <n-list hoverable v-if="allPara.dialog">
          <n-list-item>{{ contentText.intro44 }}<div :class="screenInfo.length>0?'intro':'error'">{{ screenInfo.length>0?contentText.intro45:contentText.intro72 }} {{ screenInfo[allPara.dialog.guiMonitorNum-1] }}</div>
            <template #suffix>
              <n-select v-model:value="allPara.dialog.guiMonitorNum"
                :options="screenNum" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro18 }}
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiWidth" :min="1" :max="65535" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro19 }}<div class="intro">{{ contentText.intro20 }}</div>
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiHeigth" :min="0" :max="65535" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro21 }}
            <template #suffix>
              <n-color-picker v-model:value="allPara.dialog.guiBgcolor" :modes="['hex']" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro22 }}<div class="intro">{{ contentText.intro23 }}</div>
            <template #suffix>
              <n-switch v-model:value="allPara.dialog.guiTrans" :round="false" /> 
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro24 }}<div class="intro">{{ contentText.intro25 }}</div>
            <template #suffix>
                <n-select v-model:value="allPara.dialog.guiTextFont" :options="allFontRef" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro26 }}
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiTextSize" :min="1" :max="100" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro27 }}
            <template #suffix>
              <n-select v-model:value="allPara.dialog.guiTextWeight" 
              :options="[{ label:contentText.intro67,value:'norm' }, { label:contentText.intro68,value:'bold' },{ label:contentText.intro69,value:'italic'},
              { label:contentText.intro70,value:'strike'},{ label:contentText.intro71,value:'underline' }]" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro28 }}
            <template #suffix>
              <n-color-picker v-model:value="allPara.dialog.guiTextColor" :show-alpha="false" :modes="['hex']" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro29 }}
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiLife" :min="10" :max="100000" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro30 }}
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiInterval" :min="10" :max="100000" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro31 }}<div class="intro">{{ contentText.intro32 }}</div>
            <template #suffix>
                <n-select v-model:value="allPara.dialog.guiPos"
                :options="[{label:contentText.intro33,value:'TL'},{label:contentText.intro34,value:'TR'},{label:contentText.intro35,value:'BL'},{label:contentText.intro36,value:'BR'}]" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro37 }}<div class="intro">{{ contentText.intro38 }}</div>
            <template #suffix>
                <n-select v-model:value="allPara.dialog.guiPosXY" :options="[{label:'X'},{label:'Y'}]" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro39 }}<div class="intro">{{ contentText.intro40 }}</div>
            <template #suffix>
                <n-input-number v-model:value="allPara.dialog.guiPosOffsetX" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro41 }}<div class="intro">{{ contentText.intro42 }}</div>
            <template #suffix>
                <n-input-number v-model:value="allPara.dialog.guiPosOffsetY" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro43 }}
            <template #suffix>
                <n-switch v-model:value="allPara.dialog.guiDpiscale" :round="false" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro46 }}
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.guiMargin" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro47 }}
            <template #suffix>
                <n-switch v-model:value="allPara.dialog.guiEdge" :round="false" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro48 }}
            <template #suffix>
              <n-input v-model:value="allPara.dialog.txtSplit" type="text" :placeholder="contentText.intro49" />
            </template>
          </n-list-item>
          <h4>{{ contentText.intro50 }}</h4>
          <n-list-item>{{ contentText.intro51 }}
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.ctrlX" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro52 }}
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.ctrlY" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro53 }}
            <template #suffix>
              <n-input-number v-model:value="allPara.dialog.ctrlTextSize" :min="1" :max="100" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro54 }}
              <n-dynamic-tags v-model:value="ctrlListRef" />
          </n-list-item>
          <n-list-item>{{ contentText.intro55 }}
              <n-dynamic-tags v-model:value="skipShowRef" />
          </n-list-item>
        </n-list>
        
      </n-card>
      <h2 id="StatPara">{{ contentText?.menu?.setting3 }}</h2>
      <n-card :style="myBorder.StatPara?'border:1px #18a058 solid':''">
        {{ contentText.intro56 }}
        <n-list hoverable>
          <n-list-item>{{ contentText.intro57 }}<div class="intro">{{ contentText.intro58 }}</div>
            <template #suffix>
              <n-input-number :value="24" :min="1" :max="1000" />
            </template>
          </n-list-item>
          <n-list-item>{{ contentText.intro59 }}<div class="intro">{{ contentText.intro60 }}</div>
            <template #suffix>
              <n-input-number :value="1000" :min="100" :max="100000" />
            </template>
          </n-list-item>
        </n-list>
      </n-card>
      <h2 id="KeyMap">{{ contentText?.menu?.setting4 }}</h2>
      <n-card :style="myBorder.KeyMap?'border:1px #18a058 solid':''">
        <n-dynamic-input
          v-model:value="keyMappingRef"
          preset="pair"
          :key-placeholder="contentText.intro61"
          :value-placeholder="contentText.intro62"
        />
      </n-card>
      <h2 id="Save">{{ contentText?.menu?.setting5 }}</h2>
      <n-card :title="'差异如下'" :style="myBorder.Save?'border:1px #18a058 solid':''">
        <n-button type="primary" @click="savePara">保存数据</n-button>
        <div v-if="allData?.data">
           <n-input
            :value="JSON.stringify(allData.data,null,2)"
            type="textarea"
            placeholder="基本的 Textarea"
            show-count
            size="large"
            rows="50"
          />
            <hr/>
            <n-input
            :value="JSON.stringify(allData.preData,null,2)"
            type="textarea"
            placeholder="基本的 Textarea"
            show-count
            size="large"
            rows="50"
          />
       </div>
      </n-card>

    </n-space>
  </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted ,PropType,ref,computed } from 'vue'
import { useRoute } from 'vue-router';
import { useMessage  } from 'naive-ui';
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
function arr2Str(str){
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
const boolArr =['skipCtrlKey','recordMouseMove','needShowKey','needRecordKey','ctrlState','guiBgTrans','guiTrans', 'guiEdge' ,'guiDpiscale']
// 转换字符串为数字或boolean
function str2Type(hash,flag){
  for(let k in hash)
  {
    if(flag==0){  // 进行数据转换给界面
        // 如果不是字符串类型，证明之前已经处理过无需再处理
        if(typeof hash[k] !=='string'){
          return;
        }
        if( boolArr.indexOf(k)>-1 ){
          hash[k] = hash[k] == 1
        }else if('guiBgcolor' == k && hash[k].indexOf('#') ==-1 ){
          // 有颜色字段,需要加上透明度
          hash[k]= '#' + hash[k]+ parseInt(hash.guiOpacity,10).toString(16)
        }else if('guiTextColor' == k && hash[k].indexOf('#') ==-1 ){
          // 有颜色字段
          hash[k]= '#' + hash[k]
        }else if(/^-?\d+$/.test(hash[k])){
          hash[k] = Number(hash[k]);
        }
    }else{ // 界面转换给数据
      if( boolArr.indexOf(k)>-1 ){
          hash[k] = hash[k]?'1':'0'
        }else if('guiBgcolor' == k && hash[k].indexOf('#') !=-1 ){
          // 有颜色字段,需要加上透明度
          let color = hash[k].replace(/#/,'')
          hash[k]= color.substr(0,6);
          hash.guiOpacity = parseInt(color.substr(6,2),16)
          hash.guiBgTrans = (hash.guiOpacity ==0)?1:0
        }else if('guiTextColor' == k ){
          // 有颜色字段
          hash[k]= hash[k].replace(/#/,'')
        }
    }
  }
}
function deepCopy(obj) {
  if (typeof obj !== 'object' || obj === null) {
    return obj;
  }
  const copy = Array.isArray(obj) ? [] : {};
  for (let key in obj) {
    if (Object.prototype.hasOwnProperty.call(obj, key)) {
      copy[key] = deepCopy(obj[key]);
    }
  }
  return copy;
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
    const message = useMessage()
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
    const screenNum     = ref([])  // 屏幕

    console.log('setup')
    const allData = {data:{},preData:{}}; // 重新更新数据
    // 拉取数据
    async function loadPara(){
      const store= useAustinStore();
      let data = <any>store.setting;
      if(data.config==null){
        const loading = message.loading(contentText.value.intro73,{duration:0})
      // 每次路由跳转变量会重新初始化，需要保存起来
        data = await ajax('getPara')
        loading.destroy()
        message.success( contentText.value.intro74)
        // 对配置文件中 data.config 的数字字符串转换为数字
        str2Type(data.config.common,0)
        str2Type(data.config.dialog,0)

        store.setting = data  // 在 setting中保留一份数据,进行页面切换后无需重新载入，除非页面整个刷新
        store.preData = deepCopy(data); // 之前的数据
      }
      allData.data = data;
      allData.preData = store.preData;

      // 以下为输出
      allPara.value = data.config;
      const sinfo = data.infoPC?.screen; // [{Left:0, Top:0, Right:100, Bottom:200},{Left:0, Top:0, Right:100, Bottom:200}]
      if(sinfo != null){
        screenInfo.value = sinfo
        screenNum.value = toVSelectList( [...Array(sinfo.length).keys()].map(x => x + 1) )
      }
      console.log('data.config',data.config)
      allFontRef.value = toVSelectList( data.fonts.map(x => x.replace(/"/g,'')))
      keyMappingRef.value = toKVList(data.keyList)
      skipRecordRef.value = splitArr(allPara.value.common.skipRecord)
      ctrlListRef.value = splitArr(allPara.value.dialog.ctrlList)
      skipShowRef.value = splitArr(allPara.value.dialog.skipShow)

    }
    loadPara();
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
    // 将数据进行转换和保存 allPara,keyMappingRef,skipRecordRef,ctrlListRef,skipShowRef
    async function savePara(){
      let config = deepCopy(allPara.value); // config配置文件，包含common 和 dialog
      str2Type(config.common,1)
      str2Type(config.dialog,1)

      // 转换数组为字符串
      config.common.skipRecord = skipRecordRef.value.join('|')
      config.dialog.ctrlList   = ctrlListRef.value.join('|')
      config.dialog.skipShow = skipShowRef.value.join('|')
      // 转换keyList
      let keyList = {};
      for( let v of keyMappingRef.value)
      {
        keyList[v.key] = v.value;
      }
      // 需要将数据保存给服务器
      console.log('setPara')
      const saving = message.loading(contentText.value.intro75,{duration:0})
      await ajax('setPara',{config,keyList})
      saving.destroy()
      message.success(contentText.value.intro76)
    }
    return {
      myBorder,
      scrollTo,
      allPara,
      skipRecordRef,
      contentText,
      ctrlListRef,
      skipShowRef,
      allFontRef,
      screenNum,
      keyMappingRef,
      screenInfo,
      allData,
      savePara,
    }
  },
})
</script>
<style scoped>
.intro {
  font-size:smaller;
  color:#2080f0;;
}
.error {
  font-size:smaller;
  color:#f02020;;
}
.n-input-number{
  width:150px
}
.n-color-picker , .n-select {
  width:220px;
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