<script setup lang="ts">

import { watch, ref, onMounted, nextTick } from 'vue';
import LeaderLine from 'leader-line-new';

// use the kaya style from the rust lib directly
import '../../../kaya_lib/src/style.css';

const props = defineProps<{
    contents: [string, any], // html contents, arrows
}>();
let lines = [];

function renderArrows() {
    console.log("Rendering arrows");
}

onMounted(() => {
    renderArrows();
});

watch(
    () => props.contents,
    (newValue, oldValue) => {
        console.log('Contents changed');

    //     // Need to wait until next tick to update arrows because the html we return
    //     // here will trigger contents to be redrawn (DOM update)
    //     nextTick(() => {
    //         while (lines.length > 0) {
    //             const line = lines.pop();
    //             console.log('Removed a line');
    //             line.remove();
    //         }
    //         for (const arrow of arrows) {
    //             const opt = arrow_options(arrow, 0);
    //             let srcElem = document.getElementById(arrow.src);
    //             const dstElem = document.getElementById(arrow.dst);
    //             if (arrow.src === arrow.dst) {
    //                 srcElem = srcElem.getElementsByClassName('dummy')[0];
    //             }
    //             let objopt = {};
    //             for (const [key, val] of opt) {
    //                 objopt[key] = val;
    //             }
    //             console.log(opt);
    //             console.log(objopt);
    //             if (srcElem !== null && dstElem !== null) {
    //                 const line = new LeaderLine(srcElem, dstElem, objopt);
    //                 lines.push(line);
    //             }
    //         }
    //     });
    //     return html;
    // },


    }
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
