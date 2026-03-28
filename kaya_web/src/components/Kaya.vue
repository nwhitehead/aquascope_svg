<script setup lang="ts">

import { reactive, ref, watch, onMounted } from 'vue';
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
const error = ref(null);
const contents = reactive(["", []]);

async function render() {
    console.log('Starting render');
    // Access dependency on props before we wait for anything so it's tracked properly
    let src = props.source + '\n';
    let show_partial = props.show_partial;
    if (!ready) {
        // init wasm (only need to do this once)
        await init();
        ready = true;
    }
    // If source is empty, render nothing
    if (props.source === '') {
        contents[0] = '';
        contents[1].splice(0);
        return;
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
            contents[0] = "";
            contents[1] = [];
        }
        let res2 = parse_partial(src);
        if (res2.Success === undefined) {
            // if we get here something went wrong in partial parse
            // just show nothing for rendered output
            contents[0] = "";
            contents[1] = [];
        }
        // Use partial parse for rendering
        prg = res2.Success;
    }
    let res_render = render_program(prg, true);
    const html = res_render[0];
    const arrows = res_render[1];
    // Apply options to arrows
    // clear arrows output
    contents[1].splice(0);
    for (const arrow of arrows) {
        const opt = arrow_options(arrow, 0);
        let objopt = {};
        for (const [key, val] of opt) {
            objopt[key] = val;
        }
        contents[1].push({ src: arrow.src, dst: arrow.dst, options: objopt });
    }

    contents[0] = html;
}

watch(
    // Dependencies on rendering
    () => [props.source, props.show_partial],
    async () => render(),
);

// Make sure we do rendering code on load
onMounted(() => render());

function error_text() {
    if (error.value !== null && error.value !== undefined) {
        return error.value[0];
    }
}
</script>

<style scoped>
pre.error {
    text-align: left;
    background-color: var(--el-color-danger-light-5);
}
</style>

<template>
    <pre v-if="error" class="error">{{ error_text() }}</pre>
    <Diagram :contents="contents" />
</template>
