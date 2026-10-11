<template>
  <div class="flex-1 overflow-y-auto p-6 bg-[#090d1a] select-none space-y-6">
    <div class="border-b border-[#162238] pb-4">
      <h2 class="text-base font-bold text-slate-100 flex items-center space-x-2">
        <span>对局自动化辅助中心</span>
        <span class="text-xs font-normal text-slate-400">· League-Akari 高频功能对齐</span>
      </h2>
      <p class="text-xs text-slate-400 mt-1">
        所有自动化行为均基于本地 LCU 原生事件驱动，无内存注入，安全可靠。
      </p>
    </div>

    <!-- Group 1: Match Auto Accept -->
    <div class="bg-[#0c1424] p-5 rounded-2xl border border-[#162238] space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-sm font-bold text-slate-200">排位 / 匹配自动接受对局</h3>
          <p class="text-xs text-slate-400 mt-0.5">当检测到客户端匹配成功就绪弹窗时，自动向 LCU 下发确认指令</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input 
            type="checkbox" 
            v-model="config.auto_accept_match" 
            @change="saveConfig" 
            class="sr-only peer"
          />
          <div class="w-11 h-6 bg-[#162238] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#0ac8b9]"></div>
        </label>
      </div>

      <!-- Human Delay Slider -->
      <div v-if="config.auto_accept_match" class="pt-3 border-t border-[#141f33] space-y-2">
        <div class="flex items-center justify-between text-xs">
          <span class="text-slate-300 font-medium">拟人化延迟防检测 (Anti-Cheat Delay)</span>
          <span class="text-[#0ac8b9] font-bold font-mono">{{ config.auto_accept_delay_seconds }} 秒延时</span>
        </div>
        <input 
          type="range" 
          min="0" 
          max="5" 
          step="1"
          v-model.number="config.auto_accept_delay_seconds" 
          @change="saveConfig"
          class="w-full h-1.5 bg-[#162238] rounded-lg appearance-none cursor-pointer accent-[#0ac8b9]"
        />
        <div class="flex justify-between text-[10px] text-slate-500 font-mono">
          <span>0s (秒接)</span>
          <span>1s</span>
          <span>2s (推荐)</span>
          <span>3s</span>
          <span>5s (保守)</span>
        </div>
      </div>
    </div>

    <!-- Group 2: Runes & Summoner Spells -->
    <div class="bg-[#0c1424] p-5 rounded-2xl border border-[#162238] space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-sm font-bold text-slate-200">一键 / 自动配置权威胜率符文天赋</h3>
          <p class="text-xs text-slate-400 mt-0.5">锁定英雄后，自动拉取当前版本胜率第一的符文页与召唤师技能并写入客户端</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input 
            type="checkbox" 
            v-model="config.auto_apply_runes" 
            @change="saveConfig" 
            class="sr-only peer"
          />
          <div class="w-11 h-6 bg-[#162238] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#0ac8b9]"></div>
        </label>
      </div>
    </div>

    <!-- Group 3: ARAM Bench Fast Grab -->
    <div class="bg-[#0c1424] p-5 rounded-2xl border border-[#162238] space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-sm font-bold text-slate-200">极地大乱斗长椅英雄极速秒抢</h3>
          <p class="text-xs text-slate-400 mt-0.5">当队友摇出或丢出心愿英雄池中的英雄时，以毫秒级响应自动交换长椅</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input 
            type="checkbox" 
            v-model="config.auto_swap_bench" 
            @change="saveConfig" 
            class="sr-only peer"
          />
          <div class="w-11 h-6 bg-[#162238] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#0ac8b9]"></div>
        </label>
      </div>
    </div>

    <!-- Save Status Toast -->
    <div 
      v-if="statusText" 
      class="text-xs text-[#0ac8b9] flex items-center space-x-1.5 transition-opacity"
    >
      <Check class="w-3.5 h-3.5" />
      <span>{{ statusText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Check } from 'lucide-vue-next';
import { AutomationConfig } from '../api/types';
import { getAutomationConfig, updateAutomationConfig } from '../api/tauri';
import { mockAutomationConfig } from '../api/mockData';

const config = ref<AutomationConfig>(mockAutomationConfig);
const statusText = ref('');

const saveConfig = async () => {
  try {
    await updateAutomationConfig(config.value);
    statusText.value = '配置已自动同步至 Rust 核心引擎';
    setTimeout(() => {
      statusText.value = '';
    }, 2000);
  } catch (err) {
    console.error(err);
  }
};

onMounted(async () => {
  try {
    config.value = await getAutomationConfig();
  } catch (err) {
    console.error(err);
  }
});
</script>
