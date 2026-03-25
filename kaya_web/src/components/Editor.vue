<script setup lang="ts">

import { ref, watch, shallowRef } from 'vue';
import { useDark } from '@vueuse/core';
import Kaya from './Kaya.vue';

const MONACO_EDITOR_OPTIONS = {
    automaticLayout: true,
    formatOnType: true,
    formatOnPaste: true,
};

const code = ref("# L0\n## Stack\nx: 5\ny: 7\nz: ptr(x)\np: ptr(H0)\n## Heap\nH0: 42\n");
const renderedCode = ref("");
const editor = shallowRef();
const isDark = useDark();
const autoUpdate = ref(false);

function handleMount(instance) {
    editor.value = instance;
}

function handleError(evt) {
    console.log(`ERROR ${evt}`);
}

function handleUpdate() {
    renderedCode.value = code.value;
}

function updateDisabled() {
    return renderedCode.value === code.value;
}

watch(() => code.value, () => {
    if (autoUpdate.value) handleUpdate();
});

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
            <div class="row">
                <el-switch active-text="Auto Update" v-model="autoUpdate" />
                <el-button type="primary" @click="handleUpdate" :disabled="updateDisabled()">Update</el-button>
            </div>
        </div>
      </el-splitter-panel>
      <el-splitter-panel>
        <div class="demo-panel">
          <Kaya :source="renderedCode" :show_partial="true" @error="handleError" />
        </div>
      </el-splitter-panel>
    </el-splitter>
</template>

<style scoped>
div .row {
    display: flex;
    flex-direction: row;
    gap: 24px;
}
div.demo-panel {
    display: flex;
    flex-direction: column;
}
html.dark div.demo-panel {
    background-color: var(--el-bg-color);
}
</style>
