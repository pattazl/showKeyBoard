<template>
  <n-card :title="contentText.introTitle">
    <v-node :render="contentText.introOne" />

    <n-alert
      :title="contentText.introWarningTitle"
      type="warning"
      style="margin-bottom: 16px"
    >
      <v-node :render="contentText.introWarningContent" />
    </n-alert>

    <n-button text type="primary" @click="introVisible = true">{{
      contentText.explainTitle
    }}</n-button>

    <v-node :render="contentText.introTwo" />

    <n-modal v-model:show="introVisible">
      <n-card
        class="modal-card"
        :title="contentText.explainTitle"
        closable
        @close="introVisible = false"
      >
        <v-node :render="contentText.explain" />
      </n-card>
    </n-modal>
  </n-card>
</template>

<script lang="ts">
import { ref, defineComponent, Ref, PropType, toRef, computed } from 'vue';
import { NCard, NButton, NModal, NAlert } from 'naive-ui';
import content from '../content.js';

export default defineComponent({
  name: 'Intro',
  components: {
    NCard,
    NButton,
    NModal,
    NAlert,
  },
  props: {
    lang: {
      type: String as PropType<'en-US' | 'zh-CN'>,
    },
  },
  setup: (props) => {
    const lang = toRef(props, 'lang');
    const contentText = computed(() => content[lang.value]);
    const introVisible: Ref<boolean> = ref(false);
    return { lang, contentText, introVisible };
  },
});
</script>

<style scoped></style>
