<script setup lang="ts">
import { ref, shallowRef } from 'vue';
import Kaya from './components/Kaya.vue';
import { VueMonacoEditor } from '@guolao/vue-monaco-editor';

const code = ref("# L0\n## Stack\nx: 5\ny: 7\nz: ptr(x)\np: ptr(H0)\n## Heap\nH0: 42\n");
const editor = shallowRef();
const handleMount = editorInstance => (editor.value = editorInstance);
const MONACO_EDITOR_OPTIONS = {
    automaticLayout: true,
    minimap: { enabled: false },
    formatOnType: true,
    formatOnPaste: true,
};

</script>

<style>
body {
    background-color: #181818 !important;
}
    div.container {
        display: flex;
        flex-direction: column;
        width: 100vw;
        height: 100vh;
        border-radius: 0;
        overflow: none;
    }
    div.inner {
        display: flex;
        padding: 20px;
        width: 90vw;
        height: 40vh;
    }
    div.inner-output {
        display: flex;
        background-color: var(--bg);
        padding: 20px;
        width: 100%;
        height: fit-content;
        padding-bottom: 40px;
    }
</style>

<template>
    <div class="container">
        <div class="inner">
            <VueMonacoEditor
                v-model:value="code"
                theme="vs-dark"
                :options="MONACO_EDITOR_OPTIONS"
                @mount="handleMount"
            />
        </div>
        <div class="inner-output">
            <Kaya :source="code" />
        </div>
    </div>
</template>
