<script setup lang="ts">

import { watch, ref, computed, useTemplateRef, onMounted, nextTick } from 'vue';
import LeaderLine from 'leader-line-new';

// use the kaya style from the rust lib directly
import '../../../kaya_lib/src/style.css';

const props = defineProps<{
    contents: string, // html contents
    arrows: any,
}>();
let lines = [];

watch(
    () => props.arrows,
    (newValue, oldValue) => {
        console.log('Arrows changed');
    }
);

</script>

<style scoped>
div {
    background-color: var(--ep-bg-color);
    color: var(--ep-text-color-primary);
}
</style>

<template>
    <div v-html="contents"></div>
</template>
