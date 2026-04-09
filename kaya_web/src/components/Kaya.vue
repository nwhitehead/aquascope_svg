<script setup lang="ts">

import { ref, watch, onMounted } from 'vue';
//import init, { parse, parse_partial, draw_program_png } from '../../pkg/kaya_web.js';
import { ErrorInformation, setOptions, render } from '../kaya.ts';

const props = defineProps<{
    source: string,
    scale?: number,
    theme?: string,
    show_partial?: boolean,
}>();
const emit = defineEmits<{
    error: [row: number, col: number, msg: string],
}>()

const error = ref<ErrorInformation | null>(null);
let imgURI = ref<string>('');

async function renderDiagram() {
    console.log('Starting render');
    // Access dependency on props before we wait for anything so it's tracked properly
    const scale = (props.scale === undefined) ? 1.0 : props.scale;
    const theme = (props.theme === undefined) ? "" : props.theme;
    const showPartial = (props.show_partial === true);
    setOptions({ scale, theme, showPartial });
    const src = props.source + '\n';
    // If source is empty, render nothing
    if (props.source === '') {
        imgURI.value = '';
        return;
    }
    const response = await render(src);
    error.value = response.error;
    imgURI.value = response.imgUri || "";
}

watch(
    // Dependencies on rendering
    () => [props.source, props.show_partial, props.scale, props.theme],
    async () => renderDiagram(),
);

// Make sure we do rendering code on load
onMounted(() => renderDiagram());

function error_text() {
    if (error.value !== null && error.value !== undefined) {
        return error.value.msg;
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
    <img :style="{ 'transform': `scale(${ 1.0 / (props.scale || 1.0) })` }" class="output" :src="imgURI" />
</template>
