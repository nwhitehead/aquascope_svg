<script setup lang="ts">

import { ref, shallowRef } from 'vue';
import { useDark } from '@vueuse/core';
import Kaya from './Kaya.vue';

const MONACO_EDITOR_OPTIONS = {
    automaticLayout: true,
    formatOnType: true,
    formatOnPaste: true,
};

const code = ref("# L0\n## Stack\nx: 5\ny: 7\nz: ptr(x)\np: ptr(H0)\n## Heap\nH0: 42\n");
const editor = shallowRef();
const isDark = useDark();

function handleMount(instance) {
    editor.value = instance;
}

</script>

<template>
   <el-splitter layout="horizontal">
      <el-splitter-panel>
        <div class="demo-panel">
            <vue-monaco-editor
                v-model:value="code"
                :theme="isDark ? 'vs-dark' : 'vs-light'"
                :options="MONACO_EDITOR_OPTIONS"
                height="50vh"
                @mount="handleMount"
            />
        </div>
      </el-splitter-panel>
      <el-splitter-panel>
        <div class="demo-panel">
          <Kaya :source="code" />
        </div>
      </el-splitter-panel>
    </el-splitter>
</template>

<style>
div.demo-panel {
    display: flex;
    flex-direction: column;
    background-color: var(--el-bg-color);
}
html.dark div.demo-panel {
    background-color: var(--el-bg-color);
}
</style>
