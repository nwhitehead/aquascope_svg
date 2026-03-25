<script setup lang="ts">

import { ref, computed, useTemplateRef, onMounted, nextTick } from 'vue';
import init, { parse, render, render_parts, render_program, get_css, arrow_options } from '../../pkg/kaya_web.js';
import LeaderLine from 'leader-line-new';

// use the kaya style from the rust lib directly
import '../../../kaya_lib/src/style.css';

const elem = useTemplateRef('elem');
const props = defineProps(['source']);
let ready = ref(false);
let lines = [];

onMounted(async () => {
    // init wasm
    await init();
    console.log('WASM ready');
    // update reactive value to trigger computed stuff
    ready.value = true;
});

const contents = computed(() => {
    if (!ready.value) return;
    const prg = parse(props.source + '\n');
    let [html, arrows] = render_program(prg, true);
    // Need to wait until next tick to update arrows because the html we return
    // here will trigger contents to be redrawn (DOM update)
    nextTick(() => {
        while (lines.length > 0) {
            const line = lines.pop();
            console.log('Removed a line');
            line.remove();
        }
        for (const arrow of arrows) {
            const opt = arrow_options(arrow, 0);
            let srcElem = document.getElementById(arrow.src);
            const dstElem = document.getElementById(arrow.dst);
            if (arrow.src === arrow.dst) {
                srcElem = srcElem.getElementsByClassName('dummy')[0];
            }
            let objopt = {};
            for (const [key, val] of opt) {
                objopt[key] = val;
            }
            console.log(opt);
            console.log(objopt);
            if (srcElem !== null && dstElem !== null) {
                const line = new LeaderLine(srcElem, dstElem, objopt);
                lines.push(line);
            }
        }
    });
    return html;
});
</script>

<style scoped>
div {
    background-color: var(--ep-bg-color);
    color: var(--ep-text-color-primary);
}
</style>

<template>
    <div v-html="contents"></div>
</template>
