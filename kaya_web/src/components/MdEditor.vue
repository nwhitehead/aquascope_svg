<script setup lang="ts">

import { ref, watch, nextTick, computed, onMounted } from 'vue';
import { useDark } from '@vueuse/core';

import rehypeStringify from 'rehype-stringify';
import rehypeHighlight from 'rehype-highlight';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import {unified} from 'unified';

import { ErrorInformation, render, initialize, run } from '../../../kaya_ts/ts/kaya.ts';

import '../styles/github-dark.css';
import '../styles/github-markdown.css';

const MONACO_EDITOR_OPTIONS = {
    automaticLayout: true,
    formatOnType: true,
    formatOnPaste: true,
};

const INPUT_DEBOUCE_DELAY = 300;

const DEFAULT_CODE = `
Here is a Kaya diagram showing memory:

\`\`\`kaya
# L1
## Stack
x: 5
y: 7
z: ptr(x)
p: ptr(H0)
## Heap
H0: (42, ptr(z))
\`\`\`

And some python:

\`\`\`python
print(f"What is ${2 + 2}?")
for i in range(10):
    print(i * i)
\`\`\`

Yes

`;

const code = ref(DEFAULT_CODE);
const renderedCode = ref('');
const renderedTheme = ref('');
const renderedHtml = ref('');
const isDark = useDark();
const transparent = ref(false);

const theme = computed(() => {
    if (transparent.value) return isDark.value ? 'dark_transparent' : 'light_transparent';
    return isDark.value ? 'dark' : 'light';
});

const processor = unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkMath)
    .use(remarkRehype, { allowDangerousHtml: false })
    .use(rehypeHighlight)
    .use(rehypeStringify);

async function updateKaya() {
    run();
    // const elems = document.querySelectorAll("pre code.language-kaya");
    // for (const elem of elems) {
    //     const src = elem.innerHTML;
    //     elem.innerHTML = '';
    //     // Access dependency on props before we wait for anything so it's tracked properly
    //     const scale = 1.0;
    //     const theme = "dark";
    //     const showPartial = true;
    //     // If source is empty, render nothing
    //     if (src === '') {
    //         return;
    //     }
    //     const response = await render(src, { scale, theme, showPartial });
    //     if (response.error) {
    //         console.log(response.error);
    //         const err = document.createElement('pre');
    //         err.innerHTML = response.error.msg;
    //         err.classList.add("error");
    //         elem.appendChild(err);
    //     }
    //     const img = document.createElement('img');
    //     img.src = response.imgUri || '';
    //     elem.appendChild(img);
    // }
}

async function handleUpdate() {
    if (renderedCode.value !== code.value || renderedTheme.value !== theme.value) {
        renderedCode.value = code.value;
        renderedTheme.value = theme.value;
        const html = await processor.process(renderedCode.value);
        renderedHtml.value = String(html);
        nextTick(() => {
            updateKaya();
        });
    }
}

let timeoutId: number | undefined = undefined;

onMounted(() => {
    handleUpdate();
    watch(() => [code.value, theme.value], () => {
        // Debouce input changes to input
        if (timeoutId) clearTimeout(timeoutId);
        timeoutId = setTimeout(() => handleUpdate(), INPUT_DEBOUCE_DELAY);
    });
});

</script>

<template>
    <div class="flex">

    <el-splitter layout="horizontal">
      <el-splitter-panel>
        <div class="demo-panel">
            <vue-monaco-editor
                v-model:value="code"
                :theme="isDark ? 'vs-dark' : 'vs-light'"
                :options="MONACO_EDITOR_OPTIONS"
                height="50vh"
            />
        </div>
      </el-splitter-panel>
      <el-splitter-panel>
        <article class="markdown-body" v-html="renderedHtml">
        </article>
      </el-splitter-panel>
    </el-splitter>
</div>
</template>

<style scoped>
div.flex {
    height: 100%;
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
.el-spliiter-panel {
    overflow: clip;
}
.label {
    font-size: 14px;
    color: var(--el-text-color-secondary);
    line-height: 44px;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-bottom: 0;
}
.slider-demo-block {
    max-width: 600px;
    width: 100%;
    display: flex;
    align-items: center;
}
.slider-demo-block .el-slider {
    margin-top: 0;
    margin-left: 12px;
    margin-right: 12px;
}
.slider-demo-block .demonstration {
    font-size: 14px;
    color: var(--el-text-color-secondary);
    line-height: 44px;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-bottom: 0;
}
.slider-demo-block .demonstration + .el-slider {
    flex: 0 0 70%;
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
    /* background-color: var(--el-color-danger-light-5); */
}

</style>
