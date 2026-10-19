<template>
  <div class="relative overflow-hidden rounded-lg border border-slate-700 bg-slate-950" :style="{ width: `${size}px`, height: `${size}px` }">
    <!-- Map Background Image -->
    <img
      v-if="mapId === 11"
      src="@/assets/maps/11.png"
      class="absolute inset-0 w-full h-full object-cover"
      alt="召唤师峡谷"
    />
    <img
      v-else-if="mapId === 12"
      src="@/assets/maps/12.png"
      class="absolute inset-0 w-full h-full object-cover"
      alt="极地大乱斗"
    />
    <img
      v-else-if="mapId === 21"
      src="@/assets/maps/21.png"
      class="absolute inset-0 w-full h-full object-cover"
      alt="斗魂竞技场"
    />
    <div v-else class="absolute inset-0 bg-slate-900 flex items-center justify-center text-xs text-slate-500">
      地图背景
    </div>

    <!-- Mapped Kill / Death Points -->
    <div
      v-for="(point, idx) in mappedPoints"
      :key="idx"
      class="absolute w-3.5 h-3.5 -translate-x-1/2 -translate-y-1/2 rounded-full border border-white flex items-center justify-center shadow-md animate-pulse"
      :class="pointColorClass"
      :style="{ left: `${point.left}px`, top: `${point.top}px` }"
    >
      <span class="w-1.5 h-1.5 rounded-full bg-white"></span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { mapToImagePosition } from '@shared/utils/game-map'

const props = withDefaults(
  defineProps<{
    mapId?: number
    points?: Array<{ x: number; y: number }>
    size?: number
    color?: 'red' | 'blue' | 'yellow'
  }>(),
  {
    mapId: 11,
    points: () => [],
    size: 260,
    color: 'red'
  }
)

const pointColorClass = computed(() => {
  if (props.color === 'blue') return 'bg-blue-500'
  if (props.color === 'yellow') return 'bg-amber-400'
  return 'bg-red-500'
})

const mappedPoints = computed(() => {
  if (!Array.isArray(props.points)) return []
  return props.points
    .filter((pt) => pt && Number.isFinite(pt.x) && Number.isFinite(pt.y))
    .map((pt) =>
      mapToImagePosition(pt.x, pt.y, props.size || 260, props.size || 260, props.mapId)
    )
})
</script>
