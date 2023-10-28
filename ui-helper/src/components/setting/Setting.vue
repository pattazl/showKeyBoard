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
          <n-list hoverable v-if="allConfig.common">
            <n-list-item> {{ contentText.intro2 }}<n-dynamic-tags v-model:value="skipRecordRef" />
            </n-list-item>
            <n-list-item>{{ contentText.intro7 }}
              <template #suffix>
                <n-switch :round="false" v-model:value="allConfig.common.needShowKey" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro9 }}
              <div class="intro">{{ contentText.intro10 }}</div>
              <template #suffix>
                <n-switch :round="false" v-model:value="allConfig.common.ctrlState" />
              </template>
            </n-list-item>
            <n-list-item>
              {{ contentText.intro3 }}
              <div class="intro">{{ contentText.intro4 }}</div>
              <template #suffix>
                <n-switch :round="false" v-model:value="allConfig.common.skipCtrlKey" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro8 }}
              <template #suffix>
                <n-switch :round="false" v-model:value="allConfig.common.needRecordKey" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro6 }}
              <template #suffix>
                <n-switch :round="false" v-model:value="allConfig.common.recordMouseMove" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro5 }}
              <template #suffix>
                <n-select v-model:value="allConfig.common.showMouseEvent" :options="[{ label: contentText.intro63, value: 0 /* 00 高位表示显示，低位表示记录 */ }, { label: contentText.intro64, value: 1 /*01*/ },
                { label: contentText.intro65, value: 2 /*10*/ }, { label: contentText.intro66, value: 3 /*10*/ }]" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro11 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.common.serverPort" :min="80" :max="65535" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro12 }}
              <template #suffix>
                <n-input v-model:value="allConfig.common.activeWindowProc" type="text"
                  :placeholder="contentText.intro13" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro14 }}
              <template #suffix>
                <div class="intro">{{ contentText.intro15 }}{{ contentText.keyIntro }}</div>
                <n-input v-model:value="allConfig.common.hotkey4Show" type="text" :placeholder="contentText.intro16" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro131 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.common.maxKeypressCount" :min="1" :max="500" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro132 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.common.maxCtrlpressCount" :min="1" :max="500" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro134 }}
              <div class="error" v-if="allConfig.common.remoteType!=0">{{ contentText.intro139 }}
              <div v-for="(link, index) in IPlinks" > <a :key="index" :href="link" target="blank">{{ link }}</a></div>
              </div>
              <template #suffix>
                <n-select v-model:value="allConfig.common.remoteType" :options="[{ label: contentText.intro135, value: 0 /* 00 高位表示显示，低位表示记录 */ }, { label: contentText.intro136, value: 1 /*01*/ },
                { label: contentText.intro137, value: 2 /*10*/ }]" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro133 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.common.autoSave2Db" :min="10" :max="7200" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro138 }}
              <template #suffix>
                <n-switch :round="false" v-model:value="allConfig.common.showHttpDebug" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro140 }}
              <div class="intro">{{ contentText.intro141 }}</div>
              <template #suffix>
                <n-switch :round="false" v-model:value="allConfig.common.hideInWinPwd" />
              </template>
            </n-list-item>
          </n-list>
        </n-card>
        <h2 id="KeyUI">{{ contentText?.menu?.setting2 }}</h2>
        <n-card :style="myBorder.KeyUI ? 'border:1px #18a058 solid' : ''">
          {{ contentText.intro17 }}
          <n-list hoverable v-if="allConfig.dialog">
            <n-list-item>{{ contentText.intro44 }}<div :class="screenInfo.length > 0 ? 'intro' : 'error'">{{
              screenInfo.length > 0 ? contentText.intro45 : contentText.intro72 }} {{
    screenInfo[allConfig.dialog.guiMonitorNum - 1] }}</div>
              <template #suffix>
                <n-select v-model:value="allConfig.dialog.guiMonitorNum" :options="screenNum" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro18 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.guiWidth" :min="1" :max="65535" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro19 }}<div class="intro">{{ contentText.intro20 }}</div>
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.guiHeigth" :min="0" :max="65535" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro21 }}
              <template #suffix>
                <n-color-picker v-model:value="guiBgcolorRef" :modes="['hex']" @update:value="handleUpdateColor" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro22 }}<div class="intro">{{ contentText.intro23 }}</div>
              <template #suffix>
                <n-switch v-model:value="allConfig.dialog.guiTrans" :round="false" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro24 }}<div class="intro">{{ contentText.intro25 }}</div>
              <template #suffix>
                <n-select v-model:value="allConfig.dialog.guiTextFont" :options="allFontRef" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro26 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.guiTextSize" :min="1" :max="100" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro27 }}
              <template #suffix>
                <n-select v-model:value="allConfig.dialog.guiTextWeight"
                  :options="[{ label: contentText.intro67, value: 'norm' }, { label: contentText.intro68, value: 'bold' }, { label: contentText.intro69, value: 'italic' },
                  { label: contentText.intro70, value: 'strike' }, { label: contentText.intro71, value: 'underline' }]" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro28 }}
              <template #suffix>
                <n-color-picker v-model:value="guiTextColorRef" :show-alpha="false" :modes="['hex']" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro29 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.guiLife" :min="10" :max="100000" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro30 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.guiInterval" :min="10" :max="100000" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro31 }}<div class="intro">{{ contentText.intro32 }}</div>
              <template #suffix>
                <n-select v-model:value="allConfig.dialog.guiPos"
                  :options="[{ label: contentText.intro33, value: 'TL' }, { label: contentText.intro34, value: 'TR' }, { label: contentText.intro35, value: 'BL' }, { label: contentText.intro36, value: 'BR' }]" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro37 }}<div class="intro">{{ contentText.intro38 }}</div>
              <template #suffix>
                <n-select v-model:value="allConfig.dialog.guiPosXY" :options="[{ label: 'X' }, { label: 'Y' }]" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro39 }}<div class="intro">{{ contentText.intro40 }}</div>
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.guiPosOffsetX" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro41 }}<div class="intro">{{ contentText.intro42 }}</div>
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.guiPosOffsetY" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro43 }}
              <template #suffix>
                <n-switch v-model:value="allConfig.dialog.guiDpiscale" :round="false" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro46 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.guiMargin" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro47 }}
              <template #suffix>
                <n-switch v-model:value="allConfig.dialog.guiEdge" :round="false" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro48 }}
              <template #suffix>
                <n-input v-model:value="allConfig.dialog.txtSplit" type="text" :placeholder="contentText.intro49" />
              </template>
            </n-list-item>
            <h4>{{ contentText.intro50 }}</h4>
            <n-list-item>{{ contentText.intro51 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.ctrlX" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro52 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.ctrlY" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro53 }}
              <template #suffix>
                <n-input-number v-model:value="allConfig.dialog.ctrlTextSize" :min="1" :max="100" />
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
            <n-list-item>{{ contentText.intro118 }}
              <template #suffix>
                <n-input-number v-model:value="dataSetting.topN" :min="3" :max="200" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro145 }}
              <template #suffix>
                <n-input-number v-model:value="dataSetting.appTopN" :min="1" :max="200" />
              </template>
            </n-list-item>
            <n-list-item>{{ contentText.intro148 }}
                <n-dynamic-input v-model:value="appNameListRef" preset="pair" :key-placeholder="contentText.intro146"
                :value-placeholder="contentText.intro147" />
            </n-list-item>
            <n-list-item>{{ contentText.intro99 }}<div class="intro" style="white-space: pre-line;">{{
              contentText.intro100 }}</div>
              <div class="error"> {{ keyboardDetail }} </div>
              <template #suffix>
                <n-select v-model:value="dataSetting.keymap" filterable tag :options="keymapsRef"
                  @update:value="handleUpdateValue" />
                <n-space>
                  <n-button type="warning" @click="keyboardApply">{{ contentText?.intro104 }}</n-button>
                  <n-button type="primary" :disabled="dataSetting.keymap == 'Default'" @click="keyboardSave">{{
                    keyboardSaveInfo }}</n-button>
                  <n-button type="error" :disabled="dataSetting.keymap == 'Default'" @click="keyboardDelete">{{
                    contentText?.intro106 }}</n-button>
                </n-space>
                <n-input v-model:value="mapDetailRef" :readonly="dataSetting.keymap == 'Default'" type="textarea"
                  placeholder="keymap detail" show-count size="medium" rows="10" @blur="handleUpdateDetail" />
              </template>
            </n-list-item>
            <div id="main" style="height: 500px; min-width: 800px;width:95%"></div>
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
          <n-space>
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
import { useMessage, useDialog } from 'naive-ui';
import content from '../../content.js';
import mapping from '../../mapping.js';
import { storeToRefs } from 'pinia'
import { useAustinStore } from '../../App.vue'
import { deepCopy, ajax, str2Type, splitArr } from '@/common.ts'
import CodeDiff from './CodeDiff.vue'

import * as echarts from 'echarts/core';
// 引入柱状图图表，图表后缀都为 Chart
import { HeatmapChart } from 'echarts/charts';
// 引入提示框，标题，直角坐标系，数据集，内置数据转换器组件，组件后缀都为 Component
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  DatasetComponent,
  TransformComponent,
  VisualMapComponent
} from 'echarts/components';
// 标签自动布局、全局过渡动画等特性
import { LabelLayout, UniversalTransition } from 'echarts/features';
// 引入 Canvas 渲染器，注意引入 CanvasRenderer 或者 SVGRenderer 是必须的一步
import { CanvasRenderer } from 'echarts/renderers';
echarts.use([
  TitleComponent,
  TooltipComponent,
  GridComponent,
  DatasetComponent,
  TransformComponent,
  LabelLayout,
  UniversalTransition,
  CanvasRenderer,
  HeatmapChart,
  VisualMapComponent
]);

let hashTxtData = {}; // 按键上显示的内容
let hashOriData = {}; // 原始定义的内容，提示框上显示
var option = {
  textStyle: {
    fontSize: 16
  },
  tooltip: {
    position: 'top'
  },
  grid: {
    height: '90%',
    top: '5%',
  },
  xAxis: {
    type: 'category',
    show: true,
    splitArea: {
      show: true
    }
  },
  yAxis: {
    type: 'category',
    show: true,
    splitArea: {
      show: true
    }
  },
  visualMap: {
    show: false,
    min: 0,
    max: 100,
    calculable: true,
    orient: 'horizontal',
    left: 'center',
    bottom: '2%'
  },
  series: [
    {
      tooltip: {
        trigger: 'item',
        textStyle: {
          fontSize: 18
        },
        confine: true,
        formatter: (p) => {
          //自定义提示信息
          //console.log(p);
          let dataCon = p.data;
          let key = dataCon[0] + ',' + dataCon[1]
          let txtCon = hashOriData[key] + '<hr> ' + dataCon[2];
          return txtCon;
        }
      },
      name: 'Keyboard',
      type: 'heatmap',
      data: {},
      label: {
        show: true,
        formatter: function (params) {
          let txt = hashTxtData[params.data[0] + ',' + params.data[1]]
          if (txt == null) {
            return 'N/A';
          } else {
            return txt;
          }
        }
      },
      emphasis: {
        itemStyle: {
          shadowBlur: 10,
          shadowColor: 'rgba(0, 0, 0, 0.5)'
        }
      }
    }
  ]
};
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
    const store = useAustinStore();  // 可通过 属性传递，也可通过pinia传递
    // const lang = computed(()=>store.lang) 
    const contentText = computed(() => content[props.lang])
    const message = useMessage()
    const myDialog = useDialog()
    //watch(() => store.lang, (newValue, oldValue) => {
    //  console.log(` lang New value: ${newValue}, old value: ${oldValue}`);
    //});
    // 各类数据模拟
    const allConfig = ref<any>({})
    const allFontRef = ref([]) // 字体清单
    const keyMappingRef = ref([])  // 按键匹配清单
    const appNameListRef = ref([])  // 应用名称匹配清单
    const skipRecordRef = ref([])
    const ctrlListRef = ref([])
    const skipShowRef = ref([])
    const screenInfo = ref('')  // 屏幕数据
    const screenNum = ref([])  // 屏幕
    const dataSetting = ref<any>({})  // 屏幕
    const keymapsRef = ref([])  // 选择的键盘清单
    const mapDetailRef = ref('')  // 键盘详细定义
    const keyboardDetail = ref('')  // 键盘定义的相关说明和提示
    const keyboardSaveInfo = ref('')    // 键盘保存按钮
    const guiBgcolorRef = ref('')    // 颜色需要额外处理
    const guiTextColorRef = ref('')    // 颜色需要额外处理
    let IPlinks = []   // IP地址清单

    let chartDom, myChart
    console.log('setup')
    // const allData = { data: {}, preData: {} }; // 重新更新数据
    // 拉取数据
    function loadPara() {
      let data = <any>store.data;
      //allData.data = data;
      //allData.preData = store.preData;

      // 以下为输出
      allConfig.value = data.config;
      dataSetting.value = data.dataSetting;
      keymapsRef.value = data.keymaps.map(x => { return { label: x.mapName, value: x.mapName } });

      const sinfo = data.infoPC?.screen; // [{Left:0, Top:0, Right:100, Bottom:200},{Left:0, Top:0, Right:100, Bottom:200}]
      if (sinfo != null) {
        screenInfo.value = sinfo
        screenNum.value = toVSelectList([...Array(sinfo.length).keys()].map(x => x + 1))
      }
      //console.log('data.config', data.config)
      allFontRef.value = toVSelectList(data.fonts.map(x => x.replace(/"/g, '')))
      keyMappingRef.value = toKVList(data.keyList)
      appNameListRef.value = toKVList(JSON.parse(data.dataSetting.appNameList))
      skipRecordRef.value = splitArr(allConfig.value.common.skipRecord)
      ctrlListRef.value = splitArr(allConfig.value.dialog.ctrlList)
      skipShowRef.value = splitArr(allConfig.value.dialog.skipShow)

      guiBgcolorRef.value = '#' + allConfig.value.dialog.guiBgcolor + parseInt(allConfig.value.dialog.guiOpacity, 10).toString(16)
      guiTextColorRef.value = '#' + allConfig.value.dialog.guiTextColor

      IPlinks =  data.networkIP.map(x=> location.origin.replace(location.host,x))
      //console.log(IPlinks)
    }
    loadPara();
    onMounted(() => {
      chartDom = document.getElementById('main');
      myChart = echarts.init(chartDom);
      // 更新键盘图和数据
      handleUpdateValue(store.data.dataSetting.keymap)
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
    // 更新颜色相关的变量
    function updateColor(dialogHash, colorVal) {
      let color = colorVal.replace(/#/, '')
      dialogHash.guiBgcolor = color.substr(0, 6);
      dialogHash.guiOpacity = parseInt(color.substr(6, 2), 16).toString()
      dialogHash.guiBgTrans = (dialogHash.guiOpacity == 0) ? '1' : '0'
    }
    // 还原数据
    function resetPara() {
      store.data = deepCopy(store.preData);
      loadPara()
    }
    // 将数据进行转换和保存 allConfig,keyMappingRef,skipRecordRef,ctrlListRef,skipShowRef
    async function savePara() {
      let config = deepCopy(allConfig.value); // config配置文件，包含common 和 dialog
      if (config?.common == null) {
        return
      }
      myDialog.warning({
        title: contentText.value.intro83,
        positiveText: contentText.value.intro109,
        negativeText: contentText.value.intro110,
        maskClosable: false,
        onPositiveClick: async () => {
          str2Type(config.common, 1)
          str2Type(config.dialog, 1)

          // 转换数组为字符串
          config.common.skipRecord = skipRecordRef.value.join('|')
          config.dialog.ctrlList = ctrlListRef.value.join('|')
          config.dialog.skipShow = skipShowRef.value.join('|')
          // 更新颜色信息
          updateColor(config.dialog, guiBgcolorRef.value)
          config.dialog.guiTextColor = guiTextColorRef.value.replace(/#/, '')
          // 转换keyList
          let keyList = KVListTo(keyMappingRef.value);
          dataSetting.value.appNameList = JSON.stringify(KVListTo(appNameListRef.value),null,2);
          // 需要将数据保存给服务器
          console.log('setPara')
          const saving = message.loading(contentText.value.intro75, { duration: 0 })
          let res = await ajax('setPara', { config, keyList, 'dataSetting': dataSetting.value })
          saving.destroy()
          if (res.code == 200) {
            message.success(contentText.value.intro76)
          } else {
            message.error(contentText.value.intro130)
            return
          }

          // 设置变量为新的数据
          store.preData.config = deepCopy(config)
          store.preData.keyList = deepCopy(keyList)
          // 需要应用最新键盘
          let mapDetailArr = keyboardValid()
          if (mapDetailArr.length > 0) {
            dataSetting.value.mapDetail = JSON.stringify(mapDetailArr)
          }
          store.preData.dataSetting = deepCopy(dataSetting.value)
        }
      })
    }
    // 计算差异
    const diffJsonList = computed(() => {
      const predata = <any>store.preData
      const data = <any>store.data
      let hash = {}
      if (allConfig.value == null || predata.config == null) {
        return hash
      }
      let keyList = KVListTo(keyMappingRef.value);
      dataSetting.value.appNameList = JSON.stringify(KVListTo(appNameListRef.value),null, 2);
      // 对比 config.common ，config.dialog ,store.preData.keyList
      getDiffHash(hash, data.config.common, predata.config.common, contentText.value.menu?.setting1, contentText.value)
      getDiffHash(hash, data.config.dialog, predata.config.dialog, contentText.value.menu?.setting2, contentText.value)
      getDiffHash(hash, dataSetting.value, predata.dataSetting, contentText.value.menu?.setting3, contentText.value)
      //getDiffHash(hash,config.common,predata.config.common,contentText.value.menu?.setting3)
      getDiffHash(hash, JSON.stringify(keyList, null, 2), JSON.stringify(predata.keyList, null, 2), contentText.value.menu?.setting4, contentText.value)
      return hash
    })
    // 根据选项显示文本
    function handleUpdateValue(value: string) {
      const data = <any>store.data
      let isNew = true
      for (let v of data.keymaps) {
        if (v.mapName == value) {
          mapDetailRef.value = v.mapDetail
          handleUpdateDetail()
          isNew = false;
          break
        }
      }
      if (isNew) {
        mapDetailRef.value = ''
        keyboardSaveInfo.value = contentText.value.intro105
      } else {
        keyboardSaveInfo.value = contentText.value.intro108
      }
    }
    // 进行有效性检查
    function keyboardValid() {
      let mapDetailArr = []
      try {
        mapDetailArr = JSON.parse(mapDetailRef.value)
        for (let v of mapDetailArr) {
          if (!(v instanceof Array) || v.length < 2 || v.length > 4) {
            keyboardDetail.value = contentText.value.intro102 + v
            return []
          }
          if (isNaN(v[0]) || isNaN(v[1])) {
            keyboardDetail.value = contentText.value.intro103 + v
            return []
          }
        }
      } catch (e) {
        keyboardDetail.value = contentText.value.intro101
        return []
      }
      keyboardDetail.value = '';
      //需要对  mapDetailRef.value 的内容格式化，便于编辑
      // 对于键盘数组进行格式化，方便查看
      mapDetailRef.value = `[\n${mapDetailArr.map(x => JSON.stringify(x)).join(',\n')}\n]`
      return mapDetailArr
    }
    // 输入内容变化
    function handleUpdateDetail() {
      let mapDetailArr = keyboardValid()
      if (mapDetailArr.length == 0) return;
      option.series[0].data = mapDetailArr.map(function (item) {
        let val: string | number = 0, key: string, keyMap;
        keyMap = item[3]
        if (item[2] == null) {
          val = '-'
        } else {
          key = item[2].toString()  // 界面显示的
          val = 0
        }
        // 用于产生显示在界面的文字内容
        // 用于产生显示在界面的文字内容
        let strKeyMap = '' // 配置了匹配键的
        if (keyMap != null) strKeyMap = ' (' + keyMap + ')'
        hashTxtData[item[0] + ',' + item[1]] = store.data.keyList[key] ?? key
        hashOriData[item[0] + ',' + item[1]] = key + strKeyMap
        // 将val 数据全部放到数组中，同于统计 max值
        return [item[0], item[1], val];
      })
      option && myChart.setOption(option);
    }
    // 将当前键盘应用到统计中
    function keyboardApply() {
      let mapDetailArr = keyboardValid()
      if (mapDetailArr.length == 0) return;
      (<any>store.preData).dataSetting.mapDetail = JSON.stringify(mapDetailArr)
      message.success(contentText.value.intro107)
    }
    function keyboardOptDialog(mapDetail, flag, info) {
      let mapName = dataSetting.value.keymap
      myDialog.warning({
        title: info + ' [' + mapName + ']',
        positiveText: contentText.value.intro109,
        negativeText: contentText.value.intro110,
        maskClosable: false,
        onPositiveClick: async () => {
          await ajax('optKeymap', { mapName, mapDetail, flag })
          message.success(info + contentText.value.intro111)
          // 需要刷新数据
          let data = <any>store.data; // 修改 data.keymaps
          let index = data.keymaps.findIndex(x => x.mapName == mapName)
          switch (flag) {
            case 0: // 删除
              data.keymaps.splice(index, 1)
              break;
            case 1: // 新增
              data.keymaps.push({ mapName, mapDetail })
              break;
            case 2: // 修改
              data.keymaps.splice(index, 1, { mapName, mapDetail })
              break;
          }
          //console.log(data.keymaps)
          loadPara()
        }
      })
    }
    function keyboardSave() {
      let mapDetail = keyboardValid()
      if (mapDetail.length == 0) return;
      let flag = (keyboardSaveInfo.value == contentText.value.intro105) ? 1 : 2; // 0=删除 1=新增 2=更新
      keyboardOptDialog(JSON.stringify(mapDetail), flag, keyboardSaveInfo.value)
    }
    function keyboardDelete() {
      keyboardOptDialog('', 0, contentText.value.intro106)
    }
    function handleUpdateColor(value) {
      updateColor(allConfig.value.dialog, value)
    }
    return {
      keyboardApply,
      myBorder,
      scrollTo,
      allConfig,
      dataSetting,
      skipRecordRef,
      contentText,
      ctrlListRef,
      skipShowRef,
      allFontRef,
      screenNum,
      keyMappingRef,
      appNameListRef,
      screenInfo,
      //allData,
      savePara,
      diffJsonList,
      resetPara,
      keymapsRef,
      handleUpdateValue,
      mapDetailRef,
      handleUpdateDetail,
      keyboardDetail,
      keyboardSave,
      keyboardSaveInfo,
      keyboardDelete,
      guiBgcolorRef,
      guiTextColorRef,
      handleUpdateColor,
      IPlinks,
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

.n-space {
  margin: 3px
}

body {
  scroll-behavior: smooth;
  /* 添加平滑滚动效果 */
}
</style>