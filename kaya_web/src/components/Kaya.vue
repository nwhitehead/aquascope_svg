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

</script>

<style scoped>
div {
    background-color: var(--ep-bg-color);
    color: var(--ep-text-color-primary);
}
</style>

<template>
    <div>
        {{ source }}
    </div>
</template>
