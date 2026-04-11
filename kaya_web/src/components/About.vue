<script setup lang="ts">

import { ref, watch, nextTick, computed, onMounted } from 'vue';
import { useDark } from '@vueuse/core';

import rehypeStringify from 'rehype-stringify';
import rehypeHighlight from 'rehype-highlight';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';

import manualMarkdownSrc from '../../../docs/manual.md?raw';

import { initialize, run } from '../../../kaya_ts/ts/kaya.ts';

import '../styles/github-dark.css';
import '../styles/github-markdown.css';

initialize({ startOnLoad: false });

const isDark = useDark();
const rendered = ref('');

const theme = computed(() => {
    return isDark.value ? 'dark' : 'light';
});

const processor = unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkMath)
    .use(remarkRehype, { allowDangerousHtml: false })
    .use(rehypeHighlight, { plainText: ['kaya'] })
    .use(rehypeStringify);

async function updateKaya() {
    run({
        scale: 0.85,
        querySelector: '.language-kaya',
        theme: theme.value,
    });
}

onMounted(async () => {
    const html = await processor.process(manualMarkdownSrc);
    rendered.value = String(html);
    nextTick(() => {
        updateKaya();
    });
    watch(() => [theme.value], () => {
        updateKaya();
    });
});

</script>

<template>
    <div class="flex">
        <article class="markdown-body" v-html="rendered">
        </article>
    </div>
</template>

<style scoped>
div.flex {
    height: 100%;
    overflow: auto;
}
div .row {
    display: flex;
    flex-direction: row;
}
div .gap {
    display: flex;
    flex-grow: 1;
}
div.demo-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
}
html.dark div.demo-panel {
    background-color: var(--el-bg-color);
}
.markdown-body {
    box-sizing: border-box;
    min-width: 200px;
    max-width: 980px;
    margin: 0 auto;
    padding: 45px;
}

@media (max-width: 767px) {
    .markdown-body {
        padding: 15px;
    }
}

.error {
    text-align: left;
    background-color: #f00;
}

</style>
