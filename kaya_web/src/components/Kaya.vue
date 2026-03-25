<script setup lang="ts">

import { ref, watch, computed, onMounted } from 'vue';
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
let contents = ref(["", []]);

async function render() {
    console.log('Recomputing');
    // Access dependency on props before we wait for anything so it's tracked properly
    let src = props.source + '\n';
    let show_partial = props.show_partial;
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
        error.value = res.Error;
        const row = res.Error[1][0];
        const col = res.Error[1][1];
        //emit(evt: "error", )
        if (!show_partial) {
            // empty contents
            contents.value[0] = "";
            contents.value[1] = [];
        }
        let res2 = parse_partial(src);
        if (res2.Success === undefined) {
            // if we get here something went wrong in partial parse
            // just show nothing for rendered output
            contents.value[0] = "";
            contents.value[1] = [];
        }
        // Use partial parse for rendering
        prg = res2.Success;
    }
    let res_render = render_program(prg, true);
    const html = res_render[0];
    const arrows = res_render[1];
    contents.value[0] = html;
    contents.value[1] = arrows;
}

watch(
    // Dependencies on rendering
    () => [props.source, props.show_partial],
    async () => {
        await render();
    },
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
    background-color: var(--ep-color-danger-light-5);
}
</style>

<template>
    <p><pre class="error">{{ error_text() }}</pre></p>
    <Diagram :contents="contents" />
</template>
