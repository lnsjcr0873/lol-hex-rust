<template>
  <div class="flex-1 overflow-y-auto p-6 bg-[#090d1a] select-none space-y-6">
    <div class="border-b border-[#162238] pb-4">
      <h2 class="text-base font-bold text-slate-100 flex items-center space-x-2">
        <span>系统参数与局内 HUD 设置</span>
      </h2>
      <p class="text-xs text-slate-400 mt-1">
        调节客户端底层网络参数、全局快捷键与 Win32 原生置顶层特性。
      </p>
    </div>

    <!-- Group 1: Win32 Overlay Controls -->
    <div class="bg-[#0c1424] p-5 rounded-2xl border border-[#162238] space-y-4">
      <h3 class="text-sm font-bold text-slate-200">游戏内置顶 HUD 辅助浮窗</h3>
      
      <div class="grid grid-cols-2 gap-4 text-xs">
        <div class="bg-[#080e1b] p-3 rounded-xl border border-[#152238]">
          <div class="flex items-center justify-between">
            <span class="text-slate-200 font-semibold">海克斯评级 HUD (F1)</span>
            <span class="px-1.5 py-0.5 rounded bg-emerald-950 text-emerald-400 text-[10px] font-mono">已启用</span>
          </div>
          <p class="text-slate-400 text-[11px] mt-1">海克斯强化刷新时自动屏幕识别与 T0/T1 评级浮现</p>
        </div>

        <div class="bg-[#080e1b] p-3 rounded-xl border border-[#152238]">
          <div class="flex items-center justify-between">
            <span class="text-slate-200 font-semibold">动态出装抗性推荐 (F2)</span>
            <span class="px-1.5 py-0.5 rounded bg-emerald-950 text-emerald-400 text-[10px] font-mono">已启用</span>
          </div>
          <p class="text-slate-400 text-[11px] mt-1">根据敌方物法抗性动态推演最优出装路径</p>
        </div>

        <div class="bg-[#080e1b] p-3 rounded-xl border border-[#152238]">
          <div class="flex items-center justify-between">
            <span class="text-slate-200 font-semibold">大乱斗长椅透视 HUD (F3)</span>
            <span class="px-1.5 py-0.5 rounded bg-emerald-950 text-emerald-400 text-[10px] font-mono">已启用</span>
          </div>
          <p class="text-slate-400 text-[11px] mt-1">实时展示长椅英雄池评级与胜率曲线</p>
        </div>

        <div class="bg-[#080e1b] p-3 rounded-xl border border-[#152238]">
          <div class="flex items-center justify-between">
            <span class="text-slate-200 font-semibold">排位对线克制看板 (F4)</span>
            <span class="px-1.5 py-0.5 rounded bg-emerald-950 text-emerald-400 text-[10px] font-mono">已启用</span>
          </div>
          <p class="text-slate-400 text-[11px] mt-1">召唤师峡谷对线克制率与技能连招指南</p>
        </div>
      </div>
    </div>

    <!-- Group 2: System Info & Release Pipeline -->
    <div class="bg-[#0c1424] p-5 rounded-2xl border border-[#162238] space-y-3">
      <h3 class="text-sm font-bold text-slate-200">架构与构建信息</h3>
      <div class="space-y-1.5 text-xs text-slate-400">
        <div class="flex justify-between py-1 border-b border-[#141e30]">
          <span>桌面运行时</span>
          <span class="text-slate-200 font-mono">Tauri v2.1 + Webview2</span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#141e30]">
          <span>底层核心引擎</span>
          <span class="text-slate-200 font-mono">Rust 2021 (x86_64-pc-windows-gnu)</span>
        </div>
        <div class="flex justify-between py-1 border-b border-[#141e30]">
          <span>UI 框架栈</span>
          <span class="text-slate-200 font-mono">Vue 3 + TypeScript + Tailwind CSS</span>
        </div>
        <div class="flex justify-between py-1">
          <span>云端打包流水线</span>
          <span class="text-[#0ac8b9] font-mono">GitHub Actions (Windows-latest runner)</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>
