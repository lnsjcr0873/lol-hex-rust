<template>
  <div
    class="relative inline-flex items-center justify-center shrink-0 overflow-hidden bg-black/60 rounded-full border border-[#1b263b]"
    :style="{ width: `${size}px`, height: `${size}px` }"
    :title="`副系符文风格: ${perkstyleId}`"
  >
    <img
      v-if="perkstyleId && perkstyleId > 0"
      :src="getPerkStyleIconUrl(perkstyleId)"
      class="w-full h-full object-cover scale-110"
    />
  </div>
</template>

<script setup lang="ts">
import { getPerkStyleIconUrl } from '../utils/resources'

withDefaults(
  defineProps<{
    perkstyleId?: number
    size?: number
  }>(),
  {
    perkstyleId: 0,
    size: 20
  }
)
</script>
