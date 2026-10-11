<template>
  <div 
    v-if="player" 
    class="fixed inset-y-0 right-0 w-96 bg-[#0a0f1d] border-l border-[#1e2c47] shadow-2xl z-40 flex flex-col transition-transform duration-300 select-none"
  >
    <!-- Drawer Header -->
    <div class="p-4 border-b border-[#162238] flex items-center justify-between bg-[#070b14]">
      <div class="flex items-center space-x-3">
        <div class="relative">
          <img 
            :src="player.champion_icon || player.spell1_icon" 
            :alt="player.game_name" 
            class="w-12 h-12 rounded-full border-2 border-[#c8aa6e] object-cover"
          />
          <span class="absolute -bottom-1 -right-1 px-1.5 py-0.2 bg-[#090d1a] border border-[#c8aa6e] text-[9px] text-[#c8aa6e] font-bold rounded-full">
            Lv.{{ player.summoner_level }}
          </span>
        </div>
        <div>
          <div class="flex items-center space-x-1.5">
            <h3 class="font-bold text-slate-100 text-sm tracking-wide">{{ player.game_name }}</h3>
            <span class="text-xs text-slate-400">#{{ player.tag_line }}</span>
          </div>
          <div class="flex items-center space-x-2 mt-0.5">
            <span class="text-xs font-semibold text-[#c8aa6e]">{{ player.tier }} {{ player.division }}</span>
            <span class="text-[11px] text-slate-400">{{ player.lp }} LP</span>
          </div>
        </div>
      </div>

      <button 
        @click="$emit('close')"
        class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-[#162238] transition"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Drawer Content Scroll Area -->
    <div class="flex-1 overflow-y-auto p-4 space-y-5">
      <!-- Tactical Badges -->
      <div v-if="player.badges.length > 0" class="space-y-1.5">
        <h4 class="text-xs font-medium text-slate-400">战术特征标签</h4>
        <div class="flex flex-wrap gap-1.5">
          <span
            v-for="badge in player.badges"
            :key="badge.id"
            class="px-2 py-0.5 rounded text-[11px] flex items-center space-x-1 font-medium bg-[#0f1d33] border border-[#1e3a5f] text-[#38bdf8]"
          >
            <span>{{ badge.icon }}</span>
            <span>{{ badge.label }}</span>
            <span v-if="badge.description" class="text-slate-400 text-[10px]">· {{ badge.description }}</span>
          </span>
        </div>
      </div>

      <!-- Radar Dimension Analysis -->
      <div class="bg-[#0c1424] p-3 rounded-xl border border-[#162238]">
        <div class="flex items-center justify-between mb-2">
          <h4 class="text-xs font-medium text-slate-300">六维综合战力画像</h4>
          <span class="text-[10px] text-[#0ac8b9]">标准化分值</span>
        </div>
        <div class="flex justify-center">
          <RadarChart :stats="player.radar" :size="200" />
        </div>
      </div>

      <!-- Recent 20 Games Overview -->
      <div class="bg-[#0c1424] p-3 rounded-xl border border-[#162238] space-y-2">
        <h4 class="text-xs font-medium text-slate-300">近期 20 场竞技表现</h4>
        <div class="grid grid-cols-2 gap-2 text-xs">
          <div class="bg-[#080d18] p-2 rounded-lg border border-[#141e30]">
            <span class="text-slate-400 text-[11px]">近20场胜率</span>
            <div class="text-base font-bold text-[#0ac8b9] mt-0.5">{{ player.recent_win_rate }}%</div>
          </div>
          <div class="bg-[#080d18] p-2 rounded-lg border border-[#141e30]">
            <span class="text-slate-400 text-[11px]">场均 KDA</span>
            <div class="text-base font-bold text-slate-100 mt-0.5">{{ player.recent_kda }}</div>
          </div>
        </div>
        
        <!-- Current Champion Mastery in Recent Games -->
        <div class="bg-[#080d18] p-2.5 rounded-lg border border-[#141e30] flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <img :src="player.champion_icon" class="w-7 h-7 rounded border border-[#1e293b]" />
            <div>
              <div class="text-xs text-slate-200">{{ player.champion_name }}</div>
              <div class="text-[10px] text-slate-400">近 20 场出场 {{ player.champ_recent_games }} 场</div>
            </div>
          </div>
          <div class="text-right">
            <div class="text-xs font-bold" :class="player.champ_recent_win_rate >= 60 ? 'text-[#0ac8b9]' : 'text-slate-300'">
              {{ player.champ_recent_win_rate }}% 胜率
            </div>
          </div>
        </div>
      </div>

      <!-- Recent 5 Matches dots -->
      <div class="bg-[#0c1424] p-3 rounded-xl border border-[#162238] space-y-2">
        <h4 class="text-xs font-medium text-slate-300">近 5 场走势</h4>
        <div class="flex items-center space-x-2">
          <div 
            v-for="(res, idx) in player.recent_matches" 
            :key="idx"
            class="flex-1 py-1.5 rounded flex items-center justify-center font-bold text-xs"
            :class="res === 'win' ? 'bg-blue-900/60 text-blue-300 border border-blue-500/40' : 'bg-red-950/60 text-red-400 border border-red-600/40'"
          >
            {{ res === 'win' ? '胜' : '负' }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { X } from 'lucide-vue-next';
import { PlayerCard } from '../api/types';
import RadarChart from './RadarChart.vue';

defineProps<{
  player: PlayerCard | null;
}>();

defineEmits<{
  (e: 'close'): void;
}>();
</script>
