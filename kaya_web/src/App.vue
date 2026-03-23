<script setup lang="ts">
import Kaya from './components/Kaya.vue'

let src = "# L0\n## Stack\nx: 5\ny: 7\nz: ptr(x)\np: ptr(H0)\n## Heap\nH0: 42\n";

</script>

<template>
  <textarea id="kaya">
  </textarea>
  <Kaya :source="src" />
</template>
