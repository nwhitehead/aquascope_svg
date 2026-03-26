<script setup lang="ts">
</script>

<template>
  <el-config-provider namespace="ep">
    <BaseHeader />
    <div class="main-container flex">
      <BaseSide />
      <div w="full">
        <h1>HELLO</h1>
        <!-- <RouterView /> -->
      </div>
    </div>
  </el-config-provider>
</template>

<style>
#app {
    color: var(--ep-text-color-primary);
    background-color: var(--ep-bg-color);
}

.main-container {
    height: calc(100vh - var(--ep-menu-item-height) - 4px);
}
</style>
