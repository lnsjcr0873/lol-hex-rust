<template>
  <div class="flex items-center space-x-1 p-1 bg-black/40 rounded-lg border border-slate-800">
    <button
      v-for="tab in tabs"
      :key="tab.value"
      @click="$emit('update:selectedTab', tab.value)"
      class="px-3 py-1.5 rounded-md text-xs font-semibold transition-all select-none"
      :class="[
        selectedTab === tab.value
          ? 'bg-slate-800 text-white shadow-sm ring-1 ring-slate-700 font-bold'
          : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/60'
      ]"
    >
      {{ tab.label }}
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  tabs: Array<{ label: string; value: string; show?: boolean }>
  selectedTab: string
  winResult?: string
}>()

defineEmits<{
  'update:selectedTab': [tab: string]
}>()
</script>
