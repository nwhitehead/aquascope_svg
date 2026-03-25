<script setup lang="ts">

import { ref, computed, onMounted } from 'vue';
import { computedAsync } from '@vueuse/core';
import init, { parse, parse_partial, render_program, arrow_options } from '../../pkg/kaya_web.js';
import Diagram from './Diagram.vue';

const props = defineProps<{
    source: string,
    show_partial?: boolean,
}>();
const emit = defineEmits<{
    error: [row: number, col: number, msg: string],
}>()

let ready = false;
let error = ref();

const contents = computedAsync<string>(async () => {
    // Access dependency on props before we wait for anything so it's tracked properly
    let src = props.source + '\n';
    let show_partial = props.show_partial;
    console.log('starting computedAsync');
    if (!ready) {
        // init wasm (only need to do this once)
        await init();
        ready = true;
    }
    // Try to parse, if it fails then try with partial parse
    let res = parse(src);
    let prg = null;
    if (res.Success !== undefined) {
        prg = res.Success;
        error.value = null;
    } else {
        console.log('got error');
        error.value = res.Error;
        const row = res.Error[1][0];
        const col = res.Error[1][1];
        //emit(evt: "error", )
        if (!show_partial) {
            return "";
        }
        let res2 = parse_partial(src);
        if (res2.Success === undefined) {
            // if we get here something went wrong in partial parse
            // just show nothing for rendered output
            return "";
        }
        // Use partial parse for rendering
        prg = res2.Success;
    }
    let res_render = render_program(prg, true);
    const html = res_render[0];
    const arrows = res_render[1];
    return html;
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
},
    "",
);

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
    <Diagram :contents="contents" :arrows="[]" />
</template>
