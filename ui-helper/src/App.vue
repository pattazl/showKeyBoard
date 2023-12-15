<template>
  <n-config-provider :theme="theme" :locale="lang" :date-locale="dateLang">
    <n-message-provider>
      <n-dialog-provider>
      <ServerPage />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script lang="ts">
import { computed, defineComponent ,ref } from 'vue';
import { zhCN, enUS,dateZhCN,dateEnUS } from 'naive-ui'
import { defineStore ,storeToRefs } from 'pinia'
import {
  NMessageProvider,
  NConfigProvider,
  useOsTheme,
  darkTheme,
  lightTheme,
} from 'naive-ui';
import ServerPage from './components/ServerPage.vue';

export const useAustinStore = defineStore('austin', () => {
  let str = useOsTheme().value;
  console.log('App.vue theme',str)
  let myTheme = ref(str);
  let data = <any>ref({})
  let preData = <any>ref({})
  let lang = ref('')
  //return { myTheme ,lang}
  return { myTheme,data,preData,lang}
})
export default defineComponent({
  name: 'App',
  components: {
    ServerPage,
    NMessageProvider,
    NConfigProvider,
  },
  setup() {
    const store= useAustinStore();
    //const { myTheme } = storeToRefs(store)
    return {
      theme: computed(() => (store.myTheme === 'dark' ? darkTheme : lightTheme)),
      lang: computed(() => (store.lang === 'zh-CN' ? zhCN : enUS)),
      dateLang: computed(() => (store.lang === 'zh-CN' ? dateZhCN : dateEnUS)),
    };
  },
});

//  强制修改滚动效果 
const __scrollIntoView = Element.prototype.scrollIntoView
Element.prototype.scrollIntoView = function () {
  __scrollIntoView.call(this, { behavior: "smooth" })
}
</script>

<style>
html,
body,
#app {
  margin: 0;
  padding: 0;
  height: 100%;
  min-width: 320px;
  --header-height: 50px;
  --content-width: 900px;
  --content-max-width: calc(100vw - 32px);
}

body {
  overflow: auto;
  -webkit-font-smoothing: antialiased;
}

a {
  text-decoration: none;
}

.n-card.modal-card {
  width: calc(var(--content-width) + 32px);
  max-width: var(--content-max-width);
  margin-top: 32px;
  margin-bottom: 32px;
}
</style>
