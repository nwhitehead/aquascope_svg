<script setup lang="ts">

import { ref, computed, useTemplateRef, onMounted, nextTick } from 'vue';
import init, { parse, render, render_parts, render_program, get_css, arrow_options } from '../../pkg/kaya_web.js';
import LeaderLine from 'leader-line-new';

const elem = useTemplateRef('elem');
const props = defineProps(['source']);
let ready = ref(false);
let lines = [];

const loadTheme = () => {
    if (!ready.value) return;
    // Remove any existing dynamic stylesheet first (optional, for theme switching)
    const existingLink = document.querySelector('#dynamic-theme');
    if (existingLink) {
        existingLink.remove();
    }
    // Create a new style element
    const link = document.createElement('style');
    link.id = 'dynamic-theme';
    link.textContent = get_css();
    // Append the style element to the document head
    document.head.appendChild(link);
};

onMounted(async () => {
    // init wasm
    await init();
    // update reactive value to trigger computed stuff
    ready.value = true;
    loadTheme();
});

const diagram = computed(() => {
    if (!ready.value) return;
    return get_css();
});

const contents = computed(() => {
    if (!ready.value) return;
    const prg = parse(props.source);
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

<template>
    <div v-html="contents"></div>
</template>
