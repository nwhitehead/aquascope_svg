<script setup lang="ts">

import { watch, ref, onMounted, onUnmounted, nextTick } from 'vue';
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

// keep track of drawn LeaderLine objects
let lines: any[] = [];

function clearArrows() {
    // Remove all existing lines
    while (lines.length > 0) {
        const line = lines.pop();
        line.remove();
    }
}

function renderArrows() {
    // Make sure we are not double rendering
    // If arrows are already clear it's fine
    clearArrows();
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

<style scoped>
div {
    background-color: var(--ep-bg-color);
    color: var(--ep-text-color-primary);
}
</style>

<template>
    <div v-html="contents[0]"></div>
</template>
