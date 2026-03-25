<script setup lang="ts">

import { watch, ref, onMounted, nextTick } from 'vue';
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

function renderArrows() {
    console.log("Rendering arrows");
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

watch(
    () => props.contents,
    (newValue, oldValue) => {
        console.log('Contents changed');
        console.log('Arrows', props.contents[1]);
        while (lines.length > 0) {
            const line = lines.pop();
            console.log('Removed a line');
            line.remove();
        }
        // Need to wait until next tick to update arrows because the html is also updating
        // DOM is being redrawn, need to wait until finished to add arrows
        nextTick(() => renderArrows());

    //     nextTick(() => {
    //             const opt = arrow_options(arrow, 0);
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
