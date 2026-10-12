<template>
  <span class="inline-flex items-center justify-center text-current select-none" :style="{ width: `${size}px`, height: `${size}px` }">
    <svg v-if="normalizedPosition === 'TOP'" viewBox="0 0 24 24" class="w-full h-full fill-current">
      <path d="M19 3l-4 4H7v8l-4 4V3h16z" />
      <path d="M21 5l-4 4H9v8l-4 4V5h16z" opacity=".25" transform="rotate(-180 13 13)" />
      <path d="M10 10H14V14H10z" opacity=".25" />
    </svg>
    <svg v-else-if="normalizedPosition === 'JUNGLE' || normalizedPosition === 'JUG'" viewBox="0 0 24 24" class="w-full h-full fill-current">
      <path d="M12 2L9 8h6l-3-6zm-4 7l-2 5h12l-2-5H8zm-3 6l-3 7h18l-3-7H5z" />
    </svg>
    <svg v-else-if="normalizedPosition === 'MIDDLE' || normalizedPosition === 'MID'" viewBox="0 0 24 24" class="w-full h-full fill-current">
      <path d="M17.5 6.5l-11 11L5 16l11-11 1.5 1.5z" />
      <path d="M19 3l-4 4H7v8l-4 4V3h16z" opacity=".25" />
      <path d="M21 5l-4 4H9v8l-4 4V5h16z" opacity=".25" transform="rotate(-180 13 13)" />
    </svg>
    <svg v-else-if="normalizedPosition === 'BOTTOM' || normalizedPosition === 'BOT' || normalizedPosition === 'ADC'" viewBox="0 0 24 24" class="w-full h-full fill-current">
      <path d="M21 5l-4 4H9v8l-4 4V5h16z" transform="rotate(-180 13 13)" />
      <path d="M19 3l-4 4H7v8l-4 4V3h16z" opacity=".25" />
      <path d="M10 10H14V14H10z" opacity=".25" />
    </svg>
    <svg v-else-if="normalizedPosition === 'UTILITY' || normalizedPosition === 'SUPPORT' || normalizedPosition === 'SUP'" viewBox="0 0 24 24" class="w-full h-full fill-current">
      <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="2" />
      <path d="M12 7v10M7 12h10" stroke="currentColor" stroke-width="2" />
    </svg>
    <svg v-else viewBox="0 0 24 24" class="w-full h-full fill-current">
      <circle cx="6" cy="6" r="3" />
      <circle cx="18" cy="6" r="3" />
      <circle cx="12" cy="12" r="3" />
      <circle cx="6" cy="18" r="3" />
      <circle cx="18" cy="18" r="3" />
    </svg>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    position?: string | null
    size?: number
  }>(),
  {
    position: 'ALL',
    size: 16
  }
)

const normalizedPosition = computed(() => {
  if (!props.position) return 'ALL'
  return props.position.toUpperCase()
})
</script>
