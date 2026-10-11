<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-[#090d1a] relative select-none">
    <!-- Top Summoner Career Profile Banner -->
    <div class="p-5 border-b border-[#162238] bg-[#070b14]/80 backdrop-blur shrink-0">
      <div class="flex items-center justify-between">
        <!-- Summoner Info & Badges -->
        <div class="flex items-center space-x-4">
          <div class="relative">
            <img 
              :src="`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/profile-icons/${profile.profile_icon_id}.jpg`"
              class="w-16 h-16 rounded-2xl border-2 border-[#c8aa6e] object-cover shadow-lg"
            />
            <span class="absolute -bottom-1 -right-1 px-1.5 py-0.2 bg-[#090d1a] border border-[#c8aa6e] text-[10px] text-[#c8aa6e] font-bold rounded-full">
              Lv.{{ profile.summoner_level }}
            </span>
          </div>

          <div>
            <div class="flex items-center space-x-2">
              <h1 class="text-xl font-bold text-slate-100 tracking-wide">{{ profile.game_name }}</h1>
              <span class="text-xs text-slate-400">#{{ profile.tag_line }}</span>
              <span 
                class="px-1.5 py-0.2 rounded text-[10px] font-semibold"
                :class="profile.is_connected ? 'bg-emerald-950/80 border border-emerald-600/50 text-emerald-300' : 'bg-amber-950/80 border border-amber-600/50 text-amber-300'"
              >
                {{ profile.is_connected ? '实时在线' : '离线示例' }}
              </span>
            </div>

            <!-- Dual Tier Badges -->
            <div class="flex items-center space-x-3 mt-1.5">
              <div class="flex items-center space-x-1.5 px-2 py-0.5 rounded bg-[#0f172a] border border-[#1e293b]">
                <span class="text-[11px] text-slate-400">单双:</span>
                <span class="text-xs font-bold text-[#c8aa6e]">{{ profile.tier_solo }} {{ profile.division_solo }}</span>
                <span class="text-[11px] text-slate-300 font-medium">{{ profile.lp_solo }} LP</span>
                <span class="text-[10px] text-[#0ac8b9]">({{ profile.win_rate_solo }}%)</span>
              </div>

              <div class="flex items-center space-x-1.5 px-2 py-0.5 rounded bg-[#0f172a] border border-[#1e293b]">
                <span class="text-[11px] text-slate-400">灵活:</span>
                <span class="text-xs font-bold text-[#38bdf8]">{{ profile.tier_flex }} {{ profile.division_flex }}</span>
                <span class="text-[11px] text-slate-300 font-medium">{{ profile.lp_flex }} LP</span>
                <span class="text-[10px] text-[#38bdf8]">({{ profile.win_rate_flex }}%)</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 20-Game Summary & Top Champions -->
        <div class="flex items-center space-x-6">
          <div class="flex items-center space-x-3">
            <div class="text-right">
              <div class="text-xs text-slate-400">近 20 场胜率</div>
              <div class="text-xl font-black text-[#0ac8b9]">{{ profile.recent_summary.win_rate }}%</div>
              <div class="text-[10px] text-slate-400 font-mono">{{ profile.recent_summary.avg_kda }}</div>
            </div>

            <!-- Top Champions Pill -->
            <div class="flex items-center space-x-2 pl-4 border-l border-[#1a2942]">
              <div 
                v-for="ch in profile.recent_summary.top_champions" 
                :key="ch.champion_id"
                class="flex flex-col items-center"
              >
                <img :src="ch.champion_icon" class="w-8 h-8 rounded-lg border border-[#1e2e4a]" :title="`${ch.champion_name} (${ch.games}场 ${ch.win_rate}%)`" />
                <span class="text-[9px] text-[#0ac8b9] font-bold mt-0.5">{{ ch.win_rate }}%</span>
              </div>
            </div>
          </div>

          <!-- Radar Mini View -->
          <div class="w-20 h-20 flex items-center justify-center">
            <RadarChart :stats="profile.recent_radar" :size="75" />
          </div>
        </div>
      </div>
    </div>

    <!-- Mode Filter Bar -->
    <div class="h-10 px-5 border-b border-[#162238] flex items-center justify-between bg-[#070b14]/50 shrink-0">
      <div class="flex items-center space-x-2">
        <button
          v-for="tab in filterTabs"
          :key="tab.id"
          @click="activeFilter = tab.id"
          :class="activeFilter === tab.id ? 'bg-[#0f233a] text-[#0ac8b9] border-[#0ac8b9]' : 'text-slate-400 hover:text-slate-200 border-transparent'"
          class="px-2.5 py-1 rounded text-xs font-medium border transition"
        >
          {{ tab.name }}
        </button>
      </div>

      <span class="text-[11px] text-slate-400">点击左侧比赛在右侧查看 10 人深度复盘</span>
    </div>

    <!-- Dual-Pane View: Left Matches, Right Scoreboard -->
    <div class="flex-1 overflow-hidden flex">
      <!-- Left: Matches Scroll Flow -->
      <div class="w-96 border-r border-[#162238] overflow-y-auto p-3 space-y-2.5">
        <div
          v-for="m in filteredMatches"
          :key="m.game_id"
          @click="selectMatch(m.game_id)"
          class="p-3 rounded-xl border transition-all cursor-pointer relative group"
          :class="[
            m.is_win ? 'bg-[#091526]/80 hover:bg-[#0d1d36] border-[#1d3557]' : 'bg-[#1a0c10]/80 hover:bg-[#241116] border-[#441a22]',
            selectedGameId === m.game_id ? 'ring-1 ring-[#0ac8b9]' : ''
          ]"
        >
          <!-- Match Header -->
          <div class="flex items-center justify-between text-xs mb-2">
            <div class="flex items-center space-x-2">
              <span class="font-bold text-xs" :class="m.is_win ? 'text-blue-400' : 'text-red-400'">
                {{ m.is_win ? '胜利' : '失败' }}
              </span>
              <span class="text-slate-400 text-[11px]">{{ m.queue_name }}</span>
            </div>
            <div class="flex items-center space-x-1.5 text-[11px] text-slate-400">
              <span>{{ formatDuration(m.game_duration_seconds) }}</span>
              <span 
                v-if="m.is_mvp"
                class="px-1.5 py-0.2 rounded bg-amber-500/20 text-[#f59e0b] border border-amber-500/40 text-[9px] font-black"
              >
                MVP
              </span>
              <span 
                v-if="m.is_svp"
                class="px-1.5 py-0.2 rounded bg-purple-500/20 text-purple-300 border border-purple-500/40 text-[9px] font-black"
              >
                SVP
              </span>
            </div>
          </div>

          <!-- Center Content: Hero & Performance -->
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-2.5">
              <img :src="m.champion_icon" class="w-11 h-11 rounded-lg border border-[#1e2e4a] object-cover" />
              <div>
                <div class="text-xs font-bold text-slate-100">{{ m.champion_name }}</div>
                <div class="text-xs font-semibold text-slate-200 mt-0.5">
                  {{ m.kills }} / <span class="text-red-400">{{ m.deaths }}</span> / {{ m.assists }}
                </div>
                <div class="text-[10px] text-slate-400">KDA {{ m.kda_ratio.toFixed(1) }}</div>
              </div>
            </div>

            <div class="text-right">
              <div class="text-xs text-slate-200 font-medium">{{ (m.total_damage / 1000).toFixed(1) }}k 输出</div>
              <div class="text-[10px] text-[#0ac8b9]">占比 {{ m.damage_share_pct }}%</div>
              <div class="text-[10px] text-slate-400">{{ m.minions_killed }} 补刀</div>
            </div>
          </div>

          <!-- Items Row -->
          <div class="flex items-center space-x-1 mt-2.5 pt-2 border-t border-[#162238]/60">
            <div 
              v-for="(item, idx) in m.items" 
              :key="idx" 
              class="w-5 h-5 rounded bg-[#070b14] border border-[#162238] overflow-hidden flex items-center justify-center"
            >
              <img 
                v-if="item > 0" 
                :src="`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/item-icons/${item}.png`" 
                class="w-full h-full object-cover"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Right: In-depth 10-Player Scoreboard -->
      <div class="flex-1 overflow-y-auto p-4 flex flex-col space-y-4">
        <div v-if="matchDetail" class="space-y-4">
          <!-- Game Matchup Head-to-Head Banner -->
          <div class="p-3 bg-[#080e1b] rounded-xl border border-[#162238] flex items-center justify-between">
            <div class="flex items-center space-x-3">
              <span class="text-xs font-bold text-blue-400">蓝方</span>
              <span class="text-sm font-black text-slate-100">{{ matchDetail.blue_team_kills }} 击杀</span>
              <span class="text-xs text-slate-400">{{ matchDetail.blue_team_turrets }} 塔 · {{ matchDetail.blue_team_dragons }} 龙 · {{ matchDetail.blue_team_barons }} 男爵</span>
            </div>

            <div class="text-center font-bold text-xs text-slate-300">
              {{ matchDetail.queue_name }} ({{ formatDuration(matchDetail.game_duration) }})
            </div>

            <div class="flex items-center space-x-3">
              <span class="text-xs text-slate-400">{{ matchDetail.red_team_turrets }} 塔 · {{ matchDetail.red_team_dragons }} 龙 · {{ matchDetail.red_team_barons }} 男爵</span>
              <span class="text-sm font-black text-slate-100">{{ matchDetail.red_team_kills }} 击杀</span>
              <span class="text-xs font-bold text-red-400">红方</span>
            </div>
          </div>

          <!-- Blue Team 5 Players -->
          <div class="bg-[#0b1322] rounded-xl border border-[#1c304f] overflow-hidden">
            <div class="px-3 py-1.5 bg-[#0e1b30] flex items-center justify-between text-xs font-bold text-blue-300 border-b border-[#1c304f]">
              <span>蓝方 (我方队伍)</span>
              <span>伤害 / 承伤 / 补刀 / 金币 / 装备</span>
            </div>
            
            <div class="divide-y divide-[#14233a]">
              <div 
                v-for="p in blueParticipants" 
                :key="p.puuid"
                class="p-2.5 flex items-center justify-between hover:bg-[#101e36] transition"
              >
                <!-- Hero & Player Info -->
                <div class="flex items-center space-x-2.5 w-44">
                  <div class="relative">
                    <img :src="p.champion_icon" class="w-9 h-9 rounded-lg border border-[#1e2e4a]" />
                    <span class="absolute -bottom-1 -right-1 text-[8px] bg-black/80 px-1 rounded text-slate-300 font-bold">
                      {{ p.champion_level }}
                    </span>
                  </div>
                  <div class="truncate">
                    <div class="text-xs font-bold text-slate-100 truncate flex items-center space-x-1">
                      <span>{{ p.game_name }}</span>
                      <span v-if="p.is_mvp" class="text-amber-400 font-black text-[9px]">👑 MVP</span>
                    </div>
                    <div class="text-[10px] text-slate-400">{{ p.role }} · {{ p.tier }}</div>
                  </div>
                </div>

                <!-- KDA -->
                <div class="w-24 text-center">
                  <div class="text-xs font-bold text-slate-100">{{ p.kills }}/{{ p.deaths }}/{{ p.assists }}</div>
                  <div class="text-[10px] text-slate-400">KDA {{ p.kda_ratio.toFixed(1) }}</div>
                </div>

                <!-- Damage Bar -->
                <div class="w-32 flex flex-col justify-center">
                  <div class="flex items-center justify-between text-[10px] mb-0.5">
                    <span class="text-slate-300 font-mono">{{ (p.total_damage / 1000).toFixed(1) }}k</span>
                    <span class="text-amber-400" v-if="p.is_highest_damage">⭐ 顶伤</span>
                    <span class="text-[#0ac8b9]">{{ p.damage_share_pct }}%</span>
                  </div>
                  <div class="h-1.5 bg-[#070b14] rounded-full overflow-hidden">
                    <div 
                      class="h-full bg-gradient-to-r from-blue-500 to-[#0ac8b9] rounded-full" 
                      :style="{ width: `${p.damage_share_pct * 2}%` }"
                    ></div>
                  </div>
                </div>

                <!-- Gold & CS -->
                <div class="w-24 text-right text-[11px]">
                  <div class="text-amber-300 font-mono">{{ (p.gold_earned / 1000).toFixed(1) }}k 金</div>
                  <div class="text-slate-400 text-[10px]">{{ p.minions_killed }} 刀</div>
                </div>

                <!-- Items -->
                <div class="flex items-center space-x-1">
                  <div 
                    v-for="(item, idx) in p.items" 
                    :key="idx" 
                    class="w-5 h-5 rounded bg-[#070b14] border border-[#162238] overflow-hidden flex items-center justify-center"
                  >
                    <img 
                      v-if="item > 0" 
                      :src="`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/item-icons/${item}.png`" 
                      class="w-full h-full object-cover"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Red Team 5 Players -->
          <div class="bg-[#1c0e12] rounded-xl border border-[#4a1f28] overflow-hidden">
            <div class="px-3 py-1.5 bg-[#261218] flex items-center justify-between text-xs font-bold text-red-300 border-b border-[#4a1f28]">
              <span>红方 (敌方队伍)</span>
              <span>伤害 / 承伤 / 补刀 / 金币 / 装备</span>
            </div>
            
            <div class="divide-y divide-[#2a131b]">
              <div 
                v-for="p in redParticipants" 
                :key="p.puuid"
                class="p-2.5 flex items-center justify-between hover:bg-[#26131c] transition"
              >
                <!-- Hero & Player Info -->
                <div class="flex items-center space-x-2.5 w-44">
                  <div class="relative">
                    <img :src="p.champion_icon" class="w-9 h-9 rounded-lg border border-[#1e2e4a]" />
                    <span class="absolute -bottom-1 -right-1 text-[8px] bg-black/80 px-1 rounded text-slate-300 font-bold">
                      {{ p.champion_level }}
                    </span>
                  </div>
                  <div class="truncate">
                    <div class="text-xs font-bold text-slate-100 truncate flex items-center space-x-1">
                      <span>{{ p.game_name }}</span>
                      <span v-if="p.is_svp" class="text-purple-400 font-black text-[9px]">👑 SVP</span>
                    </div>
                    <div class="text-[10px] text-slate-400">{{ p.role }} · {{ p.tier }}</div>
                  </div>
                </div>

                <!-- KDA -->
                <div class="w-24 text-center">
                  <div class="text-xs font-bold text-slate-100">{{ p.kills }}/{{ p.deaths }}/{{ p.assists }}</div>
                  <div class="text-[10px] text-slate-400">KDA {{ p.kda_ratio.toFixed(1) }}</div>
                </div>

                <!-- Damage Bar -->
                <div class="w-32 flex flex-col justify-center">
                  <div class="flex items-center justify-between text-[10px] mb-0.5">
                    <span class="text-slate-300 font-mono">{{ (p.total_damage / 1000).toFixed(1) }}k</span>
                    <span class="text-amber-400" v-if="p.is_highest_damage">⭐ 顶伤</span>
                    <span class="text-rose-400">{{ p.damage_share_pct }}%</span>
                  </div>
                  <div class="h-1.5 bg-[#070b14] rounded-full overflow-hidden">
                    <div 
                      class="h-full bg-gradient-to-r from-red-600 to-rose-400 rounded-full" 
                      :style="{ width: `${p.damage_share_pct * 2}%` }"
                    ></div>
                  </div>
                </div>

                <!-- Gold & CS -->
                <div class="w-24 text-right text-[11px]">
                  <div class="text-amber-300 font-mono">{{ (p.gold_earned / 1000).toFixed(1) }}k 金</div>
                  <div class="text-slate-400 text-[10px]">{{ p.minions_killed }} 刀</div>
                </div>

                <!-- Items -->
                <div class="flex items-center space-x-1">
                  <div 
                    v-for="(item, idx) in p.items" 
                    :key="idx" 
                    class="w-5 h-5 rounded bg-[#070b14] border border-[#162238] overflow-hidden flex items-center justify-center"
                  >
                    <img 
                      v-if="item > 0" 
                      :src="`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/item-icons/${item}.png`" 
                      class="w-full h-full object-cover"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { SummonerProfile, MatchHistoryItem, MatchDetailFull } from '../api/types';
import { getSummonerProfile, getMatchHistory, getMatchDetail } from '../api/tauri';
import { mockSummonerProfile, mockMatchHistory, mockMatchDetail } from '../api/mockData';
import RadarChart from '../components/RadarChart.vue';

const profile = ref<SummonerProfile>(mockSummonerProfile);
const matches = ref<MatchHistoryItem[]>(mockMatchHistory);
const matchDetail = ref<MatchDetailFull | null>(mockMatchDetail);
const selectedGameId = ref<number>(mockMatchHistory[0].game_id);

const activeFilter = ref('all');
const filterTabs = [
  { id: 'all', name: '全部模式' },
  { id: 'ranked_solo', name: '排位单双' },
  { id: 'ranked_flex', name: '排位灵活' },
  { id: 'aram', name: '极地大乱斗' },
];

const filteredMatches = computed(() => {
  if (activeFilter.value === 'all') return matches.value;
  if (activeFilter.value === 'ranked_solo') return matches.value.filter(m => m.queue_id === 420);
  if (activeFilter.value === 'ranked_flex') return matches.value.filter(m => m.queue_id === 440);
  if (activeFilter.value === 'aram') return matches.value.filter(m => m.queue_id === 450);
  return matches.value;
});

const blueParticipants = computed(() => {
  return matchDetail.value?.participants.filter(p => p.team === 'blue') || [];
});

const redParticipants = computed(() => {
  return matchDetail.value?.participants.filter(p => p.team === 'red') || [];
});

function formatDuration(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m}:${s < 10 ? '0' : ''}${s}`;
}

async function selectMatch(gameId: number) {
  selectedGameId.value = gameId;
  try {
    matchDetail.value = await getMatchDetail(gameId);
  } catch (err) {
    console.error(err);
  }
}

onMounted(async () => {
  try {
    profile.value = await getSummonerProfile();
    matches.value = await getMatchHistory();
    if (matches.value.length > 0) {
      selectedGameId.value = matches.value[0].game_id;
      matchDetail.value = await getMatchDetail(matches.value[0].game_id);
    }
  } catch (err) {
    console.error(err);
  }
});
</script>
