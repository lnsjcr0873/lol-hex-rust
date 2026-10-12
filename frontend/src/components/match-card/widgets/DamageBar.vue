<template>
  <svg
    :width="width"
    :height="height"
    class="overflow-hidden select-none block"
    :style="{ borderRadius: `${borderRadius}px` }"
  >
    <rect x="0" y="0" :width="width" :height="height" fill="#1e293b" />
    <rect
      v-for="(dmg, index) in ordered"
      :key="index"
      :x="dmg.x"
      y="0"
      :height="height"
      :width="dmg.width"
      :fill="dmg.color"
    />
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    physicalDamage?: number
    magicDamage?: number
    trueDamage?: number
    baselineDamage?: number
    width?: number
    height?: number
    borderRadius?: number
  }>(),
  {
    physicalDamage: 0,
    magicDamage: 0,
    trueDamage: 0,
    baselineDamage: 1,
    width: 52,
    height: 6,
    borderRadius: 3
  }
)

const ordered = computed(() => {
  const base = Math.max(props.baselineDamage || 1, 1)
  const list = [
    {
      type: 'physical',
      color: '#f97316', // 橙红
      x: 0,
      width: Math.min((props.physicalDamage / base) * props.width, props.width)
    },
    {
      type: 'magic',
      color: '#38bdf8', // 亮蓝
      x: 0,
      width: Math.min((props.magicDamage / base) * props.width, props.width)
    },
    {
      type: 'true',
      color: '#f8fafc', // 纯白/银白
      x: 0,
      width: Math.min((props.trueDamage / base) * props.width, props.width)
    }
  ].sort((a, b) => b.width - a.width)

  for (let i = 1; i < list.length; i++) {
    list[i].x = list[i - 1].x + list[i - 1].width
  }

  return list
})
</script>
