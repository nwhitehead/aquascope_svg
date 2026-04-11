<script setup lang="ts">

import BaseHeader from './components/layouts/BaseHeader.vue';

import './styles/github.css';
import './styles/github-dark.css';
import './styles/github-markdown.css';

</script>

<template>
    <BaseHeader />
    <div class="main-container flex">
        <div w="full">
            <RouterView />
        </div>
    </div>
</template>

<style>
#app {
    color: var(--el-text-color-primary);
    background-color: var(--el-bg-color);
    overflow: clip;
}

.main-container {
    height: calc(100vh - var(--el-menu-item-height) - 4px);
}
</style>
