<script setup lang="ts">

import { ref, computed, useTemplateRef, onMounted } from 'vue';
import init, { parse, render, render_parts, render_program, get_css, arrow_options } from '../../pkg/kaya_web.js';
import LeaderLine from 'leader-line';

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
    for (const line of lines) {
        line.remove();
    }
    console.log(arrows);
    for (const arrow of arrows) {
        const opt = arrow_options(arrow, 0);
        console.log(arrow);
        console.log(opt);
    }
    return html;
});
</script>

<template>
    Here is diagram:
    <div v-html="contents"></div>
</template>
