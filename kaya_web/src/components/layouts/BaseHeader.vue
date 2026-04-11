<script lang="ts" setup>

import { repository } from '~/../package.json';
import { toggleDark } from '~/composables';
import { ElMenu, ElMenuItem } from 'element-plus';

</script>

<template>
  <el-menu class="el-menu-bh" mode="horizontal" :ellipsis="false" router>
    <el-menu-item index="/about">
      <div class="flex items-center justify-center gap-2">
        <span>Kaya</span>
      </div>
    </el-menu-item>
    <el-menu-item index="/">
      <div class="flex items-center justify-center gap-2">
        <span>Diagram</span>
      </div>
    </el-menu-item>
    <el-menu-item index="/markdown">
      <div class="flex items-center justify-center gap-2">
        <span>Markdown</span>
      </div>
    </el-menu-item>

    <el-menu-item h="full" @click="toggleDark()" index="">
      <button
        class="w-full cursor-pointer border-none bg-transparent"
        style="height: var(--el-menu-item-height)"
      >
        <i inline-flex i="dark:ep-moon ep-sunny" />
      </button>
    </el-menu-item>

    <el-menu-item h="full" :index="repository.url">
      <a class="size-full flex items-center justify-center" :href="repository.url" target="_blank">
        <div i-ri-github-fill />
      </a>
    </el-menu-item>
  </el-menu>
</template>

<style lang="scss">
.el-menu-bh {
    z-index: 1;
}

.el-menu-bh {
    &.el-menu--horizontal > .el-menu-item:nth-child(1) {
        margin-right: auto;
    }
}
</style>
