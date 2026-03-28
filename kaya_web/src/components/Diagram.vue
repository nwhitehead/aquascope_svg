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

const diaElem = useTemplateRef('dia');

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
        // Make sure transform is correct for line positions
        elem.style.transform = `translate(-${box.x}px, -${box.y}px)`;
    }
    // // Remove the svg defs thing
    //document.querySelector('#leader-line-defs')?.remove();

    // Now replace first line with canvas rendering...
    const svgs = document.querySelectorAll('svg.leader-line');
    const svg = svgs[1];
    if (!svg) return;

    let serializer = new XMLSerializer();
    let svgtxt = serializer.serializeToString(svg);

    //const svgtxt = svg.outerHTML;
    console.log(svgtxt);
    (async () => {
        const canvas = document.querySelector('canvas');
        const ctx = canvas.getContext('2d');

        console.log(canvas?.getBoundingClientRect());
        const v = await Canvg.from(ctx, svgtxt);

        // Start SVG rendering with animations and mouse handling.
        //v.start();
        await v.render();
        console.log(canvas?.getBoundingClientRect());
        console.log("done render");
    })();
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

</script>

<style>
svg {
    z-index: 5;
}

/* .leader-line {
    position: absolute;
    overflow: visible !important;
    pointer-events: none !important;
    font-size: 16px;
}

.leader-line-line-path {
    fill: none;
}

.leader-line-mask-bg-rect {
    fill: #fff;
}

.leader-line-caps-mask-anchor, .leader-line-caps-mask-marker-shape {
    fill: #000;
}

.leader-line-caps-mask-anchor {
    stroke: #000;
}

.leader-line-caps-mask-line, .leader-line-plugs-face {
    stroke: transparent;
}

.leader-line-line-mask-shape {
    stroke: #fff;
}

.leader-line-line-outline-mask-shape {
    stroke: #000;
}

.leader-line-plug-mask-shape {
    fill: #fff;
    stroke: #000;
}

.leader-line-plug-outline-mask-shape {
    fill: #000;
    stroke: #fff;
}

.leader-line-areaAnchor {
    position: absolute;
    overflow: visible !important;
} */

</style>

<style scoped>
div {
    background-color: var(--ep-bg-color);
    color: var(--ep-text-color-primary);
    width: fit-content;
    position: relative;
    overflow: clip;
}
</style>

<template>
    <div ref="dia" v-html="contents[0]"></div>
    <canvas ref="canvas"></canvas>
</template>
