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
    // // Move svg defs to div element
    // const defs = document.querySelector('#leader-line-defs defs');
    // // if (defs) {
    // //     diaElem.value.appendChild(defs);
    // // }
    // // Move all lines to div element
    console.log('diaElem box = ', diaElem.value?.getBoundingClientRect());
    const box = diaElem.value.getBoundingClientRect();
    const elems = document.querySelectorAll('.leader-line');
    for (const elem of elems) {
        linesSvg.push(elem);
        diaElem.value.appendChild(elem);
        // Make sure transform is correct for line positions
        elem.style.transform = `translate(-${box.x}px, -${box.y}px)`;
    }
    // // get a line and try to make it work in canvas
    // const aline = document.querySelector('.leader-line');
    // if (defs && aline) {
    //     aline.appendChild(defs);
    // }

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
.leader-line {
    position: absolute;
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
</style>

<template>
    <div ref="dia" v-html="contents[0]"></div>
</template>
