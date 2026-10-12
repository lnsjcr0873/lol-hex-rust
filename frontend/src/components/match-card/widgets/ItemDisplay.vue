<template>
  <div
    class="relative inline-flex items-center justify-center shrink-0 overflow-hidden bg-black/60 border border-[#1b263b] transition-transform hover:scale-105"
    :class="[isTrinket ? 'rounded-full border-cyan-800/60' : 'rounded-md']"
    :style="{ width: `${size}px`, height: `${size}px` }"
    :title="itemId && itemId > 0 ? `装备 ID: ${itemId}` : '空槽位'"
  >
    <img
      v-if="itemId && itemId > 0"
      :src="imgSrc"
      @error="handleError"
      class="w-full h-full object-cover"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { getItemIconUrl, getItemFallbackUrl } from '../utils/resources'

const props = withDefaults(
  defineProps<{
    itemId?: number
    size?: number
    isTrinket?: boolean
  }>(),
  {
    itemId: 0,
    size: 24,
    isTrinket: false
  }
)

const triedFallback = ref(false)

const imgSrc = computed(() => {
  if (triedFallback.value) {
    return getItemFallbackUrl(props.itemId)
  }
  return getItemIconUrl(props.itemId)
})

watch(
  () => props.itemId,
  () => {
    triedFallback.value = false
  }
)

function handleError() {
  if (!triedFallback.value) {
    triedFallback.value = true
  }
}
</script>
