<script lang="ts" setup>

import { Edit, Setting } from '@element-plus/icons-vue';
import { ElMenu, ElMenuItem, ElIcon } from 'element-plus';

</script>

<template>
    <el-menu router default-active="1" class="el-menu-vertical-demo">
        <el-menu-item index="/">
            <el-icon><Edit /></el-icon>
            <template #title>Edit</template>
        </el-menu-item>
        <!-- <el-menu-item index="/">
            <el-icon><Setting /></el-icon>
            <template #title>Settings</template>
        </el-menu-item> -->
    </el-menu>
</template>
