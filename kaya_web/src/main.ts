

import '~/styles/index.scss';
import 'uno.css';
import 'element-plus/theme-chalk/src/message.scss';
import 'element-plus/theme-chalk/src/message-box.scss';
import 'element-plus/theme-chalk/src/overlay.scss';
import 'element-plus/dist/index.css';

import App from './App.vue';
import { createApp } from 'vue';
import { createRouter, createWebHistory } from 'vue-router';
import { install as VueMonacoEditorPlugin } from '@guolao/vue-monaco-editor';
import ElementPlus from 'element-plus'

import Editor from './components/Editor.vue';
import MdEditor from './components/MdEditor.vue';
import About from './components/About.vue';

const routes = [
    { path: '/', component: Editor },
    { path: '/about', component: About },
    { path: '/markdown', component: MdEditor },
];

const app = createApp(App);
app.use(createRouter({
    history: createWebHistory(),
    routes,
}));
app.use(ElementPlus);
app.use(VueMonacoEditorPlugin);
app.mount("#app");
