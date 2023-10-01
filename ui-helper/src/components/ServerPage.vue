<template>
  <n-config-provider :theme-overrides="themeOverrides" >
    <n-layout embedded>
      <PageHead :lang="lang" @langChange="setLang" />
      <n-layout has-sider >
        <n-layout-sider bordered collapse-mode="width" :collapsed-width="40" :width="180" :collapsed="collapsed"
          @collapse="collapsed = true" @expand="collapsed = false" show-trigger="bar">
          <n-menu style="min-height: calc(100vh - var(--header-height));" :options="menuOptions"
          @update:value="handleMenuSelect"  :default-expand-all="true" :watch-props="['defaultExpandedKeys']"/>
        </n-layout-sider>
        <n-layout
        :native-scrollbar="false"
          :position="'absolute'"
          style="min-width:320px; transition: all 0.3s;"
          :style="{ left: collapsed ? '40px' : '180px' }"
          content-style="min-height: calc(100vh - var(--header-height)); display: flex; flex-direction: column;"
        >
          <n-layout-content
          class="layout-content"
          style="border-radius: 0 16px 16px;padding: 10px 20px;"
          v-if="showMain"
        >
            <router-view v-slot="{ Component }">
              <component :is="Component" :key="$route.path" :lang="lang"/>
            </router-view>
          </n-layout-content>
          <div v-if="!showMain"><img :src="myImage"></div>
        </n-layout>
      </n-layout>
    </n-layout>
  </n-config-provider>
</template>

<script lang="ts">
import { defineComponent, ref,reactive,computed ,watch, onMounted, toRef} from 'vue';
import { NLayout, NConfigProvider, GlobalThemeOverrides, zhCN,MenuInst } from 'naive-ui';
import PageHead from './PageHead.vue';
import { useRouter } from 'vue-router'
import { getQuery, updateQuery } from './utils';
import content from '../content.js';
import {setOpt} from '../leftmenu';
// import { useAustinStore } from '../App.vue'
import { useMessage } from 'naive-ui';
import { deepCopy,ajax,str2Type } from '@/common.ts'
import { useAustinStore } from '@/App.vue'
import myImage from '@/res/loading.webp';

export default defineComponent({
  name: 'ServerPage',
  components: {
    PageHead,
    NLayout,
    NConfigProvider,
  },
  setup: () => {
    const message = useMessage()

    const lang = ref<'en-US' | 'zh-CN'>('zh-CN'); // 默认中文
    const contentText = computed(() => content[lang.value]).value;
    const menuOptions = ref([])
    const collapsed = ref(false)
    let showMain = ref(false)
    const setLang = (value: 'en-US' | 'zh-CN') => {
      lang.value = value;
      //console.log('setLang....',value)
      updateQuery({ lang: value }); // 更新地址栏
      const ct = content[value];
      setTimeout(() => {
        updateMenu(ct.menu); // 需要延迟执行
      }, 1);
      // 需要改变 store的内容
      // store.lang = value;
    };

    async function loadParaCore(){
      const store= useAustinStore();
      let data = <any>store.setting;
      if (data.config == null) {
        const loading = message.loading(content[lang.value].intro73, { duration: 0 })
        // 每次路由跳转变量会重新初始化，需要保存起来
        try {
          data = await ajax('getPara')
          loading.destroy()
          message.success(content[lang.value].intro74)
          showMain.value = true
        } catch (error) {
          loading.destroy()
          message.error(content[lang.value].intro85)
        }
        // 对配置文件中 data.config 的数字字符串转换为数字
        str2Type(data.config.common, 0)
        str2Type(data.config.dialog, 0)

        store.setting = data  // 在 setting中保留一份数据,进行页面切换后无需重新载入，除非页面整个刷新
        store.preData = deepCopy(data); // 之前的数据
      }
    }
    // 初始化时动态设置菜单内容
    const param = getQuery();
    let _lang = lang.value;
    if (param?.lang) {
      _lang = param.lang
    }
    setLang(_lang);

    // back, forward
    window.addEventListener('popstate', () => lang.value = getQuery()?.lang || 'en-US')

    const themeOverrides: GlobalThemeOverrides = {
      common: {
        fontSize: '15px',
        fontSizeMedium: '15px',
        fontSizeLarge: '16px',
      },
      Card: {
        titleFontSizeMedium: '20px',
      },
      Form: {
        labelFontSizeTopLarge: '15px',
      },
    };
     // 修改菜单内容
    const updateMenu = (menu) => {
        menuOptions.value = setOpt(menu);
    };
    //const message = useMessage()
    const activeName = ref('/')
    const router = useRouter()
    const handleMenuSelect = (value: string) => {
			activeName.value = value
			router.push({
				path: value,
			})
		}
    loadParaCore();
     //watch(() => store.myTheme, (newValue, oldValue) => {
     //  console.log('属性发生变化', newValue, oldValue);
     //});
    return {
      collapsed,
      lang,
      themeOverrides,
      setLang,
      handleMenuSelect,
      menuOptions,
      contentText,
      showMain,
      myImage,
    };
  },
});
</script>

<style scoped>
.content-box {
  border:1px solid red;
  text-align: left;
  height: auto;
  margin: auto;
  width: var(--content-width);
  max-width: var(--content-max-width);
}

.content {
  margin-top: 24px;
}
</style>
