<script setup lang="ts">

import { ref, computed, useTemplateRef, onMounted, nextTick } from 'vue';
import init, { parse, parse_partial, render_program, arrow_options } from '../../pkg/kaya_web.js';
import LeaderLine from 'leader-line-new';

// use the kaya style from the rust lib directly
import '../../../kaya_lib/src/style.css';

const elem = useTemplateRef('elem');
const props = defineProps<{
    source: string,
}>();
const emit = defineEmits([])
let ready = ref(false);
let lines = [];
let error = ref();

onMounted(async () => {
    // init wasm
    await init();
    // update reactive value to trigger computed stuff
    ready.value = true;
});

const contents = computed(() => {
    if (!ready.value) return;
    let src = props.source + '\n';
    // Try to parse, if it fails then try with partial parse
    let res = parse(src);
    let prg = null;
    if (res.Success !== undefined) {
        prg = res.Success;
        error.value = null;
    } else {
        error.value = res.Error;
        console.log(error);
        let res2 = parse_partial(src);
        console.log(res2);
        if (res2.Success === undefined) {
            // if we get here something went wrong in partial parse
            // just show nothing for rendered output
            return "";
        }
        // Use partial parse for rendering
        prg = res2.Success;
    }
    let [html, arrows] = render_program(prg, true);
    return "";
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

function error_text() {
    if (error.value !== null && error.value !== undefined) {
        console.log(error.value[0]);
        return error.value[0];
    }
}
</script>

<style scoped>
div {
    background-color: var(--ep-bg-color);
    color: var(--ep-text-color-primary);
}
pre.error {
    text-align: left;
    /* color: var(--ep-color-danger); */
    /* color: var(--ep-color-danger-dark-2); */
    background-color: var(--ep-color-danger-light-5);
}
</style>

<template>
    <p><pre class="error">{{ error_text() }}</pre></p>
    <div v-html="contents"></div>
</template>
