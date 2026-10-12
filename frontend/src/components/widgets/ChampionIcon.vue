<template>
  <div
    class="relative overflow-hidden inline-block shrink-0 select-none"
    :class="[round ? 'rounded-full' : 'rounded']"
  >
    <img
      v-if="championId && championId > 0"
      :src="iconUrl"
      :alt="String(championId)"
      class="w-full h-full object-cover transition-transform"
      :class="{ 'scale-110': stretched }"
      @error="handleImgError"
    />
    <div
      v-else
      class="w-full h-full bg-slate-800 flex items-center justify-center text-slate-500 text-[10px]"
    >
      ?
    </div>
    <div
      v-if="ring"
      class="absolute inset-0 pointer-events-none"
      :class="[round ? 'rounded-full' : 'rounded']"
      :style="{
        borderStyle: 'solid',
        borderColor: ringColor || 'rgba(255, 255, 255, 0.3)',
        borderWidth: `${ringWidth || 1}px`
      }"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

const props = withDefaults(
  defineProps<{
    championId?: number | null
    round?: boolean
    stretched?: boolean
    ring?: boolean
    ringColor?: string
    ringWidth?: number
  }>(),
  {
    championId: -1,
    round: false,
    stretched: true,
    ring: false,
    ringWidth: 1
  }
)

const hasError = ref(false)

const iconUrl = computed(() => {
  if (hasError.value || !props.championId || props.championId <= 0) {
    return 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/-1.png'
  }
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/${props.championId}.png`
})

const handleImgError = (e: Event) => {
  hasError.value = true
  const target = e.target as HTMLImageElement
  if (target) {
    target.src = 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/-1.png'
  }
}
</script>
