<template>
  <n-layout-header bordered class="nav">
    <div class="nav-box">
      <n-text tag="div" class="ui-logo" :depth="1">
        <img src="../assets/logo.svg" />
        <span>Issue Helper</span>
      </n-text>
      <n-button
        text
        class="nav-picker"
        v-if="lang === 'zh-CN'"
        @click="onLangChange('en-US')"
      >
        English</n-button
      >
      <n-button text class="nav-picker" v-else @click="onLangChange('zh-CN')">
        中文</n-button
      >
    </div>
  </n-layout-header>
</template>

<script lang="ts">
import { defineComponent, PropType, toRef } from 'vue';
import { NMenu, NLayoutHeader, NText, NButton } from 'naive-ui';

export default defineComponent({
  name: 'PageHead',
  props: {
    lang: {
      type: String as PropType<'en-US' | 'zh-CN'>,
    },
    onLangChange: {
      type: Function,
    },
  },
  components: {
    NLayoutHeader,
    NText,
    NMenu,
    NButton,
  },
  setup: (props) => {
    const lang = toRef(props, 'lang');
    return {
      lang,
      onLangChange: props.onLangChange,
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

.ui-logo > img {
  margin-right: 12px;
  height: 32px;
  width: 32px;
}
</style>
