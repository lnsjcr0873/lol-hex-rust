<template>
  <aside class="w-16 bg-[#060911] border-r border-[#151f33] flex flex-col items-center py-4 select-none justify-between shrink-0">
    <!-- Top Navigation Icons -->
    <div class="flex flex-col items-center space-y-4 w-full">
      <button
        @click="$emit('change-tab', 'ongoing')"
        :class="activeTab === 'ongoing' ? 'bg-[#0f233a] text-[#0ac8b9] border-r-2 border-[#0ac8b9]' : 'text-slate-400 hover:text-slate-200 hover:bg-[#0c1424]'"
        class="w-full py-3 flex flex-col items-center space-y-1 transition group relative"
        title="实时对局 (Ongoing Match)"
      >
        <Swords class="w-5 h-5 transition-transform group-hover:scale-110" />
        <span class="text-[10px] font-medium scale-90">实时对局</span>
      </button>

      <button
        @click="$emit('change-tab', 'history')"
        :class="activeTab === 'history' ? 'bg-[#0f233a] text-[#0ac8b9] border-r-2 border-[#0ac8b9]' : 'text-slate-400 hover:text-slate-200 hover:bg-[#0c1424]'"
        class="w-full py-3 flex flex-col items-center space-y-1 transition group relative"
        title="战绩详情 (Match History)"
      >
        <History class="w-5 h-5 transition-transform group-hover:scale-110" />
        <span class="text-[10px] font-medium scale-90">战绩复盘</span>
      </button>

      <button
        @click="$emit('change-tab', 'automation')"
        :class="activeTab === 'automation' ? 'bg-[#0f233a] text-[#0ac8b9] border-r-2 border-[#0ac8b9]' : 'text-slate-400 hover:text-slate-200 hover:bg-[#0c1424]'"
        class="w-full py-3 flex flex-col items-center space-y-1 transition group relative"
        title="自动化 (Automation)"
      >
        <Bot class="w-5 h-5 transition-transform group-hover:scale-110" />
        <span class="text-[10px] font-medium scale-90">自动化</span>
      </button>

      <button
        @click="$emit('change-tab', 'settings')"
        :class="activeTab === 'settings' ? 'bg-[#0f233a] text-[#0ac8b9] border-r-2 border-[#0ac8b9]' : 'text-slate-400 hover:text-slate-200 hover:bg-[#0c1424]'"
        class="w-full py-3 flex flex-col items-center space-y-1 transition group relative"
        title="设置 (Settings)"
      >
        <Settings class="w-5 h-5 transition-transform group-hover:scale-110" />
        <span class="text-[10px] font-medium scale-90">参数设置</span>
      </button>
    </div>

    <!-- Bottom Actions -->
    <div class="flex flex-col items-center space-y-3 w-full">
      <button
        @click="$emit('refresh')"
        class="p-2 text-slate-400 hover:text-[#0ac8b9] hover:bg-[#0c1424] rounded-lg transition"
        title="刷新对局与战绩数据"
      >
        <RotateCw class="w-4 h-4" />
      </button>
      <div class="w-2 h-2 rounded-full bg-[#0ac8b9]/40" title="在线引擎运行中"></div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { Swords, History, Bot, Settings, RotateCw } from 'lucide-vue-next';

defineProps<{
  activeTab: 'ongoing' | 'history' | 'automation' | 'settings';
}>();

defineEmits<{
  (e: 'change-tab', tab: 'ongoing' | 'history' | 'automation' | 'settings'): void;
  (e: 'refresh'): void;
}>();
</script>
