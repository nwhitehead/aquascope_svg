import App from './App.vue';

import 'uno.css';
import 'element-plus/theme-chalk/src/message.scss';
import 'element-plus/theme-chalk/src/message-box.scss';
import 'element-plus/theme-chalk/src/overlay.scss';
import 'element-plus/dist/index.css';

import { createApp } from 'vue';
import { install as VueMonacoEditorPlugin } from '@guolao/vue-monaco-editor';
import ElementPlus from 'element-plus'

const app = createApp(App);
app.use(ElementPlus);
app.mount("#app");
