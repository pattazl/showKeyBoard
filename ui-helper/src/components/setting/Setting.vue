<template>
  <div>
    <div style="height: 120px ;margin-right: 90px;float: right;">
      <n-anchor affix :top="80" style="z-index: 10; font-size: 18px; " :bound="50" :show-rail="false" :ignore-gap="true"
        type='block' position='fix'>
        <n-anchor-link :title="contentText?.menu?.setting1" @click.prevent="scrollTo('General')" href="#General" />
        <n-anchor-link :title="contentText?.menu?.setting2" @click.prevent="scrollTo('KeyUI')" href="#KeyUI" />
        <n-anchor-link :title="contentText?.menu?.setting3" @click.prevent="scrollTo('StatPara')" href="#StatPara" />
        <n-anchor-link :title="contentText?.menu?.setting4" @click.prevent="scrollTo('KeyMap')" href="#KeyMap" />
        <n-anchor-link :title="contentText?.menu?.setting5" @click.prevent="scrollTo('Save')" href="#Save" />
      </n-anchor>
    </div>
    <div>
      <n-space vertical>
        <h2 id="General">{{ contentText?.menu?.setting1 }}</h2>
        <n-card :style="myBorder.General ? 'border:1px #18a058 solid' : ''">
          {{ contentText.intro1 }}
          <n-list hoverable v-if="allPara.common">
            <n-list-item> {{ contentText.intro2 }}<n-dynamic-tags v-model:value="skipRecordRef" />
            </n-list-item>
            <n-list-item>
              {{ contentText.intro3 }}
              <div class="intro">{{ contentText.intro4 }}</div>
              <template #suffix>
                <n-switch :round="false" v-model:value="allPara.common.skipCtrlKey" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro5 }}
              <template #suffix>
                <n-select v-model:value="allPara.common.showMouseEvent" :options="[{ label: contentText.intro63, value: 0 /* 00 高位表示显示，低位表示记录 */ }, { label: contentText.intro64, value: 1 /*01*/ },
                { label: contentText.intro65, value: 2 /*10*/ }, { label: contentText.intro66, value: 3 /*10*/ }]" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro6 }}
              <template #suffix>
                <n-switch :round="false" v-model:value="allPara.common.recordMouseMove" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro7 }}
              <template #suffix>
                <n-switch :round="false" v-model:value="allPara.common.needShowKey" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro8 }}
              <template #suffix>
                <n-switch :round="false" v-model:value="allPara.common.needRecordKey" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro9 }}
              <div class="intro">{{ contentText.intro10 }}</div>
              <template #suffix>
                <n-switch :round="false" v-model:value="allPara.common.ctrlState" />
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
        <n-card :style="myBorder.KeyUI ? 'border:1px #18a058 solid' : ''">
          {{ contentText.intro17 }}
          <n-list hoverable v-if="allPara.dialog">
            <n-list-item>{{ contentText.intro44 }}<div :class="screenInfo.length > 0 ? 'intro' : 'error'">{{
              screenInfo.length > 0 ? contentText.intro45 : contentText.intro72 }} {{ screenInfo[allPara.dialog.guiMonitorNum - 1] }}</div>
              <template #suffix>
                <n-select v-model:value="allPara.dialog.guiMonitorNum" :options="screenNum" />
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
                  :options="[{ label: contentText.intro67, value: 'norm' }, { label: contentText.intro68, value: 'bold' }, { label: contentText.intro69, value: 'italic' },
                  { label: contentText.intro70, value: 'strike' }, { label: contentText.intro71, value: 'underline' }]" />
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
                  :options="[{ label: contentText.intro33, value: 'TL' }, { label: contentText.intro34, value: 'TR' }, { label: contentText.intro35, value: 'BL' }, { label: contentText.intro36, value: 'BR' }]" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro37 }}<div class="intro">{{ contentText.intro38 }}</div>
              <template #suffix>
                <n-select v-model:value="allPara.dialog.guiPosXY" :options="[{ label: 'X' }, { label: 'Y' }]" />
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
        <n-card :style="myBorder.StatPara ? 'border:1px #18a058 solid' : ''">
          {{ contentText.intro56 }}
          <n-list hoverable>
            <n-list-item>{{ contentText.intro57 }}<div class="intro">{{ contentText.intro58 }}</div>
              <template #suffix>
                <n-input-number v-model:value="dataSetting.screenSize" :min="1" :max="1000" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro59 }}<div class="intro">{{ contentText.intro60 }}</div>
              <template #suffix>
                <n-input-number v-model:value="dataSetting.mouseDPI" :min="100" :max="100000" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro99 }}<div class="intro">{{ contentText.intro100 }}</div>
              <template #suffix>
                <n-select v-model:value="dataSetting.keymap" :options="keymapsRef" @update:value="handleUpdateValue"/>
                <n-input
                v-model:value="mapDetailRef"
            type="textarea"
            placeholder="keymap detail"
            show-count
            size="medium"
            rows="10"
          />
              </template>
              
            </n-list-item>
          </n-list>
        </n-card>
        <h2 id="KeyMap">{{ contentText?.menu?.setting4 }}</h2>
        <n-card :style="myBorder.KeyMap ? 'border:1px #18a058 solid' : ''">
          <div class="intro">{{ contentText.keyIntro }}</div>
          <n-dynamic-input v-model:value="keyMappingRef" preset="pair" :key-placeholder="contentText.intro61"
            :value-placeholder="contentText.intro62" />
        </n-card>
        <h2 id="Save">{{ contentText?.menu?.setting5 }}</h2>
        <n-card :style="myBorder.Save ? 'border:1px #18a058 solid' : ''">
          <n-space style="margin:3px">
            <n-button type="warning" @click="resetPara">{{ contentText?.intro81 }}</n-button>
            <n-button type="primary" @click="savePara">{{ contentText?.intro82 }}</n-button>
            <n-tag :type="Object.keys(diffJsonList).length > 0 ? 'error' : 'success'">{{ Object.keys(diffJsonList).length
              > 0 ?
              contentText?.intro83 : contentText?.intro84 }}</n-tag>
          </n-space>
          <code-diff v-for="(item, key) in diffJsonList" :lang="lang" :key="key" :old-string="item['old']"
            :new-string="item['new']" :context="50" :file-name="key" output-format="side-by-side"
            :file-content-toggle="true" :render-nothing-when-empty="false" />
        </n-card>

      </n-space>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, PropType, ref, computed } from 'vue'
import { useMessage } from 'naive-ui';
import content from '../../content.js';
import mapping from '../../mapping.js';
import { storeToRefs } from 'pinia'
import { useAustinStore } from '../../App.vue'
import { deepCopy, ajax, str2Type, splitArr } from '@/common.ts'
import CodeDiff from './CodeDiff.vue'

// 生成界面上select的数组
function toVSelectList(arr: Array<string | number>) {
  let resArr = [];
  for (let v of arr) {
    resArr.push({ label: v, value: v })
  }
  return resArr
}
// 生成界面上key,value数组
function toKVList(obj: {}) {
  let resArr = [];
  for (let k in obj) {
    resArr.push({ key: k, value: obj[k] })
  }
  return resArr
}
// 功能和 toKVList 相反
function KVListTo(arr: Array<any>) {
  let keyList = {};
  for (let v of arr) {
    keyList[v.key] = v.value;
  }
  return keyList
}

// 获取不同节点的差异数据
function getDiffHash(hash/*out */, hash1, hash2, named, contentText) {
  let oldArr = []
  let newArr = []
  if (typeof hash1 == 'string') {
    if (hash1 != hash2) {
      hash[named] = { 'old': hash1, 'new': hash2 }
    }
  } else {
    for (let k in hash1) {
      let before = hash2[k]
      let after = hash1[k]
      let keyName = mapping[k] == null ? k : (contentText[mapping[k]] ?? k);
      if (before != after) {
        oldArr.push(`${keyName}: ${before}`)
        newArr.push(`${keyName}: ${after}`)
      }
    }
    if (oldArr.length > 0) {
      hash[named] = { 'old': oldArr.join('\n'), 'new': newArr.join('\n') }
    }
  }
}
export default defineComponent({
  name: 'Setting',
  props: {
    lang: {
      type: String as PropType<'en-US' | 'zh-CN'>,
    },
  },
  setup(props) {
    const store= useAustinStore();  // 可通过 属性传递，也可通过pinia传递
    // const lang = computed(()=>store.lang) 
    const contentText = computed(() => content[props.lang])
    const message = useMessage()
    //watch(() => store.lang, (newValue, oldValue) => {
    //  console.log(` lang New value: ${newValue}, old value: ${oldValue}`);
    //});

    // 各类数据模拟
    const allPara = ref<any>({})
    const allFontRef = ref([]) // 字体清单
    const keyMappingRef = ref([])  // 按键匹配清单
    const skipRecordRef = ref([])
    const ctrlListRef = ref([])
    const skipShowRef = ref([])
    const screenInfo = ref('')  // 屏幕数据
    const screenNum = ref([])  // 屏幕
    const dataSetting = ref<any>({})  // 屏幕
    const keymapsRef = ref([])  // 选择的键盘清单
    const mapDetailRef = ref('')  // 键盘详细定义

    console.log('setup')
    // const allData = { data: {}, preData: {} }; // 重新更新数据
    // 拉取数据
    function loadPara() {
      let data = <any>store.data;
      //allData.data = data;
      //allData.preData = store.preData;

      // 以下为输出
      allPara.value = data.config;
      dataSetting.value = data.dataSetting;
      keymapsRef.value = data.keymaps.map(x=>{ return {label:x.mapName,value:x.mapName}});
      handleUpdateValue(data.dataSetting.keymap)

      const sinfo = data.infoPC?.screen; // [{Left:0, Top:0, Right:100, Bottom:200},{Left:0, Top:0, Right:100, Bottom:200}]
      if (sinfo != null) {
        screenInfo.value = sinfo
        screenNum.value = toVSelectList([...Array(sinfo.length).keys()].map(x => x + 1))
      }
      //console.log('data.config', data.config)
      allFontRef.value = toVSelectList(data.fonts.map(x => x.replace(/"/g, '')))
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
    let myBorder = ref({
      General: true,
      KeyUI: false,
      StatPara: false,
      KeyMap: false,
      Save: false,
    })
    const scrollTo = (href: string) => {
      for (let k in myBorder.value) {
        myBorder.value[k] = false;
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
    // 还原数据
    function resetPara() {
      store.data = deepCopy(store.preData);
      loadPara()
    }
    // 将数据进行转换和保存 allPara,keyMappingRef,skipRecordRef,ctrlListRef,skipShowRef
    async function savePara() {
      let config = deepCopy(allPara.value); // config配置文件，包含common 和 dialog
      if (config?.common == null) {
        return
      }
      str2Type(config.common, 1)
      str2Type(config.dialog, 1)

      // 转换数组为字符串
      config.common.skipRecord = skipRecordRef.value.join('|')
      config.dialog.ctrlList = ctrlListRef.value.join('|')
      config.dialog.skipShow = skipShowRef.value.join('|')
      // 转换keyList
      let keyList = KVListTo(keyMappingRef.value);
      // 需要将数据保存给服务器
      console.log('setPara')
      const saving = message.loading(contentText.value.intro75, { duration: 0 })
      await ajax('setPara', { config, keyList,'dataSetting':dataSetting.value })
      saving.destroy()
      message.success(contentText.value.intro76)
      
      // 设置变量为新的数据
      store.preData.keyList = deepCopy(keyList)
      store.preData.dataSetting = deepCopy(dataSetting.value)
    }
    // 计算差异
    const diffJsonList = computed(() => {
      const predata = <any>store.preData
      let hash = {}
      if (allPara.value == null || predata.config == null) {
        return hash
      }
      let keyList = KVListTo(keyMappingRef.value);
      // 对比 config.common ，config.dialog ,store.preData.keyList
      getDiffHash(hash, allPara.value.common, predata.config.common, contentText.value.menu?.setting1, contentText.value)
      getDiffHash(hash, allPara.value.dialog, predata.config.dialog, contentText.value.menu?.setting2, contentText.value)
      getDiffHash(hash, dataSetting.value, predata.dataSetting, contentText.value.menu?.setting3, contentText.value)
      //getDiffHash(hash,config.common,predata.config.common,contentText.value.menu?.setting3)
      getDiffHash(hash, JSON.stringify(keyList, null, 2), JSON.stringify(predata.keyList, null, 2), contentText.value.menu?.setting4, contentText.value)
      return hash
    })
    // 根据选项显示文本
    function handleUpdateValue(value: string){
      console.log(value)
      const data = <any>store.data
      for(let v of data.keymaps)
      {
        if(v.mapName == value){
          mapDetailRef.value = v.mapDetail
          break
        }
      }
    }
    return {
      myBorder,
      scrollTo,
      allPara,
      dataSetting,
      skipRecordRef,
      contentText,
      ctrlListRef,
      skipShowRef,
      allFontRef,
      screenNum,
      keyMappingRef,
      screenInfo,
      //allData,
      savePara,
      diffJsonList,
      resetPara,
      keymapsRef,
      handleUpdateValue,
      mapDetailRef,
    }
  },
})
</script>
<style scoped>
.intro {
  font-size: smaller;
  color: #2080f0;
  ;
}

.error {
  font-size: smaller;
  color: #f02020;
  ;
}

.n-input-number {
  width: 150px
}

.n-color-picker,
.n-select {
  width: 220px;
}

.n-input {
  width: 300px;
}

.n-anchor-link {
  font-size: 16px;
}

body {
  scroll-behavior: smooth;
  /* 添加平滑滚动效果 */
}
</style>