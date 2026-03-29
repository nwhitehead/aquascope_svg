<script setup lang="ts">

import { ref, watch, useTemplateRef, nextTick } from 'vue';
import { useDark } from '@vueuse/core';
import Kaya from './Kaya.vue';
import html2canvas from 'html2canvas-pro';

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
//"# L0\n## Stack\nx: 5\ny: 7\nz: ptr(x)\np: ptr(H0)\n## Heap\nH0: 42\n");
const renderedCode = ref("");
const isDark = useDark();
const autoUpdate = ref(false);
const kayaKey = ref(0);
const kayaElem = useTemplateRef('kaya');
const outputElem = useTemplateRef('output');

// Virtual offscreen canvas for arrow rendering (limits max size of diagram)
const CANVAS_SIZE = 2048;
// Not sure why this scale factor is needed for viewBox??? 
const CANVAS_SCALE = 0.5;

// Scale for display canvas (how much bigger than final size is it?)
const CANVAS_QUALITY_SCALE = 4.0;

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
    nextTick(async () => {
        if (!kayaElem.value) return;
        if (!outputElem.value) return;

        console.log("Generating PNG");
        await convertArrowsSvg();
        html2canvas(kayaElem.value, {
            scale: 4.0,
        }).then((canvas) => {
            outputElem.value?.replaceChildren();
            outputElem.value?.appendChild(canvas);
            const canvasRef = document.querySelector('.canvas-target');
            if (!canvasRef) return;
            const ctxc = canvasRef.getContext('2d');
            ctxc.clearRect(0, 0, ctxc.canvas.width, ctxc.canvas.height);
            kayaKey.value++;
        });
    });
}

async function waitForEvent(elem: Element, evt: any, setup: any) {
    return new Promise((resolve) => {
        elem.addEventListener(evt, () => {
            resolve(null);
        });
        setup();
    });
}

async function convertArrowsSvg() {
    console.log('Render to canvas start');
    const canvasRef = document.querySelector('.canvas-target');
    const diaElem = document.querySelector('.dia');

    if (!canvasRef) {
        console.error("canvasRef null");
        return;
    }
    if (!diaElem) {
        console.error("diaElem null");
        return;
    }

    // make offscreen canvas clear canvas
    const canvas = new OffscreenCanvas(CANVAS_SIZE, CANVAS_SIZE);
    const ctx = canvas.getContext('2d');
    if (!ctx) {
        console.error('ctx null');
        return;
    }
    ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
    const svgs = document.querySelectorAll('svg.leader-line');
    if (!svgs.length) {
        console.warn('No leader lines found');
        return;
    }
    for (const svg of svgs) {
        // Create copy of the svg using deep clone
        const svgCopy = svg.cloneNode(true);
        // Set viewbox to fixed size of big backing canvas
        svgCopy.viewBox.baseVal.width = CANVAS_SIZE * CANVAS_SCALE;
        svgCopy.viewBox.baseVal.height = CANVAS_SIZE * CANVAS_SCALE;

        // User proper serializer to convert node to dataURI with svg contents
        const serializer = new XMLSerializer();
        const svgtxt = serializer.serializeToString(svgCopy);
        const datauriv = 'data:image/svg+xml,' + encodeURIComponent(svgtxt);
        // Now make an image with that svg data
        const img = new Image();
        // Wait until it is loaded (svg rendering is async even for dataURI I think)
        await waitForEvent(img, "load", () => {
            img.src = datauriv;
        });
        // Find original location of svg from node
        let x = parseFloat(svgCopy.style.left);
        let y = parseFloat(svgCopy.style.top);
        // Draw SVG to canvas at that location
        ctx.drawImage(img, 0, 0, CANVAS_SIZE, CANVAS_SIZE,  x, y, CANVAS_SIZE, CANVAS_SIZE);
    }
    const w = diaElem.clientWidth;
    const h = diaElem.clientHeight;
    console.log('client w,h = ', w, h);
    document.querySelectorAll('.leader-line').forEach(el => el.remove());

    // Render offscreen canvas to actual canvas
    // first set actual gfx canvas size to final size to avoid stretching
    canvasRef.width = w * CANVAS_QUALITY_SCALE;
    canvasRef.height = h * CANVAS_QUALITY_SCALE;
    canvasRef.style.width = `${w}px`;
    canvasRef.style.height = `${h}px`;
    const ctxc = canvasRef.getContext('2d');
    if (!ctxc) return;
    // read from (0, 0) - (w, h) scaled by vscale (2?)
    // write to entire dst canvas (should already be right size)
    const vscale = 2.0;
    ctxc.drawImage(canvas, 0, 0, w * vscale, h * vscale, 0, 0, ctxc.canvas.width, ctxc.canvas.height);
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
            />
            <div class="row">
                <el-switch active-text="Auto Update" v-model="autoUpdate" />
                <div class="gap"></div>
                <el-button type="primary" @click="handleUpdate" :disabled="updateDisabled()">Update</el-button>
            </div>
            <div class="row">
                <el-button @click="handleRender">Re-render</el-button>
                <el-button @click="handlePNG">Save PNG</el-button>
            </div>
            <div ref="output">
            </div>
        </div>
      </el-splitter-panel>
      <el-splitter-panel>
        <div class="demo-panel">
            <div class="kaya" ref="kaya">
                <Kaya :source="renderedCode" :show_partial="true" @error="handleError" :key="kayaKey"/>
            </div>
        </div>
      </el-splitter-panel>
    </el-splitter>
</template>

<style scoped>
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
    overflow: auto;
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
</style>
