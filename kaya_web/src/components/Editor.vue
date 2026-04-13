<script setup lang="ts">

import { ref, watch, computed, nextTick } from 'vue';
import { Download } from '@element-plus/icons-vue';
import { useDark } from '@vueuse/core';
import Kaya from './Kaya.vue';

const MONACO_EDITOR_OPTIONS = {
    automaticLayout: true,
    formatOnType: true,
    formatOnPaste: true,
};

const code = ref(`
# L0
## Stack
x: 5
y: 7
z: ptr(x).se.de.g5
p: ptr(H0).se.ds.g4
## Heap
H0: (42, ptr(z).c4.sn.dw)
`);

const renderedCode = ref("");
const isDark = useDark();
const autoUpdate = ref(false);
const kayaKey = ref(0);
const resolution = ref(100);
const transparent = ref(false);

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

// use the above approach for string input
function base64(string) {
    const bytes = new TextEncoder().encode(string);
    const binString = String.fromCodePoint(...bytes);
    return btoa(binString);
}

function saveAsFile(dataUri, filename) {
    var link = document.createElement('a');
    link.download = filename;
    link.href = dataUri;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
}

function handleKaya() {
    const dataURL = `data:text/plain;base64,${base64(code.value + '\n')}`
    saveAsFile(dataURL, "diagram.kaya");
}

function handlePNG() {
    handleUpdate();
    nextTick(() => {
        nextTick(() => {
            const dataURL = document.querySelector('img.output').src;
            saveAsFile(dataURL, "diagram.png");
        });
    });
}

const theme = computed(() => {
    if (transparent.value) return isDark.value ? 'dark_transparent' : 'light_transparent';
    return isDark.value ? 'dark' : 'light';
});

</script>

<template>
    <div class="flex">
        <el-menu router default-active="1" class="el-menu-vertical-demo">
            <el-menu-item index="/" @click="handleKaya">
                <el-icon><Download /></el-icon>
                <template #title>Kaya (text)</template>
            </el-menu-item>
            <el-menu-item index="/" @click="handlePNG">
                <el-icon><Download /></el-icon>
                <template #title>PNG</template>
            </el-menu-item>
        </el-menu>

    <el-splitter layout="horizontal">
      <el-splitter-panel>
        <div class="demo-panel">
            <vue-monaco-editor
                v-model:value="code"
                :theme="isDark ? 'vs-dark' : 'vs-light'"
                :options="MONACO_EDITOR_OPTIONS"
                height="50vh"
            />
            <div class="row">
                <el-switch active-text="Auto Update" v-model="autoUpdate" />
                <div class="gap"></div>
                <el-button type="primary" @click="handleUpdate" :disabled="updateDisabled()">Update</el-button>
            </div>
            <p class="label">PNG generation options</p>
            <div class="row">
                <div class="slider-demo-block">
                    <span class="demonstration">Resolution</span>
                    <el-slider v-model="resolution" :min="50" :max="600" :step="50" show-stops />
                </div>
            </div>
            <div class="row">
                <el-switch active-text="Transparent background" v-model="transparent" />
            </div>
        </div>
      </el-splitter-panel>
      <el-splitter-panel>
        <div class="demo-panel">
            <div class="kaya">
                <Kaya :source="renderedCode" :show_partial="true" :theme="theme" :scale="resolution / 100.0" @error="handleError" :key="kayaKey"/>
            </div>
        </div>
      </el-splitter-panel>
    </el-splitter>
</div>
</template>

<style scoped>
div.flex {
    height: 100%;
}
div .row {
    display: flex;
    flex-direction: row;
}
div .gap {
    display: flex;
    flex-grow: 1;
}
div.demo-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
    background-color: transparent !important;
}
.kaya {
    background-color: transparent !important;
    width: fit-content;
}
html.dark div.demo-panel {
    background-color: var(--el-bg-color);
}
.el-spliiter-panel {
    overflow: clip;
}
.label {
    font-size: 14px;
    color: var(--el-text-color-secondary);
    line-height: 44px;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-bottom: 0;
}
.slider-demo-block {
    max-width: 600px;
    width: 100%;
    display: flex;
    align-items: center;
}
.slider-demo-block .el-slider {
    margin-top: 0;
    margin-left: 12px;
    margin-right: 12px;
}
.slider-demo-block .demonstration {
    font-size: 14px;
    color: var(--el-text-color-secondary);
    line-height: 44px;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-bottom: 0;
}
.slider-demo-block .demonstration + .el-slider {
    flex: 0 0 70%;
}

</style>
