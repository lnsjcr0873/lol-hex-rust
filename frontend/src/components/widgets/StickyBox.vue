<!-- StickyBox.vue -->
<template>
  <div ref="rootEl">
    <slot />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { type StickyBoxConfig, useStickyBox } from './useStickyBox'

const props = defineProps<StickyBoxConfig>()
const rootEl = ref<HTMLElement | null>(null)

useStickyBox(rootEl, props)
</script>
