<template>
  <n-layout-header bordered class="nav">
    <div class="nav-box">
      <n-text tag="div" class="ui-logo" :depth="1">
        <span>{{ contentText.title + strVersion }} {{ serverVer }}</span> <span :style="urlColor"><a target="_blank" :href="url" :title="urlInfo" >{{ majorVersion }}</a></span>
      </n-text>
      <n-space justify="end">
        <n-button v-if="lang === 'zh-CN'" @click="onLangChange('en-US')">
          English</n-button>
        <n-button v-else @click="onLangChange('zh-CN')">
          中文</n-button>
        <n-button @click="changeTheme()">
          {{ theme === 'dark' ? contentText.theme2 : contentText.theme1 }}
        </n-button>
      </n-space>
    </div>
  </n-layout-header>
</template>

<script lang="ts">
import { defineComponent, PropType, toRef, computed, onMounted, ref } from 'vue';
import { strVersion } from '@/version.ts';
import {
  NMenu, NLayoutHeader, NText, NButton, NSpace
} from 'naive-ui';
import { useAustinStore } from '../App.vue'
import content from '../content.js';
import { ajax } from '@/common';

export default defineComponent({
  name: 'PageHead',
  props: {
    lang: {
      type: String as PropType<'en-US' | 'zh-CN'>,
    },
    onLangChange: {
      type: Function,
    }
  },
  components: {
    NLayoutHeader,
    NText,
    NMenu,
    NButton,
  },
  setup: (props) => {
    const lang = toRef(props, 'lang');
    const contentText = computed(() => content[lang.value]);

    const store = useAustinStore();
    // console.log(store.myTheme)
    const theme = toRef(store.myTheme)  // 修复默认主题异常
    function changeTheme() {
      if (store.myTheme == 'dark') {
        store.myTheme = 'light'
        theme.value = 'light'
      } else {
        store.myTheme = 'dark'
        theme.value = 'dark'
      }
    };
    // 获取 gitee 或github上的信息
    function getGitLatestRelease() {
      // 循环访问
      let gitServer = ['https://api.github.com/repos/pattazl/showKeyBoard/releases/latest',
        'https://gitee.com/api/v5/repos/pattazl/showkeyboard/releases/latest']
      // 获取最新版本信息， lastVer , info
      let getLatestVerInfo = [
        (x) => {
          let latestVer = x.tag_name;
          let info = x.body;
          let url = 'https://github.com/pattazl/showkeyboard/releases/latest'
          return { latestVer, info, url }
        }, // 对于github 的获取方式
        x => {
          let latestVer = x.tag_name;
          let info = x.body;
          let url = 'https://gitee.com/pattazl/showkeyboard/releases/latest'
          return { latestVer, info, url }
        }, // 对于gitee 的获取方式
      ]
      let latestVerInfo = null
      gitServer.some(async (url, index) => {
        let rsp = await fetch(url, {
          method: "get"
        })
        let result = await rsp.json();
        let info = getLatestVerInfo[index](result)
        if (typeof info?.latestVer == 'string') {
          latestVerInfo = info
          gitServerInfo.value = latestVerInfo
          return true;
        }
      })
      console.log(latestVerInfo)
    }
    let gitServerInfo = {}
    let serverVer = ref('');
    let urlColor = ref('');
    let url = ref('');
    let urlInfo = ref('');
    let majorVersion = ref('');
    // ajax方式获取服务器端版本号
    onMounted(async () => {
      let res = await ajax('version')
      let ver = res?.ver
      if (ver != null) {
        serverVer.value = `(Server ${ver})`
      }
      let mVer = res?.majorVersion
      if (mVer != null) {
        // 尝试获取 git上的版本信息
        getGitLatestRelease();
        // 判断版本号
        majorVersion.value = mVer
      }
    })
    return {
      lang,
      theme,
      onLangChange: props.onLangChange,
      changeTheme,
      contentText,
      strVersion,
      serverVer,
      majorVersion,
      urlColor,
      url,
      urlInfo,
    };
  },
});
</script>

<style scoped>
.nav {
  height: calc(var(--header-height) - 1px);
  display: flex;
  justify-content: center;
}

.nav-box {
  display: flex;
  justify-content: space-between;
  margin: auto;
  width: var(--content-width);
  max-width: var(--content-max-width);
}

.ui-logo {
  display: flex;
  align-items: center;
  font-size: 18px;
}
</style>
