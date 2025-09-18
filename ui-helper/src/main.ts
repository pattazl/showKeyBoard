import { createApp, defineComponent } from 'vue';
import App from './App.vue';
import router from './router/router'
import 'vfonts/Inter.css';
import { createPinia } from 'pinia'


const pinia = createPinia()

const app = createApp(App)
app.component('VNode', defineComponent({
  props: {
    render: {
      type: Function,
      required: true
    }
  },
  render() {
    return this.render();
  },
}),)
app.use(router)
app.use(pinia)
app.mount('#app');
