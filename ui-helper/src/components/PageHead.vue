<template>
  <n-layout-header bordered class="nav">
    <div class="nav-box">
      <n-text tag="div" class="ui-logo" :depth="1">
        <span>{{ contentText.title }}</span>
      </n-text>
      <n-button text class="nav-picker" v-if="lang === 'zh-CN'" @click="onLangChange('en-US')">
        English</n-button>
      <n-button text class="nav-picker" v-else @click="onLangChange('zh-CN')">
        中文</n-button>
      <n-space justify="end">
        <n-button @click="changeTheme('dark')">
          {{ contentText.theme1 }}
        </n-button>
        <n-button @click="changeTheme('light')">
          {{ contentText.theme2 }}
        </n-button>
      </n-space>
    </div>
  </n-layout-header>
</template>

<script lang="ts">
import { defineComponent, PropType, toRef ,computed} from 'vue';
import {
  NMenu, NLayoutHeader, NText, NButton,NSpace
} from 'naive-ui';
import { useAustinStore } from '../App.vue'
import content from '../content.js';

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

    const store= useAustinStore();
    console.log(store.myTheme)
    function changeTheme(style){
      store.myTheme = style
      //store.changeTh()
      console.log(store.myTheme)
    };
    return {
      lang,
      onLangChange: props.onLangChange,
      changeTheme,
      contentText
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
  cursor: pointer;
  display: flex;
  align-items: center;
  font-size: 18px;
}

.ui-logo>img {
  margin-right: 12px;
  height: 32px;
  width: 32px;
}
</style>
