<template>
  <n-config-provider :theme-overrides="themeOverrides" :locale="locale">
    <n-layout embedded>
      <PageHead :lang="lang" @langChange="setLang" />
      <div class="content-box">
        <div class="content"><Intro :lang="lang" /></div>
        <div class="content"><IssueForm :lang="lang" /></div>
      </div>
    </n-layout>
  </n-config-provider>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { NLayout, NConfigProvider, GlobalThemeOverrides, zhCN } from 'naive-ui';
import PageHead from './PageHead.vue';
import Intro from './Intro.vue';
import IssueForm from './IssueForm.vue';
import { getQuery, updateQuery } from './utils';

export default defineComponent({
  name: 'IssuePage',
  components: {
    PageHead,
    Intro,
    IssueForm,
    NLayout,
    NConfigProvider,
  },
  setup: () => {
    const lang = ref<'en-US' | 'zh-CN'>('en-US');
    const locale = ref<undefined | typeof zhCN>();

    const setLang = (value: 'en-US' | 'zh-CN') => {
      lang.value = value;
      locale.value = value === 'en-US' ? undefined : zhCN;
      updateQuery({ lang: value });
    };

    // init
    const param = getQuery();
    if (!param?.lang) {
      updateQuery({ lang: lang.value });
    } else {
      setLang(param.lang);
    }

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

    return {
      lang,
      locale,
      themeOverrides,
      setLang,
    };
  },
});
</script>

<style scoped>
.content-box {
  margin: auto;
  width: var(--content-width);
  max-width: var(--content-max-width);
}

.content {
  margin-top: 24px;
}
</style>
