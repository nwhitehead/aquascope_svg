<script lang="ts" setup>
import {
  Menu as IconMenu,
  Edit,
  Setting,
} from '@element-plus/icons-vue'

function handleOpen(key: string, keyPath: string[]) {
    console.log(key, keyPath)
}
function handleClose(key: string, keyPath: string[]) {
    console.log(key, keyPath)
}
</script>

<template>
  <el-menu
    router
    default-active="1"
    class="el-menu-vertical-demo"
    @open="handleOpen"
    @close="handleClose"
  >
    <el-menu-item index="/">
      <el-icon>
        <Edit />
      </el-icon>
      <template #title>
        Edit
      </template>
    </el-menu-item>
    <el-menu-item index="/settings" disabled>
      <el-icon>
        <Setting />
      </el-icon>
      <template #title>
        Settings
      </template>
    </el-menu-item>
  </el-menu>
</template>
