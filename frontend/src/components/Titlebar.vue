<template>
  <header 
    data-tauri-drag-region
    class="h-9 bg-[#070b14] border-b border-[#162238] flex items-center justify-between px-3 select-none text-xs text-slate-300 z-50 sticky top-0"
  >
    <!-- Left: Brand & Connection Status -->
    <div class="flex items-center space-x-3 pointer-events-none">
      <div class="flex items-center space-x-2">
        <div class="w-5 h-5 rounded bg-gradient-to-tr from-[#0ac8b9] to-[#3b82f6] flex items-center justify-center font-bold text-black text-[10px] shadow-sm">
          HB
        </div>
        <span class="font-bold tracking-wider text-slate-100 text-sm">HexBrain</span>
        <span class="px-1.5 py-0.2 bg-[#0f172a] text-[#0ac8b9] text-[10px] rounded border border-[#1e293b]">Tauri v2</span>
      </div>

      <!-- LCU Connection Status Dot -->
      <div class="flex items-center space-x-1.5 pl-2 border-l border-slate-800">
        <span class="relative flex h-2 w-2">
          <span 
            class="animate-ping absolute inline-flex h-full w-full rounded-full opacity-75"
            :class="lcuStore.isConnected ? 'bg-[#0ac8b9]' : 'bg-amber-500'"
          ></span>
          <span 
            class="relative inline-flex rounded-full h-2 w-2"
            :class="lcuStore.isConnected ? 'bg-[#0ac8b9]' : 'bg-amber-500'"
          ></span>
        </span>
        <span class="text-[11px]" :class="lcuStore.isConnected ? 'text-slate-300' : 'text-amber-400/80'">
          {{ lcuStore.connectionDisplayText }}
        </span>
      </div>
    </div>

    <!-- Center: Gameflow Phase Notification -->
    <div class="hidden sm:flex items-center space-x-2 pointer-events-none">
      <span 
        class="px-2 py-0.5 rounded-full border text-[11px]"
        :class="lcuStore.isConnected ? 'bg-[#0d1829] border-[#1e3a5f] text-[#38bdf8]' : 'bg-slate-900 border-slate-800 text-slate-500'"
      >
        {{ lcuStore.isConnected ? currentPhase : '等待客户端启动' }}
      </span>
    </div>

    <!-- Right: Window Controls -->
    <div class="flex items-center space-x-1 pointer-events-auto">
      <button 
        @click="handleMinimize"
        title="最小化"
        class="w-7 h-6 flex items-center justify-center hover:bg-[#162238] text-slate-400 hover:text-white rounded transition"
      >
        <Minus class="w-3.5 h-3.5" />
      </button>
      <button 
        @click="handleMaximize"
        title="最大化 / 还原"
        class="w-7 h-6 flex items-center justify-center hover:bg-[#162238] text-slate-400 hover:text-white rounded transition"
      >
        <Square class="w-3 h-3" />
      </button>
      <button 
        @click="handleClose"
        title="关闭"
        class="w-7 h-6 flex items-center justify-center hover:bg-red-600/80 text-slate-400 hover:text-white rounded transition"
      >
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { Minus, Square, X } from 'lucide-vue-next';
import { minimizeWindow, toggleMaximizeWindow, closeWindow } from '../api/tauri';
import { useLcuStore } from '../stores/lcu';

const props = defineProps<{
  gameflowPhaseText?: string;
}>();

const lcuStore = useLcuStore();
const currentPhase = computed(() => props.gameflowPhaseText || lcuStore.phaseDisplayText);

onMounted(() => {
  lcuStore.startPolling(3000);
});

onUnmounted(() => {
  lcuStore.stopPolling();
});

const handleMinimize = () => minimizeWindow();
const handleMaximize = () => toggleMaximizeWindow();
const handleClose = () => closeWindow();
</script>
