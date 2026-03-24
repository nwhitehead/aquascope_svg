<script setup lang="ts">

import { ref, shallowRef } from 'vue';

const MONACO_EDITOR_OPTIONS = {
    automaticLayout: true,
    formatOnType: true,
    formatOnPaste: true,
};

const code = ref('// some code...');
const editor = shallowRef();

function handleMount(instance) {
    editor.value = instance;
}

</script>

<template>
   <el-splitter layout="horizontal">
      <el-splitter-panel>
        <div class="demo-panel">
            <vue-monaco-editor
                v-model:value="code"
                theme="vs-dark"
                :options="MONACO_EDITOR_OPTIONS"
                height="50vh"
                @mount="handleMount"
            />
        </div>
      </el-splitter-panel>
      <el-splitter-panel>
        <div class="demo-panel">
          <p>Content goes here. 2</p>
        </div>
      </el-splitter-panel>
    </el-splitter>
</template>

<style>
div.demo-panel {
    display: flex;
    flex-direction: column;
}
</style>
