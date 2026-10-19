<template>
  <div class="flex-1 overflow-y-auto p-6 bg-[#090d1a] select-none space-y-6">
    <div class="border-b border-[#162238] pb-4">
      <h2 class="text-base font-bold text-slate-100 flex items-center space-x-2">
        <span>系统参数与运行设置</span>
      </h2>
      <p class="text-xs text-slate-400 mt-1">
        检查客户端与 LCU 链路连接状态、调节界面偏好与查看底层引擎规格。
      </p>
    </div>

    <!-- Group 1: LCU Client Connection Status -->
    <div class="bg-[#0c1424] p-5 rounded-2xl border border-[#162238] space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-bold text-slate-200">英雄联盟客户端与 LCU 链路</h3>
        <button
          @click="checkClientStatus"
          :disabled="isChecking"
          class="px-3 py-1 rounded-lg bg-[#0e273c] hover:bg-[#143452] border border-[#1b4366] text-[#38bdf8] text-xs font-semibold flex items-center space-x-1.5 transition active:scale-95 disabled:opacity-50"
        >
          <RotateCw class="w-3.5 h-3.5" :class="{ 'animate-spin': isChecking }" />
          <span>重新检测链路</span>
        </button>
      </div>

      <div class="grid grid-cols-2 gap-4 text-xs">
        <div class="bg-[#080e1b] p-3 rounded-xl border border-[#152238] space-y-1.5">
          <div class="text-slate-400">LCU 客户端状态</div>
          <div class="flex items-center space-x-2 pt-0.5">
            <span
              class="w-2.5 h-2.5 rounded-full"
              :class="clientConnected ? 'bg-[#0ac8b9] animate-pulse' : 'bg-rose-500'"
            ></span>
            <span class="text-slate-100 font-bold font-mono">
              {{ clientConnected ? '已建立安全双工连接' : '未感知到 LeagueClient 进程' }}
            </span>
          </div>
          <p class="text-[11px] text-slate-500">
            {{ clientConnected ? '已自动从 lockfile 解析通信端口与安全令牌' : '请在游戏客户端启动并登录后重新检测' }}
          </p>
        </div>

        <div class="bg-[#080e1b] p-3 rounded-xl border border-[#152238] space-y-1.5">
          <div class="text-slate-400">当前召唤师</div>
          <div class="text-slate-100 font-bold font-mono pt-0.5 truncate">
            {{ currentSummonerName || '—' }}
          </div>
          <p class="text-[11px] text-slate-500">
            {{ currentSummonerName ? '当前处于实时同步模式' : '请先登录国服客户端' }}
          </p>
        </div>
      </div>
    </div>

    <!-- Group 2: Display & Broadcast Preferences -->
    <div class="bg-[#0c1424] p-5 rounded-2xl border border-[#162238] space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-bold text-slate-200">交互与广播偏好</h3>
        <span class="text-[11px] text-slate-400">本地首选项配置</span>
      </div>

      <div class="grid grid-cols-2 gap-4 text-xs">
        <div class="bg-[#080e1b] p-3 rounded-xl border border-[#152238] space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-slate-200 font-semibold">战绩默认拉取深度</span>
            <span class="text-slate-400 font-mono">{{ preferences.defaultPageSize }} 场/页</span>
          </div>
          <p class="text-slate-400 text-[11px]">进入对局与战绩页时首次拉取的历史对局样本数</p>
          <div class="flex items-center space-x-2 pt-1">
            <button
              v-for="size in [10, 20, 50, 100]"
              :key="size"
              @click="setPageSize(size)"
              class="px-2.5 py-1 rounded text-xs font-mono border transition"
              :class="preferences.defaultPageSize === size ? 'bg-[#0f233a] text-[#0ac8b9] border-[#0ac8b9]' : 'bg-slate-900 text-slate-400 border-slate-800 hover:text-slate-200'"
            >
              {{ size }} 场
            </button>
          </div>
        </div>

        <div class="bg-[#080e1b] p-3 rounded-xl border border-[#152238] space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-slate-200 font-semibold">选人聊天广播偏好</span>
            <span class="text-[#0ac8b9] font-mono">{{ preferences.broadcastFormat === 'concise' ? '精简格式' : '详细格式' }}</span>
          </div>
          <p class="text-slate-400 text-[11px]">点击一键广播时默认发送的战绩排版风格</p>
          <div class="flex items-center space-x-2 pt-1">
            <button
              @click="setBroadcastFormat('concise')"
              class="px-3 py-1 rounded text-xs border transition"
              :class="preferences.broadcastFormat === 'concise' ? 'bg-[#0f233a] text-[#0ac8b9] border-[#0ac8b9]' : 'bg-slate-900 text-slate-400 border-slate-800 hover:text-slate-200'"
            >
              精简版 (胜率与常用)
            </button>
            <button
              @click="setBroadcastFormat('detailed')"
              class="px-3 py-1 rounded text-xs border transition"
              :class="preferences.broadcastFormat === 'detailed' ? 'bg-[#0f233a] text-[#0ac8b9] border-[#0ac8b9]' : 'bg-slate-900 text-slate-400 border-slate-800 hover:text-slate-200'"
            >
              详细版 (含车队标签)
            </button>
          </div>
        </div>
      </div>

      <!-- Save Status Toast -->
      <div 
        v-if="statusText" 
        class="text-xs text-[#0ac8b9] flex items-center space-x-1.5 transition-opacity pt-1"
      >
        <Check class="w-3.5 h-3.5" />
        <span>{{ statusText }}</span>
      </div>
    </div>

    <!-- Group 3: LCU Automation Policies (League-Akari Aligned) -->
    <div class="bg-[#0c1424] p-5 rounded-2xl border border-[#162238] space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-bold text-slate-200">LCU 自动化辅助策略</h3>
        <span class="text-[11px] text-emerald-400 font-mono">底层原生事件驱动 · 实时生效</span>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
        <!-- Auto Accept -->
        <div class="bg-[#080e1b] p-3.5 rounded-xl border border-[#152238] space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <div class="text-slate-200 font-semibold">自动接受对局就绪</div>
              <p class="text-slate-400 text-[11px] mt-0.5">检测到匹配就绪弹窗时向 LCU 下发确认</p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                v-model="automation.auto_accept_match"
                @change="saveAutomation"
                class="sr-only peer"
              />
              <div class="w-9 h-5 bg-[#162238] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-[#0ac8b9]"></div>
            </label>
          </div>

          <div v-if="automation.auto_accept_match" class="pt-2 border-t border-[#141f33] space-y-1.5">
            <div class="flex items-center justify-between text-[11px]">
              <span class="text-slate-400">拟人化延时</span>
              <span class="text-[#0ac8b9] font-mono font-bold">{{ automation.auto_accept_delay_seconds }} 秒</span>
            </div>
            <input
              type="range"
              min="0"
              max="5"
              step="1"
              v-model.number="automation.auto_accept_delay_seconds"
              @change="saveAutomation"
              class="w-full h-1.5 bg-[#162238] rounded-lg appearance-none cursor-pointer accent-[#0ac8b9]"
            />
          </div>
        </div>

        <!-- Auto Apply Runes -->
        <div class="bg-[#080e1b] p-3.5 rounded-xl border border-[#152238] space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <div class="text-slate-200 font-semibold">一键 / 自动配置权威胜率符文</div>
              <p class="text-slate-400 text-[11px] mt-0.5">锁定英雄后自动写入版本最高胜率符文</p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                v-model="automation.auto_apply_runes"
                @change="saveAutomation"
                class="sr-only peer"
              />
              <div class="w-9 h-5 bg-[#162238] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-[#0ac8b9]"></div>
            </label>
          </div>
        </div>

        <!-- Auto Swap ARAM Bench -->
        <div class="bg-[#080e1b] p-3.5 rounded-xl border border-[#152238] space-y-3 md:col-span-2">
          <div class="flex items-center justify-between">
            <div>
              <div class="text-slate-200 font-semibold">极地大乱斗长椅英雄极速秒抢</div>
              <p class="text-slate-400 text-[11px] mt-0.5">队友丢出备战席英雄时，毫秒级响应自动交换</p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                v-model="automation.auto_swap_bench"
                @change="saveAutomation"
                class="sr-only peer"
              />
              <div class="w-9 h-5 bg-[#162238] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-[#0ac8b9]"></div>
            </label>
          </div>
        </div>
      </div>
    </div>

    <!-- Group 4: Architecture & Build Info -->
    <div class="bg-[#0c1424] p-5 rounded-2xl border border-[#162238] space-y-3">
      <h3 class="text-sm font-bold text-slate-200">架构与构建规格</h3>
      <div class="space-y-1.5 text-xs text-slate-400">
        <div class="flex justify-between py-1 border-b border-[#141e30]">
          <span>桌面容器运行时</span>
          <span class="text-slate-200 font-mono">Tauri v2.1 + WebView2</span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#141e30]">
          <span>底层核心引擎</span>
          <span class="text-slate-200 font-mono">Rust 2021 (x86_64-pc-windows-gnu)</span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#141e30]">
          <span>前端技术栈</span>
          <span class="text-slate-200 font-mono">Vue 3 + TypeScript + Naive UI + Tailwind CSS</span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#141e30]">
          <span>数据合规性</span>
          <span class="text-emerald-400 font-mono">100% Riot LCU 官方接口对齐 · 零内存挂钩 · 零注入</span>
        </div>
        <div class="flex justify-between py-1">
          <span>构建与发布流水线</span>
          <span class="text-[#0ac8b9] font-mono">GitHub Actions (Windows x86_64 runner)</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Check, RotateCw } from 'lucide-vue-next';
import { getSummonerProfile, getAutomationConfig, updateAutomationConfig } from '../api/tauri';
import type { AutomationConfig } from '../api/types';

interface UserPreferences {
  defaultPageSize: number;
  broadcastFormat: 'concise' | 'detailed';
}

const defaultPreferences: UserPreferences = {
  defaultPageSize: 20,
  broadcastFormat: 'concise'
};

const preferences = ref<UserPreferences>({ ...defaultPreferences });
const automation = ref<AutomationConfig>({
  auto_accept_match: false,
  auto_accept_delay_seconds: 2,
  auto_swap_bench: false,
  auto_apply_runes: false,
  bench_preferred_champions: []
});

const statusText = ref('');
let statusTimer: ReturnType<typeof setTimeout> | null = null;
const isChecking = ref(false);
const clientConnected = ref(false);
const currentSummonerName = ref('');

const checkClientStatus = async () => {
  isChecking.value = true;
  try {
    const profile = await getSummonerProfile();
    if (profile && profile.is_connected && profile.game_name !== '未连接客户端') {
      clientConnected.value = true;
      currentSummonerName.value = profile.tag_line ? `${profile.game_name}#${profile.tag_line}` : profile.game_name;
    } else {
      clientConnected.value = false;
      currentSummonerName.value = '';
    }
  } catch (err) {
    console.error('Failed to check client status:', err);
    clientConnected.value = false;
    currentSummonerName.value = '';
  } finally {
    isChecking.value = false;
  }
};

const setPageSize = (size: number) => {
  preferences.value.defaultPageSize = size;
  savePreferences();
};

const setBroadcastFormat = (fmt: 'concise' | 'detailed') => {
  preferences.value.broadcastFormat = fmt;
  savePreferences();
};

const savePreferences = () => {
  try {
    const raw = JSON.stringify(preferences.value);
    localStorage.setItem('hexbrain_preferences', raw);
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('hexbrain-preferences-updated', { detail: preferences.value }));
      window.dispatchEvent(new StorageEvent('storage', { key: 'hexbrain_preferences', newValue: raw }));
    }
    statusText.value = '偏好设置已更新';
    if (statusTimer) {
      clearTimeout(statusTimer);
    }
    statusTimer = setTimeout(() => {
      statusText.value = '';
      statusTimer = null;
    }, 2000);
  } catch (err) {
    console.error('Failed to save preferences:', err);
  }
};

const saveAutomation = async () => {
  try {
    automation.value.auto_accept_delay_seconds = Math.max(0, Math.min(10, Math.round(Number(automation.value.auto_accept_delay_seconds) || 0)));
    automation.value.bench_preferred_champions = Array.from(new Set((automation.value.bench_preferred_champions || []).map(String).filter(Boolean)));
    await updateAutomationConfig(automation.value);
    statusText.value = '自动化策略已同步至 Rust 核心引擎';
    if (statusTimer) {
      clearTimeout(statusTimer);
    }
    statusTimer = setTimeout(() => {
      statusText.value = '';
      statusTimer = null;
    }, 2000);
  } catch (err) {
    console.error('Failed to save automation config:', err);
  }
};

onMounted(async () => {
  try {
    const saved = localStorage.getItem('hexbrain_preferences');
    if (saved) {
      const parsed = JSON.parse(saved);
      preferences.value = {
        defaultPageSize: [10, 20, 50, 100].includes(parsed.defaultPageSize) ? parsed.defaultPageSize : 20,
        broadcastFormat: parsed.broadcastFormat === 'detailed' ? 'detailed' : 'concise'
      };
    }
  } catch (err) {
    console.error('Failed to load preferences:', err);
  }

  try {
    const autoCfg = await getAutomationConfig();
    if (autoCfg) {
      automation.value = autoCfg;
    }
  } catch (err) {
    console.error('Failed to load automation config:', err);
  }

  checkClientStatus();
});

onUnmounted(() => {
  if (statusTimer) {
    clearTimeout(statusTimer);
    statusTimer = null;
  }
});
</script>
