<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-[#090d1a] relative select-none">
    <!-- Top Action Bar -->
    <div class="h-14 px-6 border-b border-[#162238] flex items-center justify-between shrink-0 bg-[#070b14]/80 backdrop-blur z-20">
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
            {{ isDemoMode ? '离线模拟演练对局 (5v5 严格分路对位演示)' : (matchData?.queue_name || '实时对局侦查') }}
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
        <!-- 1-Click Chat Record Broadcast Options -->
        <div v-if="(matchData?.is_active || isDemoMode)" class="flex items-center space-x-1.5">
          <button
            @click="sendChat('concise')"
            :disabled="isBroadcasting"
            class="px-3 py-1.5 rounded-lg bg-[#0e273c] hover:bg-[#143452] border border-[#1b4366] text-[#38bdf8] text-xs font-semibold flex items-center space-x-1.5 transition active:scale-95 disabled:opacity-50 shadow-sm"
            title="将我方 5 人近期胜率与战绩以精简格式发送到选人聊天室"
          >
            <MessageSquare class="w-3.5 h-3.5" />
            <span>{{ isBroadcasting ? '广播中...' : '广播精简战绩' }}</span>
          </button>

          <button
            @click="sendChat('detailed')"
            :disabled="isBroadcasting"
            class="px-2.5 py-1.5 rounded-lg bg-[#0d2133] hover:bg-[#122e47] border border-[#183955] text-slate-300 hover:text-white text-xs font-medium transition active:scale-95 disabled:opacity-50"
            title="发送包含段位、胜率与车队标记的详细战绩"
          >
            <span>详细版</span>
          </button>
        </div>

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
          {{ matchData?.lcu_connected ? '英雄联盟客户端已连接 · 大厅待命' : '等待英雄联盟客户端启动' }}
        </h3>
        
        <p class="text-xs text-slate-400 leading-relaxed mb-6">
          {{ matchData?.lcu_connected 
              ? '当前处于大厅或组队状态。进入【英雄选择阶段】或【加载界面】后，HexBrain 智能反匿名引擎将自动为您穿透展示 10 人实战战绩、车队标记与 5v5 严格分路对位。' 
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
          <span>心跳嗅探进程运行中 · 每 2.5 秒自动同步</span>
        </div>
      </div>
    </div>

    <!-- Active 5v5 Dual Columns View with Strict Horizontal Lane Alignment -->
    <div v-else class="flex-1 flex flex-col overflow-hidden">
      <!-- Demo Mode Banner -->
      <div 
        v-if="isDemoMode" 
        class="h-8 px-6 bg-amber-950/60 border-b border-amber-600/30 flex items-center justify-between text-xs text-amber-300/90 shrink-0"
      >
        <span class="flex items-center space-x-1.5">
          <span>⚠️</span>
          <span>当前处于【模拟演练模式】：展示 5v5 严格分路横向对位 (上单对上单、打野对打野等) 与多排车队高亮效果</span>
        </span>
        <button 
          @click="toggleDemoMode" 
          class="text-[11px] underline hover:text-amber-100 transition"
        >
          退出演练回到实时监听
        </button>
      </div>

      <!-- Teams Summary Header Bar -->
      <div class="px-6 py-2 bg-[#080d19] border-b border-[#142137] grid grid-cols-[1fr_80px_1fr] gap-3 items-center shrink-0">
        <!-- Blue Header -->
        <div class="flex items-center justify-between px-3 py-1.5 bg-[#0b1626] rounded-lg border border-[#1b3456]">
          <div class="flex items-center space-x-2">
            <span class="w-2.5 h-2.5 rounded-full bg-blue-400"></span>
            <span class="text-xs font-bold text-blue-200 uppercase tracking-wider">我方阵营 (蓝方)</span>
            <span v-if="bluePremadeSummary" class="text-[10px] px-1.5 py-0.2 rounded bg-rose-950/80 text-rose-300 border border-rose-600/40 font-semibold">
              {{ bluePremadeSummary }}
            </span>
          </div>
          <span class="text-[11px] text-blue-300/70 font-mono">{{ (matchData?.blue_team || []).length }} 位召唤师</span>
        </div>

        <!-- Center Matchup Tag -->
        <div class="text-center">
          <span class="text-[10px] font-black text-slate-400 tracking-widest uppercase bg-[#101b2e] px-2 py-0.5 rounded border border-[#1e2e4a]">
            VS 对位
          </span>
        </div>

        <!-- Red Header -->
        <div class="flex items-center justify-between px-3 py-1.5 bg-[#240e14] rounded-lg border border-[#4a1a24]">
          <span class="text-[11px] text-red-300/70 font-mono">{{ (matchData?.red_team || []).length }} 位召唤师</span>
          <div class="flex items-center space-x-2">
            <span v-if="redPremadeSummary" class="text-[10px] px-1.5 py-0.2 rounded bg-amber-950/80 text-amber-300 border border-amber-600/40 font-semibold">
              {{ redPremadeSummary }}
            </span>
            <span class="text-xs font-bold text-red-200 uppercase tracking-wider">敌方阵营 (红方)</span>
            <span class="w-2.5 h-2.5 rounded-full bg-red-400"></span>
          </div>
        </div>
      </div>

      <!-- 5v5 Paired Lane Rows Scroll Flow -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        <div 
          v-for="pair in pairedLaneRows" 
          :key="pair.key"
          class="grid grid-cols-[1fr_80px_1fr] gap-3 items-stretch"
        >
          <!-- Left: Blue Player Card -->
          <div 
            v-if="pair.blue"
            @click="selectPlayer(pair.blue)"
            class="p-3 rounded-xl bg-[#0b1220] border transition-all cursor-pointer hover:bg-[#101b30] hover:border-[#23385d] relative group flex flex-col justify-between"
            :class="[
              getPlayerBorderClass(pair.blue, selectedPlayer?.puuid === pair.blue.puuid, true),
              selectedPlayer?.puuid === pair.blue.puuid ? 'ring-1 ring-[#0ac8b9]' : ''
            ]"
          >
            <!-- Top Section: Avatar, Spells, Name, Ranks, Performance -->
            <div class="flex items-center justify-between">
              <!-- Left: Avatar & Spells -->
              <div class="flex items-center space-x-3">
                <div class="relative shrink-0">
                  <img 
                    :src="pair.blue.champion_icon || getChampIconFallback(pair.blue.champion_id)" 
                    class="w-12 h-12 rounded-lg border border-[#1e2e4a] object-cover transition-transform group-hover:scale-105"
                  />
                  <span 
                    v-if="pair.blue.assigned_position" 
                    class="absolute -bottom-1 -left-1 px-1 py-0.2 bg-[#070b14] border border-[#1e2e4a] text-[9px] font-bold text-slate-300 rounded"
                  >
                    {{ formatPositionTag(pair.blue.assigned_position) }}
                  </span>
                  <span 
                    v-if="pair.blue.is_locked"
                    class="absolute -top-1 -right-1 w-3.5 h-3.5 rounded-full bg-[#0ac8b9] border border-[#070b14] flex items-center justify-center text-[8px] text-black font-black"
                    title="已锁定英雄"
                  >
                    ✓
                  </span>
                </div>

                <!-- Spells -->
                <div class="flex flex-col space-y-1 shrink-0">
                  <img :src="pair.blue.spell1_icon" class="w-5 h-5 rounded border border-[#1e2e4a]" :title="pair.blue.spell1_name" />
                  <img :src="pair.blue.spell2_icon" class="w-5 h-5 rounded border border-[#1e2e4a]" :title="pair.blue.spell2_name" />
                </div>

                <!-- Name & Rank -->
                <div class="flex flex-col min-w-0">
                  <div class="flex items-center space-x-1.5 truncate">
                    <span class="text-xs font-bold text-slate-100 group-hover:text-[#0ac8b9] transition-colors truncate">
                      {{ pair.blue.game_name }}
                    </span>
                    <span class="text-[10px] text-slate-400 shrink-0">#{{ pair.blue.tag_line }}</span>
                    <!-- Premade Tag -->
                    <span 
                      v-if="pair.blue.premade_party"
                      class="px-1.5 py-0.2 rounded text-[9px] font-bold shrink-0 shadow-sm"
                      :class="pair.blue.premade_party.party_id === 1 ? 'bg-rose-950/90 text-rose-300 border border-rose-500/60' : 'bg-amber-950/90 text-amber-300 border border-amber-500/60'"
                    >
                      {{ pair.blue.premade_party.label }}
                    </span>
                  </div>
                  <div class="flex items-center space-x-2 mt-0.5">
                    <span class="text-[11px] font-semibold text-[#c8aa6e]">{{ pair.blue.tier }} {{ pair.blue.division }}</span>
                    <span class="text-[10px] text-slate-400 font-mono">{{ pair.blue.lp }} LP</span>
                  </div>
                </div>
              </div>

              <!-- Right: Performance & Recent form -->
              <div class="flex flex-col items-end space-y-1 shrink-0 pl-2">
                <div class="flex items-center space-x-2">
                  <span class="text-xs font-bold" :class="pair.blue.recent_win_rate >= 60 ? 'text-[#0ac8b9]' : 'text-slate-200'">
                    {{ pair.blue.recent_win_rate }}% 胜率
                  </span>
                  <span class="text-[11px] text-slate-400 font-mono">{{ pair.blue.recent_kda }}</span>
                </div>

                <!-- Recent 5 Games dots -->
                <div class="flex items-center space-x-1">
                  <span 
                    v-for="(m, i) in (pair.blue.recent_matches || [])" 
                    :key="i"
                    class="w-2 h-2 rounded-full shadow-sm"
                    :class="m === 'win' ? 'bg-[#0ac8b9]' : 'bg-red-500'"
                    :title="m === 'win' ? '胜利' : '失败'"
                  ></span>
                </div>
              </div>
            </div>

            <!-- Bottom Badges Row -->
            <div v-if="pair.blue.badges && pair.blue.badges.length > 0" class="mt-2.5 pt-2 border-t border-[#131d2e] flex flex-wrap gap-1.5">
              <span
                v-for="b in pair.blue.badges"
                :key="b.id"
                class="px-1.5 py-0.5 rounded text-[10px] flex items-center space-x-1 font-semibold border transition"
                :class="getBadgeClass(b)"
                :title="b.description || b.label"
              >
                <span>{{ b.icon }}</span>
                <span>{{ b.label }}</span>
              </span>
            </div>
          </div>
          <!-- Empty Blue slot fallback -->
          <div v-else class="p-3 rounded-xl bg-[#090e18]/40 border border-dashed border-[#18263d] flex items-center justify-center text-xs text-slate-500">
            等待玩家进入...
          </div>

          <!-- Center: Lane Matchup Indicator Pillar -->
          <div class="flex flex-col items-center justify-center py-2 px-1 rounded-xl bg-[#0a1120] border border-[#17253d] relative overflow-hidden group shadow-inner">
            <span class="text-base select-none">{{ pair.icon }}</span>
            <span class="text-[11px] font-black text-slate-200 tracking-wider mt-0.5 uppercase">
              {{ pair.shortLabel }}
            </span>
            <span class="text-[9px] font-medium text-slate-400 mt-0.5">
              {{ pair.label }}
            </span>
          </div>

          <!-- Right: Red Player Card -->
          <div 
            v-if="pair.red"
            @click="selectPlayer(pair.red)"
            class="p-3 rounded-xl bg-[#140b10] border transition-all cursor-pointer hover:bg-[#1f0f18] hover:border-[#4d1f2b] relative group flex flex-col justify-between"
            :class="[
              getPlayerBorderClass(pair.red, selectedPlayer?.puuid === pair.red.puuid, false),
              selectedPlayer?.puuid === pair.red.puuid ? 'ring-1 ring-red-500' : ''
            ]"
          >
            <!-- Top Section: Avatar, Spells, Name, Ranks, Performance -->
            <div class="flex items-center justify-between">
              <!-- Left: Avatar & Spells -->
              <div class="flex items-center space-x-3">
                <div class="relative shrink-0">
                  <img 
                    :src="pair.red.champion_icon || getChampIconFallback(pair.red.champion_id)" 
                    class="w-12 h-12 rounded-lg border border-[#3b1924] object-cover transition-transform group-hover:scale-105"
                  />
                  <span 
                    v-if="pair.red.assigned_position" 
                    class="absolute -bottom-1 -left-1 px-1 py-0.2 bg-[#070b14] border border-[#3b1924] text-[9px] font-bold text-slate-300 rounded"
                  >
                    {{ formatPositionTag(pair.red.assigned_position) }}
                  </span>
                  <span 
                    v-if="pair.red.is_locked"
                    class="absolute -top-1 -right-1 w-3.5 h-3.5 rounded-full bg-red-500 border border-[#070b14] flex items-center justify-center text-[8px] text-white font-black"
                    title="已锁定英雄"
                  >
                    ✓
                  </span>
                </div>

                <!-- Spells -->
                <div class="flex flex-col space-y-1 shrink-0">
                  <img :src="pair.red.spell1_icon" class="w-5 h-5 rounded border border-[#3b1924]" :title="pair.red.spell1_name" />
                  <img :src="pair.red.spell2_icon" class="w-5 h-5 rounded border border-[#3b1924]" :title="pair.red.spell2_name" />
                </div>

                <!-- Name & Rank -->
                <div class="flex flex-col min-w-0">
                  <div class="flex items-center space-x-1.5 truncate">
                    <span class="text-xs font-bold text-slate-100 group-hover:text-red-400 transition-colors truncate">
                      {{ pair.red.game_name }}
                    </span>
                    <span class="text-[10px] text-slate-400 shrink-0">#{{ pair.red.tag_line }}</span>
                    <!-- Premade Tag -->
                    <span 
                      v-if="pair.red.premade_party"
                      class="px-1.5 py-0.2 rounded text-[9px] font-bold shrink-0 shadow-sm"
                      :class="pair.red.premade_party.party_id === 1 ? 'bg-rose-950/90 text-rose-300 border border-rose-500/60' : 'bg-amber-950/90 text-amber-300 border border-amber-500/60'"
                    >
                      {{ pair.red.premade_party.label }}
                    </span>
                  </div>
                  <div class="flex items-center space-x-2 mt-0.5">
                    <span class="text-[11px] font-semibold text-[#c8aa6e]">{{ pair.red.tier }} {{ pair.red.division }}</span>
                    <span class="text-[10px] text-slate-400 font-mono">{{ pair.red.lp }} LP</span>
                  </div>
                </div>
              </div>

              <!-- Right: Performance & Recent form -->
              <div class="flex flex-col items-end space-y-1 shrink-0 pl-2">
                <div class="flex items-center space-x-2">
                  <span class="text-xs font-bold" :class="pair.red.recent_win_rate >= 60 ? 'text-[#0ac8b9]' : 'text-slate-200'">
                    {{ pair.red.recent_win_rate }}% 胜率
                  </span>
                  <span class="text-[11px] text-slate-400 font-mono">{{ pair.red.recent_kda }}</span>
                </div>

                <!-- Recent 5 Games dots -->
                <div class="flex items-center space-x-1">
                  <span 
                    v-for="(m, i) in (pair.red.recent_matches || [])" 
                    :key="i"
                    class="w-2 h-2 rounded-full shadow-sm"
                    :class="m === 'win' ? 'bg-[#0ac8b9]' : 'bg-red-500'"
                    :title="m === 'win' ? '胜利' : '失败'"
                  ></span>
                </div>
              </div>
            </div>

            <!-- Bottom Badges Row -->
            <div v-if="pair.red.badges && pair.red.badges.length > 0" class="mt-2.5 pt-2 border-t border-[#29131d] flex flex-wrap gap-1.5">
              <span
                v-for="b in pair.red.badges"
                :key="b.id"
                class="px-1.5 py-0.5 rounded text-[10px] flex items-center space-x-1 font-semibold border transition"
                :class="getBadgeClass(b)"
                :title="b.description || b.label"
              >
                <span>{{ b.icon }}</span>
                <span>{{ b.label }}</span>
              </span>
            </div>
          </div>
          <!-- Empty Red slot fallback -->
          <div v-else class="p-3 rounded-xl bg-[#140b10]/40 border border-dashed border-[#3b1924] flex items-center justify-center text-xs text-slate-500">
            等待玩家进入...
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
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { 
  MessageSquare, 
  RotateCw, 
  Check, 
  Gamepad2, 
  Radio, 
  AlertCircle, 
  Sparkles 
} from 'lucide-vue-next';
import { PlayerCard, OngoingMatchData, TacticalBadge } from '../api/types';
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
  showToast(isDemoMode.value ? '已开启【模拟演练模式】(5v5 严格分路对位)' : '已退出演练，恢复实时对局感知');
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
    console.error('sendChat error:', err);
    showToast('战绩广播失败，请确认客户端与聊天室在线');
  } finally {
    isBroadcasting.value = false;
  }
};

function normalizePosition(pos?: string): string {
  if (!pos) return '';
  const p = pos.toUpperCase();
  if (p === 'TOP' || p === '上单') return 'TOP';
  if (p === 'JUNGLE' || p === 'JUG' || p === '打野') return 'JUNGLE';
  if (p === 'MID' || p === 'MIDDLE' || p === '中单') return 'MID';
  if (p === 'ADC' || p === 'BOTTOM' || p === 'BOT' || p === '下路') return 'ADC';
  if (p === 'SUPPORT' || p === 'UTILITY' || p === 'SUP' || p === '辅助') return 'SUPPORT';
  return '';
}

function formatPositionTag(pos?: string): string {
  const norm = normalizePosition(pos);
  switch (norm) {
    case 'TOP': return '上路';
    case 'JUNGLE': return '打野';
    case 'MID': return '中路';
    case 'ADC': return '下路';
    case 'SUPPORT': return '辅助';
    default: return pos || '';
  }
}

interface LaneRowPair {
  key: string;
  label: string;
  shortLabel: string;
  icon: string;
  blue: PlayerCard | null;
  red: PlayerCard | null;
}

const pairedLaneRows = computed<LaneRowPair[]>(() => {
  const blueList = matchData.value?.blue_team || [];
  const redList = matchData.value?.red_team || [];

  const laneDefs = [
    { key: 'TOP', label: '上路', shortLabel: 'TOP', icon: '🛡️' },
    { key: 'JUNGLE', label: '打野', shortLabel: 'JUG', icon: '🌲' },
    { key: 'MID', label: '中路', shortLabel: 'MID', icon: '⚡' },
    { key: 'ADC', label: '下路', shortLabel: 'BOT', icon: '🏹' },
    { key: 'SUPPORT', label: '辅助', shortLabel: 'SUP', icon: '❤️' },
  ];

  const hasStandardPositions = blueList.some(p => normalizePosition(p.assigned_position)) || 
                               redList.some(p => normalizePosition(p.assigned_position));

  if (hasStandardPositions) {
    const blueRemaining = [...blueList];
    const redRemaining = [...redList];

    return laneDefs.map((def) => {
      // Find blue player matching this position
      const bIdx = blueRemaining.findIndex(p => normalizePosition(p.assigned_position) === def.key);
      let bluePlayer: PlayerCard | null = null;
      if (bIdx !== -1) {
        bluePlayer = blueRemaining.splice(bIdx, 1)[0];
      }

      // Find red player matching this position
      const rIdx = redRemaining.findIndex(p => normalizePosition(p.assigned_position) === def.key);
      let redPlayer: PlayerCard | null = null;
      if (rIdx !== -1) {
        redPlayer = redRemaining.splice(rIdx, 1)[0];
      }

      // Fallback: if not explicitly assigned, pick from remaining unassigned
      if (!bluePlayer && blueRemaining.length > 0) {
        bluePlayer = blueRemaining.shift() || null;
      }
      if (!redPlayer && redRemaining.length > 0) {
        redPlayer = redRemaining.shift() || null;
      }

      return {
        key: def.key,
        label: def.label,
        shortLabel: def.shortLabel,
        icon: def.icon,
        blue: bluePlayer,
        red: redPlayer,
      };
    });
  } else {
    // Non-positioned games (ARAM or custom): pair row-by-row up to max team length
    const maxLen = Math.max(blueList.length, redList.length, 5);
    const rows: LaneRowPair[] = [];
    for (let i = 0; i < maxLen; i++) {
      const fallbackDef = laneDefs[i] || { key: `slot_${i}`, label: `席位 ${i + 1}`, shortLabel: `#${i + 1}`, icon: '⚔️' };
      rows.push({
        key: fallbackDef.key,
        label: fallbackDef.label,
        shortLabel: fallbackDef.shortLabel,
        icon: fallbackDef.icon,
        blue: blueList[i] || null,
        red: redList[i] || null,
      });
    }
    return rows;
  }
});

const bluePremadeSummary = computed(() => {
  const parties = (matchData.value?.blue_team || []).filter(p => p.premade_party);
  if (parties.length === 0) return '';
  const first = parties[0].premade_party?.label || '车队';
  return `🚗 蓝方检测到 ${first}`;
});

const redPremadeSummary = computed(() => {
  const parties = (matchData.value?.red_team || []).filter(p => p.premade_party);
  if (parties.length === 0) return '';
  const first = parties[0].premade_party?.label || '车队';
  return `🚗 红方检测到 ${first}`;
});

function getPlayerBorderClass(player: PlayerCard, isSelected: boolean, isBlue: boolean): string {
  if (isSelected) {
    return isBlue ? 'border-[#0ac8b9] ring-2 ring-[#0ac8b9]/50' : 'border-red-500 ring-2 ring-red-500/50';
  }
  if (player.premade_party) {
    if (player.premade_party.color_class) {
      return player.premade_party.color_class;
    }
    return player.premade_party.party_id === 1 ? 'party-border-pink' : 'party-border-yellow';
  }
  return isBlue ? 'border-[#152033]' : 'border-[#2d141e]';
}

function getBadgeClass(badge: TacticalBadge): string {
  switch (badge.color) {
    case 'gold':
      return 'bg-amber-950/80 border-amber-500/50 text-amber-300 shadow-[0_0_8px_rgba(245,158,11,0.25)]';
    case 'red':
      return 'bg-red-950/80 border-red-500/50 text-red-300 shadow-[0_0_8px_rgba(239,68,68,0.25)]';
    case 'cyan':
      return 'bg-cyan-950/80 border-cyan-500/50 text-cyan-300 shadow-[0_0_8px_rgba(6,182,212,0.25)]';
    case 'green':
      return 'bg-emerald-950/80 border-emerald-500/50 text-emerald-300 shadow-[0_0_8px_rgba(16,185,129,0.25)]';
    case 'purple':
      return 'bg-purple-950/80 border-purple-500/50 text-purple-300 shadow-[0_0_8px_rgba(168,85,247,0.25)]';
    case 'blue':
    default:
      return 'bg-blue-950/80 border-blue-500/50 text-[#38bdf8] shadow-[0_0_8px_rgba(56,189,248,0.25)]';
  }
}

function getChampIconFallback(champId: number): string {
  if (!champId) return '';
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/${champId}.png`;
}

onMounted(() => {
  refreshData();
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
