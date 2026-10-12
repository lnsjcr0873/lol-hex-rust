<template>
  <div class="flex items-center space-x-3 select-none">
    <!-- Solo / Duo Card -->
    <div class="flex items-center space-x-2.5 px-3 py-1.5 rounded-xl bg-slate-900/90 border border-slate-800 shadow-sm">
      <div class="w-8 h-8 rounded-lg bg-amber-500/10 border border-amber-500/30 flex items-center justify-center font-black text-xs text-amber-400">
        单
      </div>
      <div class="flex flex-col">
        <div class="flex items-center space-x-1.5">
          <span class="text-xs font-black text-amber-300">
            {{ profile?.tier_solo || 'DIAMOND' }} {{ profile?.division_solo || 'I' }}
          </span>
          <span class="text-[11px] font-mono text-slate-300">{{ profile?.lp_solo ?? 75 }} LP</span>
        </div>
        <div class="text-[10px] text-slate-400 font-mono flex items-center space-x-1.5 mt-0.5">
          <span class="text-[#0ac8b9] font-bold">胜率 {{ profile?.win_rate_solo ?? 65 }}%</span>
          <span>·</span>
          <span>{{ profile?.wins_solo ?? 52 }}胜 {{ profile?.losses_solo ?? 28 }}负</span>
        </div>
      </div>
    </div>

    <!-- Flex SR Card -->
    <div class="flex items-center space-x-2.5 px-3 py-1.5 rounded-xl bg-slate-900/90 border border-slate-800 shadow-sm">
      <div class="w-8 h-8 rounded-lg bg-sky-500/10 border border-sky-500/30 flex items-center justify-center font-black text-xs text-sky-400">
        组
      </div>
      <div class="flex flex-col">
        <div class="flex items-center space-x-1.5">
          <span class="text-xs font-black text-sky-300">
            {{ profile?.tier_flex || 'EMERALD' }} {{ profile?.division_flex || 'II' }}
          </span>
          <span class="text-[11px] font-mono text-slate-300">{{ profile?.lp_flex ?? 42 }} LP</span>
        </div>
        <div class="text-[10px] text-slate-400 font-mono flex items-center space-x-1.5 mt-0.5">
          <span class="text-[#0ac8b9] font-bold">胜率 {{ profile?.win_rate_flex ?? 58 }}%</span>
          <span>·</span>
          <span>{{ profile?.wins_flex ?? 35 }}胜 {{ profile?.losses_flex ?? 25 }}负</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  profile?: any
}>()
</script>
