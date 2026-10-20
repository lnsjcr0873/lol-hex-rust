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
  const pDmg = Math.max(0, props.physicalDamage || 0)
  const mDmg = Math.max(0, props.magicDamage || 0)
  const tDmg = Math.max(0, props.trueDamage || 0)
  const total = pDmg + mDmg + tDmg
  const base = Math.max(props.baselineDamage || 1, total, 1)

  const totalBarWidth = (Math.min(total, base) / base) * props.width
  const factor = total > 0 ? totalBarWidth / total : 0

  const pWidth = pDmg * factor
  const mWidth = mDmg * factor
  const tWidth = tDmg * factor

  return [
    {
      type: 'physical',
      color: '#f97316', // 橙红
      x: 0,
      width: pWidth
    },
    {
      type: 'magic',
      color: '#38bdf8', // 亮蓝
      x: pWidth,
      width: mWidth
    },
    {
      type: 'true',
      color: '#f8fafc', // 纯白/银白
      x: pWidth + mWidth,
      width: tWidth
    }
  ]
})
</script>
