<script setup lang="ts">

import { watch, ref, useTemplateRef, onMounted, onUnmounted, nextTick } from 'vue';
import LeaderLine from 'leader-line-new';
import { Canvg } from 'canvg';

// use the kaya style from the rust lib directly
import '../../../kaya_lib/src/style.css';

type ArrowInfo = {
    src: string,
    dst: string,
    options: object,
};

const props = defineProps<{
    contents: [string, ArrowInfo[]], // html contents, arrows
}>();

const test = ref(false);
const diaElem = useTemplateRef('dia');
const dataUri = ref("");
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
        const offset = 0;//box.x;
        elem.style.left = `${currLeft - box.x + offset}px`;
        const currTop = parseFloat(elem.style.top);
        elem.style.top = `${currTop - box.y}px`;
        elem.viewBox.baseVal.x += offset;
        elem.viewBox.baseVal.y -= 0;
    }
    // // Remove the svg defs thing
    //document.querySelector('#leader-line-defs')?.remove();

    // Now replace first line with canvas rendering...
    const ctx = canvasRef.value?.getContext('2d');
    ctx?.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
    const svgs = document.querySelectorAll('svg.leader-line');
    if (!svgs.length) {
        return;
    }
    for (const svg of [svgs[0], svgs[1], svgs[2]]) {
        const serializer = new XMLSerializer();
        const svgCopy = svg.cloneNode(true);
        if (test.value) {
            // // moving left and top moves the rendered image
            // svgCopy.style.left = "0px";

            // // removing width and height on style has no effect
            // svgCopy.style.width = "";
            // svgCopy.style.height = "";

            // // moving x, y translates final image, and sometimes clips it if drawn outside viewbox
            // svgCopy.viewBox.baseVal.x += 20.0;
            // svgCopy.viewBox.baseVal.y -= 20.0;

            // // increasing width and height makes image smaller
            // svgCopy.viewBox.baseVal.width += 200.0;
            // svgCopy.viewBox.baseVal.height += 200.0;

            // // hmm, what about fixed width and height?
            svgCopy.viewBox.baseVal.width = 1024;
            svgCopy.viewBox.baseVal.height = 1024;
        }
        // svgCopy.style.right = "";
        // const scale = 1.0;
        // svgCopy.viewBox.baseVal.x *= scale;
        // svgCopy.viewBox.baseVal.y *= scale;
        // svgCopy.viewBox.baseVal.width *= scale;
        // svgCopy.viewBox.baseVal.height *= scale;
        const svgtxt = serializer.serializeToString(svgCopy);

        const datauriv = 'data:image/svg+xml,' + encodeURIComponent(svgtxt);

        const img = new Image();
        img.addEventListener("load", () => {
            if (!ctx) return;
            let x = parseFloat(svgCopy.style.left);
            let y = parseFloat(svgCopy.style.top);
            let w = svgCopy.viewBox.baseVal.width * 1;
            let h = svgCopy.viewBox.baseVal.height * 1;
            console.log('leader line ', x, y, w, h);
            ctx.drawImage(img, x, y);
            ctx.strokeStyle="green";
            ctx.beginPath();
//            ctx.rect(x, y, w, h);
            ctx.stroke();
        });
        img.src = datauriv;
    }
}

onMounted(() => {
    renderArrows();
});

onUnmounted(() => {
    clearArrows();
});

watch(
    () => [props.contents, test],
    () => {
        // Need to wait until next tick to draw new arrows because DOM is also updating
        // DOM is being redrawn, need to wait until finished to add arrows
        nextTick(() => renderArrows());
    },
    // needs to be deep to see changing inside array
    { deep: true },
);

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
    <el-switch active-text="Test On" v-model="test" />

    <div ref="dia" v-html="contents[0]"></div>
    <canvas ref="canvas" width="2048" height="2048"></canvas>
</template>
