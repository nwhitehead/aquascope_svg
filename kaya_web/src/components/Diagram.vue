<script setup lang="ts">

import { watch, ref, useTemplateRef, onMounted, onUnmounted, nextTick } from 'vue';
import LeaderLine from 'leader-line-new';

// use the kaya style from the rust lib directly
import '../../../kaya_lib/src/style.css';

type ArrowInfo = {
    src: string,
    dst: string,
    options: object,
};

const CANVAS_SIZE = 2048;
const CANVAS_SCALE = 0.5;

const props = defineProps<{
    contents: [string, ArrowInfo[]], // html contents, arrows
}>();

const diaElem = useTemplateRef('dia');
const canvasRef = useTemplateRef<HTMLCanvasElement>('canvas');

// keep track of drawn LeaderLine objects
let lines: any[] = [];
let linesSvg: any[] = [];

function clearArrows() {
    // Remove all existing lines
    while (linesSvg.length > 0) {
        const lineSvg = linesSvg.pop();
        // Move to body
        document.body.appendChild(lineSvg);
    }
    while (lines.length > 0) {
        const line = lines.pop();
        line.remove();
    }
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

    // clear canvas
    const ctx = canvasRef.value?.getContext('2d');
    if (!ctx) return;
    ctx?.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
    const svgs = document.querySelectorAll('svg.leader-line');
    if (!svgs.length) {
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
        ctx.drawImage(img, x, y);
        img.src = datauriv;
    }

}

function renderArrows() {
    // Make sure we are not double rendering
    // If arrows are already clear it's fine
    clearArrows();
    // set global option for LeaderLine (secret option not specified in type I guess)
    // this makes LeaderLine not reposition the lines on window resize (we re-parent them so not needed)
    LeaderLine.positionByWindowResize = false;
    if (diaElem.value === null) return;
    const arrows = props.contents[1];
    for (const arrow of arrows) {
        let srcElem = document.getElementById(arrow.src);
        const dstElem = document.getElementById(arrow.dst);
        if (arrow.src === arrow.dst) {
            srcElem = srcElem.getElementsByClassName('dummy')[0];
        }
        if (srcElem !== null && dstElem !== null) {
            const line = new LeaderLine(srcElem, dstElem, arrow.options);
            lines.push(line);
        }
    }
    // // Move copy of svg defs and styles to each line
    const defs = document.querySelector('#leader-line-defs defs');
    const svgStyle = document.querySelector('#leader-line-defs style');
    const box = diaElem.value.getBoundingClientRect();
    const elems = document.querySelectorAll('.leader-line');
    for (const elem of elems) {
        // Give each svg leader line it's own copy of the global defs
        elem.prepend(svgStyle?.cloneNode(true));
        elem.prepend(defs.cloneNode(true));
        linesSvg.push(elem);
        diaElem.value.appendChild(elem);
        const currLeft = parseFloat(elem.style.left);
        elem.style.left = `${currLeft - box.x}px`;
        const currTop = parseFloat(elem.style.top);
        elem.style.top = `${currTop - box.y}px`;
    }
    // // Remove the svg defs thing
    //document.querySelector('#leader-line-defs')?.remove();
}

onMounted(() => {
    renderArrows();
});

onUnmounted(() => {
    clearArrows();
});

watch(
    () => props.contents,
    () => {
        // Need to wait until next tick to draw new arrows because DOM is also updating
        // DOM is being redrawn, need to wait until finished to add arrows
        nextTick(() => renderArrows());
    },
    // needs to be deep to see changing inside array
    { deep: true },
);

function handleClick() {
    convertArrowsSvg();
}

</script>

<style>

svg {
    z-index: 5;
}

</style>

<style scoped>
div {
    background-color: var(--ep-bg-color);
    color: var(--ep-text-color-primary);
    width: fit-content;
    position: relative;
    overflow: clip;
}
img.svgimg {
    background-color: #f00;
    width: 100%;
    height: 100%;
}
</style>

<template>
    <el-button @click="handleClick">Render to canvas</el-button>
    <div ref="dia" v-html="contents[0]"></div>
    <canvas ref="canvas" :width="CANVAS_SIZE" :height="CANVAS_SIZE"></canvas>
</template>
