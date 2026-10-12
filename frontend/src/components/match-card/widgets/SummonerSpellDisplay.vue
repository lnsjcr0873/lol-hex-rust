<template>
  <div
    class="relative inline-flex items-center justify-center shrink-0 overflow-hidden bg-black/60 rounded-md border border-[#1b263b]"
    :style="{ width: `${size}px`, height: `${size}px` }"
    :title="getSpellName(spellId)"
  >
    <img
      v-if="spellId && spellId > 0"
      :src="getSpellIconUrl(spellId)"
      class="w-full h-full object-cover"
      :alt="getSpellName(spellId)"
    />
  </div>
</template>

<script setup lang="ts">
import { getSpellIconUrl, getSpellName } from '../utils/resources'

withDefaults(
  defineProps<{
    spellId?: number
    size?: number
  }>(),
  {
    spellId: 0,
    size: 20
  }
)
</script>
