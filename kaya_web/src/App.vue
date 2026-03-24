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
    div.container {
        width: 100vw;
        height: 50vh;
    }
</style>

<template>
    <div class="container">
        <VueMonacoEditor
            v-model:value="code"
            theme="vs-dark"
            :options="MONACO_EDITOR_OPTIONS"
            @mount="handleMount"
        />
        <Kaya :source="code" />
    </div>
</template>
