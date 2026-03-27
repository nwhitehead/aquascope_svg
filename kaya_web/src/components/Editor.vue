<script setup lang="ts">

import { ref, watch, shallowRef, useTemplateRef } from 'vue';
import { useDark } from '@vueuse/core';
import Kaya from './Kaya.vue';
import html2canvas from 'html2canvas-pro';
import { toPng, toJpeg, toBlob, toPixelData, toSvg } from 'html-to-image';

const MONACO_EDITOR_OPTIONS = {
    automaticLayout: true,
    formatOnType: true,
    formatOnPaste: true,
};

//const code = ref("# L0\n## Stack\nx: 5\ny: 7\nz: ptr(x)\np: ptr(H0)\n## Heap\nH0: 42\n");
const code = ref(`
# L0
## Stack
x: 5
y: 7
x: 5
y: 7
x: 5
y: 7
x: 5
y: 7
x: 5
y: 7
## HEAP
## MORE HEAP

`);

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

function handleRender() {
    kayaKey.value++;
}

function handlePNG() {
    console.log("Generating PNG");
    console.log(kayaElem.value);
    // html2canvas(kayaElem.value, {
    //     scale: 1.0,
    // }).then((canvas) => {
    //     outputElem.value?.replaceChildren();
    //     outputElem.value.appendChild(canvas);
    // });
    toPng(kayaElem.value)
        .then((dataUrl) => {
            const img = new Image();
            img.src = dataUrl;
            outputElem.value.replaceChildren();
            outputElem.value.appendChild(img);
        })
        .catch((err) => {
            console.error('oops', err);
        });
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
            <div class="row">
                <el-button @click="handleRender">Re-render</el-button>
                <el-button @click="handlePNG">Save PNG</el-button>
                <el-switch active-text="Auto Update" v-model="autoUpdate" />
                <el-button type="primary" @click="handleUpdate" :disabled="updateDisabled()">Update</el-button>
            </div>
            <div ref="output">
            </div>
        </div>
      </el-splitter-panel>
      <el-splitter-panel>
        <div ref="kaya" class="demo-panel">

            <svg viewBox="0 0 10 10" xmlns="http://www.w3.org/2000/svg" class="leader-line-sc" style="left: 100px; top: 50px; width: 180px; height: 180px;">
                <circle cx="5" cy="5" r="1" fill="white" />
            </svg>

            <svg class="leader-line-sc" viewBox="829.383 71.5818 16 211.406" style="left: 129.383px; top: 71.5818px; width: 16px; height: 211.406px;">
                <defs>
                    <path id="leader-line-1-line-path" class="leader-line-line-path" style="fill: none;" d="M 837.383 197 C 837.383 79.5818 837.383 274.988 837.383 171"></path>
                    <use id="leader-line-1-line-shape" href="#leader-line-1-line-path" style="stroke-width: 4px;"></use>
                    <rect width="100%" height="100%" id="leader-line-1-mask-bg-rect" class="leader-line-mask-bg-rect" style="fill: white;" x="829.383" y="71.5818"></rect>
                    <mask id="leader-line-1-line-mask" maskUnits="userSpaceOnUse" x="829.383px" y="71.5818px" width="100%" height="100%"><use href="#leader-line-1-mask-bg-rect"></use><use class="leader-line-line-mask-shape" href="#leader-line-1-line-path" style="display: none;"></use><use href="#leader-line-1-caps" style="display: inline;"></use></mask>
                </defs>
                <g>
                    <use href="#leader-line-1-line-shape" style="stroke: var(--arrow0); mask: url(&quot;#leader-line-1-line-mask&quot;);"></use>
                    <use href="#leader-line-1-line-shape" style="mask: url(&quot;#leader-line-1-line-outline-mask&quot;); display: none;"></use>
                    <use class="leader-line-plugs-face" href="#leader-line-1-line-shape" style="display: inline; marker-end: url(&quot;#leader-line-1-plug-marker-1&quot;);"></use>
                    <circle cx="837" cy="182" r="8" fill="white" style="stroke: var(--arrow0); mask: url(&quot;#leader-line-1-line-mask&quot;);" />
                </g>
            </svg>

            <Kaya :source="renderedCode" :show_partial="true" @error="handleError" :key="kayaKey"/>
        </div>
      </el-splitter-panel>
    </el-splitter>
</template>

<style scoped>
.leader-line-sc {
    position: absolute;
    z-index: 5;
}
div .row {
    display: flex;
    flex-direction: row;
    gap: 24px;
}
div.demo-panel {
    display: flex;
    flex-direction: column;
    width: fit-content;
    overflow: scroll;
    position: relative;
}
html.dark div.demo-panel {
    background-color: var(--el-bg-color);
}
.el-spliiter-panel {
    overflow: clip;
}
</style>
