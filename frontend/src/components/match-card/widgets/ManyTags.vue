<template>
  <div class="flex items-center space-x-1.5 flex-wrap">
    <div
      v-for="(tag, index) in displayTags"
      :key="index"
      class="px-1.5 py-0.5 rounded text-[10px] font-semibold flex items-center shadow-xs cursor-default transition-transform hover:scale-105"
      :class="[tag.color, tag.textColor]"
      :title="tag.content"
    >
      {{ tag.label }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMatchCard } from '../context'
import { computePlayerTags, type PlayerTag } from '../utils/tags'

const props = defineProps<{
  customTags?: PlayerTag[]
}>()

const { participant, team, teams, basicInfo } = useMatchCard()

const displayTags = computed(() => {
  if (props.customTags && props.customTags.length > 0) {
    return props.customTags
  }
  return computePlayerTags(participant.value, team.value, teams.value, basicInfo.value)
})
</script>
