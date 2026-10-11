<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-[#090d1a] relative select-none">
    <!-- Top Action Bar -->
    <div class="h-14 px-6 border-b border-[#162238] flex items-center justify-between shrink-0 bg-[#070b14]/70 backdrop-blur">
      <div class="flex items-center space-x-3">
        <div class="flex items-center space-x-2">
          <span class="w-2.5 h-2.5 rounded-full bg-[#0ac8b9] animate-pulse"></span>
          <h2 class="text-base font-bold text-slate-100 tracking-wide">
            {{ matchData.queue_name || '实时对局侦查' }}
          </h2>
        </div>
        <span class="px-2 py-0.5 rounded text-xs bg-[#101b2e] border border-[#1e2e4a] text-[#0ac8b9] font-medium">
          {{ matchData.gameflow_phase === 'ChampSelect' ? '英雄选择中' : '游戏中' }}
        </span>
      </div>

      <!-- Quick Actions -->
      <div class="flex items-center space-x-3">
        <button
          @click="sendChat('concise')"
          :disabled="isBroadcasting"
          class="px-3 py-1.5 rounded-lg bg-[#0e273c] hover:bg-[#143452] border border-[#1b4366] text-[#38bdf8] text-xs font-medium flex items-center space-x-1.5 transition active:scale-95 disabled:opacity-50"
        >
          <MessageSquare class="w-3.5 h-3.5" />
          <span>{{ isBroadcasting ? '发送中...' : '发送战绩到聊天框' }}</span>
        </button>

        <button
          @click="refreshData"
          class="p-2 rounded-lg bg-[#0f172a] hover:bg-[#162238] border border-[#1e293b] text-slate-300 hover:text-white text-xs transition"
          title="强制重新感知 LCU 对局"
        >
          <RotateCw class="w-3.5 h-3.5" :class="{ 'animate-spin': isRefreshing }" />
        </button>
      </div>
    </div>

    <!-- 5v5 Dual Columns -->
    <div class="flex-1 overflow-y-auto p-4 grid grid-cols-2 gap-4">
      <!-- Blue Team Column -->
      <div class="flex flex-col space-y-2.5">
        <div class="flex items-center justify-between px-3 py-2 bg-[#0c182b] rounded-lg border border-[#1d3557]">
          <div class="flex items-center space-x-2">
            <span class="w-2 h-2 rounded-full bg-blue-400"></span>
            <span class="text-xs font-bold text-blue-200 uppercase tracking-wider">我方阵营 (蓝方)</span>
          </div>
          <span class="text-[11px] text-blue-300/70">5 位召唤师已就绪</span>
        </div>

        <!-- Blue Cards -->
        <div 
          v-for="player in matchData.blue_team" 
          :key="player.puuid"
          @click="selectPlayer(player)"
          class="p-3 rounded-xl bg-[#0b1220] border transition-all cursor-pointer hover:bg-[#101b30] hover:border-[#23385d] relative group"
          :class="[
            player.premade_party ? player.premade_party.color_class : 'border-[#152033]',
            selectedPlayer?.puuid === player.puuid ? 'ring-1 ring-[#0ac8b9]' : ''
          ]"
        >
          <div class="flex items-center justify-between">
            <!-- Left: Hero avatar & spells -->
            <div class="flex items-center space-x-3">
              <div class="relative">
                <img 
                  :src="player.champion_icon" 
                  class="w-12 h-12 rounded-lg border border-[#1e2e4a] object-cover transition-transform group-hover:scale-105"
                />
                <span 
                  v-if="player.assigned_position" 
                  class="absolute -bottom-1 -left-1 px-1 py-0.2 bg-[#070b14] border border-[#1e2e4a] text-[9px] font-bold text-slate-300 rounded"
                >
                  {{ player.assigned_position }}
                </span>
                <span 
                  v-if="player.is_locked"
                  class="absolute -top-1 -right-1 w-3 h-3 rounded-full bg-[#0ac8b9] border border-[#070b14] flex items-center justify-center text-[8px] text-black font-black"
                >
                  ✓
                </span>
              </div>

              <!-- Spells -->
              <div class="flex flex-col space-y-1">
                <img :src="player.spell1_icon" class="w-5 h-5 rounded border border-[#1e2e4a]" :title="player.spell1_name" />
                <img :src="player.spell2_icon" class="w-5 h-5 rounded border border-[#1e2e4a]" :title="player.spell2_name" />
              </div>

              <!-- Name & Rank -->
              <div class="flex flex-col">
                <div class="flex items-center space-x-1.5">
                  <span class="text-xs font-bold text-slate-100 group-hover:text-[#0ac8b9] transition-colors">{{ player.game_name }}</span>
                  <span class="text-[10px] text-slate-400">#{{ player.tag_line }}</span>
                  <span 
                    v-if="player.premade_party"
                    class="px-1.5 py-0.2 rounded text-[9px] font-semibold bg-rose-950/80 text-rose-300 border border-rose-600/40"
                  >
                    {{ player.premade_party.label }}
                  </span>
                </div>
                <div class="flex items-center space-x-2 mt-0.5">
                  <span class="text-[11px] font-semibold text-[#c8aa6e]">{{ player.tier }} {{ player.division }}</span>
                  <span class="text-[10px] text-slate-400">{{ player.lp }} LP</span>
                </div>
              </div>
            </div>

            <!-- Right: Performance & Recent form -->
            <div class="flex flex-col items-end space-y-1">
              <div class="flex items-center space-x-2">
                <span class="text-xs font-bold" :class="player.recent_win_rate >= 60 ? 'text-[#0ac8b9]' : 'text-slate-200'">
                  {{ player.recent_win_rate }}% 胜率
                </span>
                <span class="text-[11px] text-slate-400">{{ player.recent_kda }}</span>
              </div>

              <!-- Recent 5 Games dots -->
              <div class="flex items-center space-x-1">
                <span 
                  v-for="(m, i) in player.recent_matches" 
                  :key="i"
                  class="w-2 h-2 rounded-full"
                  :class="m === 'win' ? 'bg-[#0ac8b9]' : 'bg-red-500'"
                ></span>
              </div>
            </div>
          </div>

          <!-- Bottom Badges -->
          <div v-if="player.badges.length > 0" class="mt-2 pt-2 border-t border-[#131d2e] flex flex-wrap gap-1.5">
            <span
              v-for="b in player.badges"
              :key="b.id"
              class="px-1.5 py-0.5 rounded text-[10px] flex items-center space-x-1 font-medium bg-[#0d1b2e] border border-[#1b3454] text-[#38bdf8]"
            >
              <span>{{ b.icon }}</span>
              <span>{{ b.label }}</span>
            </span>
          </div>
        </div>
      </div>

      <!-- Red Team Column -->
      <div class="flex flex-col space-y-2.5">
        <div class="flex items-center justify-between px-3 py-2 bg-[#260e14] rounded-lg border border-[#4d1924]">
          <div class="flex items-center space-x-2">
            <span class="w-2 h-2 rounded-full bg-red-400"></span>
            <span class="text-xs font-bold text-red-200 uppercase tracking-wider">敌方阵营 (红方)</span>
          </div>
          <span class="text-[11px] text-red-300/70">敌方情报已穿透解析</span>
        </div>

        <!-- Red Cards -->
        <div 
          v-for="player in matchData.red_team" 
          :key="player.puuid"
          @click="selectPlayer(player)"
          class="p-3 rounded-xl bg-[#0b1220] border transition-all cursor-pointer hover:bg-[#101b30] hover:border-[#23385d] relative group"
          :class="[
            player.premade_party ? player.premade_party.color_class : 'border-[#152033]',
            selectedPlayer?.puuid === player.puuid ? 'ring-1 ring-[#0ac8b9]' : ''
          ]"
        >
          <div class="flex items-center justify-between">
            <!-- Left: Hero avatar & spells -->
            <div class="flex items-center space-x-3">
              <div class="relative">
                <img 
                  :src="player.champion_icon" 
                  class="w-12 h-12 rounded-lg border border-[#1e2e4a] object-cover transition-transform group-hover:scale-105"
                />
                <span 
                  v-if="player.assigned_position" 
                  class="absolute -bottom-1 -left-1 px-1 py-0.2 bg-[#070b14] border border-[#1e2e4a] text-[9px] font-bold text-slate-300 rounded"
                >
                  {{ player.assigned_position }}
                </span>
                <span 
                  v-if="player.is_locked"
                  class="absolute -top-1 -right-1 w-3 h-3 rounded-full bg-red-500 border border-[#070b14] flex items-center justify-center text-[8px] text-white font-black"
                >
                  ✓
                </span>
              </div>

              <!-- Spells -->
              <div class="flex flex-col space-y-1">
                <img :src="player.spell1_icon" class="w-5 h-5 rounded border border-[#1e2e4a]" :title="player.spell1_name" />
                <img :src="player.spell2_icon" class="w-5 h-5 rounded border border-[#1e2e4a]" :title="player.spell2_name" />
              </div>

              <!-- Name & Rank -->
              <div class="flex flex-col">
                <div class="flex items-center space-x-1.5">
                  <span class="text-xs font-bold text-slate-100 group-hover:text-red-400 transition-colors">{{ player.game_name }}</span>
                  <span class="text-[10px] text-slate-400">#{{ player.tag_line }}</span>
                  <span 
                    v-if="player.premade_party"
                    class="px-1.5 py-0.2 rounded text-[9px] font-semibold bg-amber-950/80 text-amber-300 border border-amber-600/40"
                  >
                    {{ player.premade_party.label }}
                  </span>
                </div>
                <div class="flex items-center space-x-2 mt-0.5">
                  <span class="text-[11px] font-semibold text-[#c8aa6e]">{{ player.tier }} {{ player.division }}</span>
                  <span class="text-[10px] text-slate-400">{{ player.lp }} LP</span>
                </div>
              </div>
            </div>

            <!-- Right: Performance & Recent form -->
            <div class="flex flex-col items-end space-y-1">
              <div class="flex items-center space-x-2">
                <span class="text-xs font-bold" :class="player.recent_win_rate >= 60 ? 'text-[#0ac8b9]' : 'text-slate-200'">
                  {{ player.recent_win_rate }}% 胜率
                </span>
                <span class="text-[11px] text-slate-400">{{ player.recent_kda }}</span>
              </div>

              <!-- Recent 5 Games dots -->
              <div class="flex items-center space-x-1">
                <span 
                  v-for="(m, i) in player.recent_matches" 
                  :key="i"
                  class="w-2 h-2 rounded-full"
                  :class="m === 'win' ? 'bg-[#0ac8b9]' : 'bg-red-500'"
                ></span>
              </div>
            </div>
          </div>

          <!-- Bottom Badges -->
          <div v-if="player.badges.length > 0" class="mt-2 pt-2 border-t border-[#131d2e] flex flex-wrap gap-1.5">
            <span
              v-for="b in player.badges"
              :key="b.id"
              class="px-1.5 py-0.5 rounded text-[10px] flex items-center space-x-1 font-medium bg-[#1e131d] border border-[#3b1d30] text-[#f472b6]"
            >
              <span>{{ b.icon }}</span>
              <span>{{ b.label }}</span>
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Slide-over Tactical Drawer -->
    <PlayerDetailDrawer 
      :player="selectedPlayer" 
      @close="selectedPlayer = null" 
    />

    <!-- Toast Notification -->
    <div 
      v-if="toastMessage" 
      class="absolute bottom-6 left-1/2 -translate-x-1/2 px-4 py-2 bg-[#0e273c] border border-[#0ac8b9] text-[#0ac8b9] text-xs font-semibold rounded-lg shadow-xl flex items-center space-x-2 z-50 animate-bounce"
    >
      <Check class="w-4 h-4" />
      <span>{{ toastMessage }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { MessageSquare, RotateCw, Check } from 'lucide-vue-next';
import { PlayerCard, OngoingMatchData } from '../api/types';
import { getOngoingMatch, sendChatRecords } from '../api/tauri';
import { mockOngoingMatch } from '../api/mockData';
import PlayerDetailDrawer from '../components/PlayerDetailDrawer.vue';

const matchData = ref<OngoingMatchData>(mockOngoingMatch);
const selectedPlayer = ref<PlayerCard | null>(null);
const isRefreshing = ref(false);
const isBroadcasting = ref(false);
const toastMessage = ref('');

const selectPlayer = (player: PlayerCard) => {
  selectedPlayer.value = player;
};

const showToast = (msg: string) => {
  toastMessage.value = msg;
  setTimeout(() => {
    toastMessage.value = '';
  }, 2500);
};

const refreshData = async () => {
  isRefreshing.value = true;
  try {
    matchData.value = await getOngoingMatch();
    showToast('对局情报已刷新');
  } catch (err) {
    console.error(err);
  } finally {
    isRefreshing.value = false;
  }
};

const sendChat = async (format: 'concise' | 'detailed') => {
  isBroadcasting.value = true;
  try {
    const res = await sendChatRecords(matchData.value.chat_room_id || '', format);
    showToast(res.message);
  } catch (err) {
    console.error(err);
  } finally {
    isBroadcasting.value = false;
  }
};

onMounted(() => {
  refreshData();
});
</script>
