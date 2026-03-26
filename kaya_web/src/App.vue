<script setup lang="ts">

import BaseHeader from './components/layouts/BaseHeader.vue';
import BaseSide from './components/layouts/BaseSide.vue';

</script>

<template>
    <BaseHeader />
    <div class="main-container flex">
        <BaseSide />
        <div w="full">
            <RouterView />
        </div>
    </div>
</template>

<style>
#app {
    color: var(--el-text-color-primary);
    background-color: var(--el-bg-color);
}

.main-container {
    height: calc(100vh - var(--el-menu-item-height) - 4px);
}
</style>
