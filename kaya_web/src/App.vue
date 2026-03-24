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
    color: #ccc;
    background-color: #181818 !important;
}
div.inner-output {
    display: flex;
    background-color: var(--bg);
    padding: 20px;
    width: 100%;
    height: fit-content;
    padding-bottom: 40px;
}
div.demo-panel {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100px;
    width: 100px;
    background-color: #f00;
}
.main-container {
    height: calc(100vh - var(--el-menu-item-height) - 4px);
}
</style>

<template>
    <div class="main-container flex">
        <div w="full" py="4">
            <el-container>

                <el-header>
                    <el-text class="mx-1">Default text</el-text>
                </el-header>

                <el-main>
                    <el-splitter layout="vertical">
                        <el-splitter-panel>
                            <el-text class="mx-1">Panel 1</el-text>
                        </el-splitter-panel>
                        <el-splitter-panel>
                            <el-text class="mx-1">Panel 2</el-text>
                        </el-splitter-panel>
                    </el-splitter>
                </el-main>

                <h1>HI</h1>
            </el-container>
        </div>
    </div>
</template>
