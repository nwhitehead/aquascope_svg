<script setup lang="ts">

import { ref, watch, onMounted } from 'vue';
import init, { parse, parse_partial, draw_program_png } from '../../pkg/kaya_web.js';

const props = defineProps<{
    source: string,
    scale?: number,
    theme?: string,
    show_partial?: boolean,
}>();
const emit = defineEmits<{
    error: [row: number, col: number, msg: string],
}>()

let ready = false;
const error = ref(null);
let imgURI = ref('');

/// Given Uint8Array with PNG data, construct a Data URI for IMG tags to use for it
function createDataURI(data) {
    return URL.createObjectURL(new Blob([data], { type: 'image/png' }));
}

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
        imgURI.value = '';
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
            imgURI.value = '';
        }
        let res2 = parse_partial(src);
        if (res2.Success === undefined) {
            // if we get here something went wrong in partial parse
            // just show nothing for rendered output
            imgURI.value = '';
        }
        // Use partial parse for rendering
        prg = res2.Success;
    }
    if (prg && prg.length) {
        console.log(prg, props.scale, props.theme);
        const scale = (props.scale === undefined) ? 1.0 : props.scale;
        const theme = (props.theme === undefined) ? "" : props.theme;
        let png_data = draw_program_png(prg, scale, theme);
        imgURI.value = createDataURI(png_data);
    }
}

watch(
    // Dependencies on rendering
    () => [props.source, props.show_partial, props.scale, props.theme],
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
img.output {
    transform-origin: top left;
}
</style>

<template>
    <pre v-if="error" class="error">{{ error_text() }}</pre>
    <img :style="{ 'transform': `scale(${ 1.0 / props.scale })` }" class="output" :src="imgURI" />
</template>
