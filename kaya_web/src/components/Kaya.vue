<script setup lang="ts">

import { ref, computed, useTemplateRef, onMounted, nextTick } from 'vue';
import init, { parse, render, render_parts, render_program, get_css, arrow_options } from '../../pkg/kaya_web.js';
import LeaderLine from 'leader-line-new';

const elem = useTemplateRef('elem');
const props = defineProps(['source']);
let ready = ref(false);
let lines = [];

onMounted(async () => {
    // init wasm
    await init();
    // update reactive value to trigger computed stuff
    ready.value = true;
});

</script>

<style>

</style>

<template>
    <div>
        {{ source }}
    </div>
</template>