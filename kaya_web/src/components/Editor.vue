<script setup lang="ts">

import { ref, watch, shallowRef, useTemplateRef } from 'vue';
import { useDark } from '@vueuse/core';
import Kaya from './Kaya.vue';
//import html2canvas from 'html2canvas-pro';
import { toPng, toJpeg, toBlob, toPixelData, toSvg } from 'html-to-image';
// import { CodeEditor } from 'monaco-editor-vue3';
//import vue-monaco-editor from 'vue-monaco-editor';

const MONACO_EDITOR_OPTIONS = {
    automaticLayout: true,
    formatOnType: true,
    formatOnPaste: true,
};

const code = ref("# L0\n## Stack\nx: 5\ny: 7\nz: ptr(x)\np: ptr(H0)\n## Heap\nH0: 42\n");
const renderedCode = ref("");
let editor = null;
const isDark = useDark();
const autoUpdate = ref(false);
const kayaKey = ref(0);
const kayaElem = useTemplateRef('kaya');
const outputElem = useTemplateRef('output');

function handleMount(instance) {
    editor = instance;
}

function handleError(evt) {
    console.log(`ERROR ${evt}`);
}

function handleUpdate() {
    renderedCode.value = code.value;
    kayaKey.value++;
}

function updateDisabled() {
    return false;
    // // or we could do:
    // return renderedCode.value === code.value;
}

watch(() => code.value, () => {
    if (autoUpdate.value) handleUpdate();
});

function handleScroll() {
    // Force re-render of DOM for Kaya subcomponent
    // This is needed to redraw arrows, which are done in absolute position
    console.log('scroll');
    kayaKey.value++;
    return false;
}

function handleResize() {
    console.log('resize');
    // Force re-render of DOM for Kaya subcomponent
    kayaKey.value++;
}

function handlePNG() {
    console.log("Generating PNG");
    console.log(kayaElem.value);
    toPng(document.body)
        .then((dataUrl) => {
            const img = new Image();
            img.src = dataUrl;
            outputElem.value.appendChild(img);
        })
        .catch((err) => {
            console.error('oops', err);
        });
}

</script>

<template>
   <el-splitter layout="horizontal" @resize="handleResize">
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
                <el-button @click="handlePNG">Save PNG</el-button>
                <el-switch active-text="Auto Update" v-model="autoUpdate" />
                <el-button type="primary" @click="handleUpdate" :disabled="updateDisabled()">Update</el-button>
            </div>
            <div ref="output">
            </div>
        </div>
      </el-splitter-panel>
      <el-splitter-panel @scroll="handleScroll">
        <div ref="kaya" class="demo-panel">
            <Kaya :source="renderedCode" :show_partial="true" @error="handleError" :key="kayaKey"/>
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
    width: fit-content;
}
html.dark div.demo-panel {
    background-color: var(--el-bg-color);
}
</style>
