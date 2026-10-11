<template>
  <div class="relative flex items-center justify-center select-none" :style="{ width: `${size}px`, height: `${size}px` }">
    <svg :width="size" :height="size" class="overflow-visible">
      <defs>
        <radialGradient id="radarGlow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stop-color="#0ac8b9" stop-opacity="0.35" />
          <stop offset="100%" stop-color="#0ac8b9" stop-opacity="0.05" />
        </radialGradient>
        <linearGradient id="polygonStroke" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#0ac8b9" />
          <stop offset="100%" stop-color="#38bdf8" />
        </linearGradient>
      </defs>

      <!-- Background Hexagonal Rings -->
      <polygon
        v-for="ring in [0.25, 0.5, 0.75, 1]"
        :key="ring"
        :points="getRingPoints(ring)"
        fill="none"
        stroke="#1e2c47"
        stroke-dasharray="2,2"
        stroke-width="1"
      />

      <!-- Radial Axis Lines -->
      <line
        v-for="(axis, i) in axes"
        :key="i"
        :x1="center"
        :y1="center"
        :x2="getAxisPoint(i, 1).x"
        :y2="getAxisPoint(i, 1).y"
        stroke="#162238"
        stroke-width="1"
      />

      <!-- Data Polygon Fill -->
      <polygon
        :points="dataPolygonPoints"
        fill="url(#radarGlow)"
        stroke="url(#polygonStroke)"
        stroke-width="2"
        class="transition-all duration-300"
      />

      <!-- Data Points -->
      <circle
        v-for="(point, i) in dataPoints"
        :key="i"
        :cx="point.x"
        :cy="point.y"
        r="3"
        fill="#00efd6"
        stroke="#070b14"
        stroke-width="1.5"
      />

      <!-- Axis Labels -->
      <text
        v-for="(axis, i) in axes"
        :key="'label-' + i"
        :x="getLabelPoint(i).x"
        :y="getLabelPoint(i).y"
        text-anchor="middle"
        dominant-baseline="central"
        fill="#94a3b8"
        class="text-[10px] font-medium"
      >
        {{ axis.name }}
      </text>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { RadarStat } from '../api/types';

const props = withDefaults(
  defineProps<{
    stats: RadarStat;
    size?: number;
  }>(),
  {
    size: 180,
  }
);

const center = computed(() => props.size / 2);
const radius = computed(() => (props.size / 2) * 0.72);

const axes = [
  { name: '输出', key: 'output' },
  { name: 'KDA', key: 'kda' },
  { name: '参团', key: 'join_rate' },
  { name: '生存', key: 'survival' },
  { name: '经济', key: 'gold' },
  { name: '视野', key: 'vision' },
] as const;

function getAngle(index: number): number {
  return (Math.PI * 2 / 6) * index - Math.PI / 2;
}

function getAxisPoint(index: number, factor: number): { x: number; y: number } {
  const angle = getAngle(index);
  const r = radius.value * factor;
  return {
    x: center.value + r * Math.cos(angle),
    y: center.value + r * Math.sin(angle),
  };
}

function getRingPoints(factor: number): string {
  return axes
    .map((_, i) => {
      const pt = getAxisPoint(i, factor);
      return `${pt.x},${pt.y}`;
    })
    .join(' ');
}

function getLabelPoint(index: number): { x: number; y: number } {
  const angle = getAngle(index);
  const r = radius.value + 16;
  return {
    x: center.value + r * Math.cos(angle),
    y: center.value + r * Math.sin(angle),
  };
}

const dataPoints = computed(() => {
  return axes.map((axis, i) => {
    const rawVal = props.stats[axis.key] ?? 50;
    const factor = Math.max(0.15, Math.min(1.0, rawVal / 100));
    return getAxisPoint(i, factor);
  });
});

const dataPolygonPoints = computed(() => {
  return dataPoints.value.map(pt => `${pt.x},${pt.y}`).join(' ');
});
</script>
