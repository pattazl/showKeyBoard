<template>
  <n-config-provider :theme="theme">
    <n-message-provider>
      <IssuePage />
    </n-message-provider>
  </n-config-provider>
</template>

<script lang="ts">
import { computed, defineComponent } from 'vue';
import {
  NMessageProvider,
  NConfigProvider,
  useOsTheme,
  darkTheme,
} from 'naive-ui';
import IssuePage from './components/IssuePage.vue';

export default defineComponent({
  name: 'App',
  components: {
    IssuePage,
    NMessageProvider,
    NConfigProvider,
  },
  setup() {
    const theme = useOsTheme();

    return {
      theme: computed(() => (theme.value === 'dark' ? darkTheme : {})),
    };
  },
});
</script>

<style>
html,
body,
#app {
  margin: 0;
  padding: 0;
  height: 100%;
  min-width: 320px;
  --header-height: 64px;
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
