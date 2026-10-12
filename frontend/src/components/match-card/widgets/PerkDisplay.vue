<template>
  <div
    class="relative inline-flex items-center justify-center shrink-0 overflow-hidden bg-black/60 rounded-full border border-[#1b263b]"
    :style="{ width: `${size}px`, height: `${size}px` }"
    :title="getPerkName(perkId)"
  >
    <img
      v-if="perkId && perkId > 0"
      :src="getPerkIconUrl(perkId)"
      class="w-full h-full object-cover scale-110"
      :alt="getPerkName(perkId)"
    />
  </div>
</template>

<script setup lang="ts">
import { getPerkIconUrl, getPerkName } from '../utils/resources'

withDefaults(
  defineProps<{
    perkId?: number
    size?: number
  }>(),
  {
    perkId: 0,
    size: 20
  }
)
</script>
