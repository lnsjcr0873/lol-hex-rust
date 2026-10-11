<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-[#090d1a] relative select-none">
    <!-- Top Action Bar -->
    <div class="h-14 px-6 border-b border-[#162238] flex items-center justify-between shrink-0 bg-[#070b14]/70 backdrop-blur">
      <div class="flex items-center space-x-3">
        <div class="flex items-center space-x-2">
          <span 
            class="w-2.5 h-2.5 rounded-full" 
            :class="[
              isDemoMode ? 'bg-amber-400 animate-pulse' :
              (matchData?.lcu_connected ? (matchData?.is_active ? 'bg-[#0ac8b9] animate-pulse' : 'bg-emerald-400') : 'bg-yellow-500/70')
            ]"
          ></span>
          <h2 class="text-base font-bold text-slate-100 tracking-wide">
            {{ isDemoMode ? '离线模拟演练对局 (5v5 排版演示)' : (matchData?.queue_name || '实时对局侦查') }}
          </h2>
        </div>
        
        <span 
          v-if="isDemoMode" 
          class="px-2 py-0.5 rounded text-xs bg-amber-950/80 border border-amber-600/50 text-amber-300 font-medium"
        >
          演练模式
        </span>
        <span 
          v-else-if="matchData?.lcu_connected && matchData?.is_active" 
          class="px-2 py-0.5 rounded text-xs bg-[#101b2e] border border-[#1e2e4a] text-[#0ac8b9] font-medium"
        >
          {{ matchData?.gameflow_phase === 'ChampSelect' ? '英雄选择中' : '游戏中' }}
        </span>
        <span 
          v-else-if="matchData?.lcu_connected" 
          class="px-2 py-0.5 rounded text-xs bg-emerald-950/60 border border-emerald-600/40 text-emerald-300 font-medium"
        >
          客户端在线
        </span>
        <span 
          v-else 
          class="px-2 py-0.5 rounded text-xs bg-slate-800/80 border border-slate-700 text-slate-400 font-medium"
        >
          未连接
        </span>
      </div>

      <!-- Quick Actions -->
      <div class="flex items-center space-x-3">
        <!-- Demo Mode Toggle -->
        <button
          @click="toggleDemoMode"
          class="px-3 py-1.5 rounded-lg border text-xs font-medium flex items-center space-x-1.5 transition active:scale-95"
          :class="isDemoMode ? 'bg-amber-950/80 hover:bg-amber-900 border-amber-600 text-amber-200' : 'bg-[#0f172a] hover:bg-[#162238] border-[#1e293b] text-slate-300'"
        >
          <Gamepad2 class="w-3.5 h-3.5" />
          <span>{{ isDemoMode ? '退出演练 (切回实时)' : '模拟演练模式' }}</span>
        </button>

        <button
          v-if="(matchData?.is_active || isDemoMode)"
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

    <!-- Offline Empty State View -->
    <div 
      v-if="!isDemoMode && (!matchData?.lcu_connected || !matchData?.is_active)" 
      class="flex-1 flex flex-col items-center justify-center p-8 text-center"
    >
      <div class="max-w-md p-8 rounded-2xl bg-[#0c1424]/90 border border-[#192b4a] shadow-2xl flex flex-col items-center relative overflow-hidden backdrop-blur">
        <!-- Background Ambient Glow -->
        <div class="absolute -top-16 -right-16 w-32 h-32 bg-[#0ac8b9]/10 rounded-full blur-2xl pointer-events-none"></div>
        <div class="absolute -bottom-16 -left-16 w-32 h-32 bg-blue-600/10 rounded-full blur-2xl pointer-events-none"></div>

        <div class="w-16 h-16 rounded-2xl bg-[#101b30] border border-[#1e2e4a] flex items-center justify-center mb-5 text-[#0ac8b9] shadow-inner">
          <Radio v-if="matchData?.lcu_connected" class="w-8 h-8 animate-pulse text-emerald-400" />
          <AlertCircle v-else class="w-8 h-8 text-amber-400" />
        </div>

        <h3 class="text-lg font-bold text-slate-100 mb-2">
          {{ matchData?.lcu_connected ? '英雄联盟客户端已连接' : '等待英雄联盟客户端启动' }}
        </h3>
        
        <p class="text-xs text-slate-400 leading-relaxed mb-6">
          {{ matchData?.lcu_connected 
              ? '当前处于大厅或组队状态。进入【英雄选择阶段】或【加载界面】后，HexBrain 智能反匿名引擎将自动为您穿透展示 10 人实战战绩与车队标记。' 
              : '后台已开启实时嗅探。请启动英雄联盟并登录账号，系统将即时捕获对局信息，无需手动配置。' 
          }}
        </p>

        <div class="flex items-center space-x-3 w-full">
          <button
            @click="toggleDemoMode"
            class="flex-1 px-4 py-2.5 rounded-xl bg-gradient-to-r from-[#0ac8b9]/20 to-blue-600/20 hover:from-[#0ac8b9]/30 hover:to-blue-600/30 border border-[#0ac8b9]/40 text-[#0ac8b9] hover:text-white text-xs font-semibold flex items-center justify-center space-x-2 transition active:scale-95 shadow-lg"
          >
            <Sparkles class="w-4 h-4" />
            <span>开启演练模拟模式</span>
          </button>

          <button
            @click="refreshData"
            class="px-4 py-2.5 rounded-xl bg-[#121c2e] hover:bg-[#1a2942] border border-[#233552] text-slate-300 text-xs font-medium transition active:scale-95"
          >
            立即检测
          </button>
        </div>

        <div class="mt-4 flex items-center space-x-1.5 text-[11px] text-slate-500">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-ping"></span>
          <span>心跳嗅探进程运行中 · 每秒自动轮询</span>
        </div>
      </div>
    </div>

    <!-- Active 5v5 Dual Columns View (Real match or Demo mode) -->
    <div v-else class="flex-1 flex flex-col overflow-hidden">
      <!-- Demo Mode Banner -->
      <div 
        v-if="isDemoMode" 
        class="h-8 px-6 bg-amber-950/60 border-b border-amber-600/30 flex items-center justify-between text-xs text-amber-300/90"
      >
        <span class="flex items-center space-x-1.5">
          <span>⚠️</span>
          <span>当前处于【模拟演练模式】，用于预览 10 人透视排版与车队标记效果</span>
        </span>
        <button 
          @click="toggleDemoMode" 
          class="text-[11px] underline hover:text-amber-100 transition"
        >
          退出演练回到实时监听
        </button>
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
            <span class="text-[11px] text-blue-300/70">{{ (matchData?.blue_team || []).length }} 位召唤师已就绪</span>
          </div>

          <!-- Blue Cards -->
          <div 
            v-for="player in matchData?.blue_team || []" 
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
            v-for="player in matchData?.red_team || []" 
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
import { ref, onMounted, onUnmounted } from 'vue';
import { 
  MessageSquare, 
  RotateCw, 
  Check, 
  Gamepad2, 
  Radio, 
  AlertCircle, 
  Sparkles 
} from 'lucide-vue-next';
import { PlayerCard, OngoingMatchData } from '../api/types';
import { getOngoingMatch, sendChatRecords } from '../api/tauri';
import PlayerDetailDrawer from '../components/PlayerDetailDrawer.vue';

const matchData = ref<OngoingMatchData | null>(null);
const selectedPlayer = ref<PlayerCard | null>(null);
const isRefreshing = ref(false);
const isBroadcasting = ref(false);
const isDemoMode = ref(false);
const toastMessage = ref('');
let pollTimer: any = null;

const selectPlayer = (player: PlayerCard) => {
  selectedPlayer.value = player;
};

const showToast = (msg: string) => {
  toastMessage.value = msg;
  setTimeout(() => {
    toastMessage.value = '';
  }, 2500);
};

const toggleDemoMode = async () => {
  isDemoMode.value = !isDemoMode.value;
  await refreshData();
  showToast(isDemoMode.value ? '已开启【模拟演练模式】' : '已退出演练，恢复实时对局感知');
};

const refreshData = async () => {
  isRefreshing.value = true;
  try {
    matchData.value = await getOngoingMatch(isDemoMode.value);
  } catch (err) {
    console.error('refreshData error:', err);
  } finally {
    isRefreshing.value = false;
  }
};

const sendChat = async (format: 'concise' | 'detailed') => {
  isBroadcasting.value = true;
  try {
    const res = await sendChatRecords(matchData.value?.chat_room_id || '', format);
    showToast(res.message);
  } catch (err) {
    console.error(err);
  } finally {
    isBroadcasting.value = false;
  }
};

onMounted(() => {
  refreshData();
  // Auto poll every 2.5s when not in demo mode
  pollTimer = setInterval(() => {
    if (!isDemoMode.value && !isRefreshing.value) {
      getOngoingMatch(false).then((res) => {
        matchData.value = res;
      }).catch(() => {});
    }
  }, 2500);
});

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});
</script>
