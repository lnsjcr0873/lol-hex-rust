<template>
  <div
    class="relative inline-flex items-center justify-center overflow-hidden shrink-0 select-none bg-neutral-900 border border-neutral-800"
    :class="[round ? 'rounded-full' : 'rounded-lg']"
    :style="{ width: `${size}px`, height: `${size}px` }"
  >
    <img
      v-if="championId && championId > 0"
      :src="iconSrc"
      @error="handleImgError"
      class="w-full h-full object-cover"
      :class="{ 'scale-110': stretched }"
      :alt="getChampionName(championId)"
    />
    <div v-else class="w-full h-full bg-neutral-800 flex items-center justify-center text-[10px] text-neutral-500">
      ?
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { getChampionIconUrl, getChampionIconFallbackUrl, getChampionName } from '../utils/resources'

const props = withDefaults(
  defineProps<{
    championId?: number
    size?: number
    round?: boolean
    stretched?: boolean
  }>(),
  {
    championId: 0,
    size: 40,
    round: false,
    stretched: true
  }
)

const triedFallback = ref(false)

const iconSrc = computed(() => {
  if (triedFallback.value) {
    return getChampionIconFallbackUrl(props.championId)
  }
  return getChampionIconUrl(props.championId)
})

watch(
  () => props.championId,
  () => {
    triedFallback.value = false
  }
)

function handleImgError() {
  if (!triedFallback.value) {
    triedFallback.value = true
  }
}
</script>
