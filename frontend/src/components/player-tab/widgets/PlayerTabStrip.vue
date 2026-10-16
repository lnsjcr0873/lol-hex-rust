<template>
  <div class="h-10 px-3 bg-[#070b14]/90 border-b border-[#162238] flex items-center justify-between select-none shrink-0 relative z-30">
    <!-- Context Menu -->
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :show="contextMenu.show"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :options="contextMenuOptions"
      size="small"
      @clickoutside="contextMenu.show = false"
      @select="handleContextMenuSelect"
    />

    <!-- Left: Search / Home Entry Button + Tabs Strip -->
    <div class="flex items-center space-x-1.5 min-w-0 flex-1 overflow-hidden h-full">
      <!-- Search / Home Button -->
      <button
        @click="$emit('openSearch')"
        class="h-7 px-2.5 rounded-lg flex items-center space-x-1.5 text-xs font-semibold shrink-0 transition"
        :class="activeTabId === null ? 'bg-cyan-950/80 text-cyan-300 border border-cyan-500/50 shadow-sm' : 'bg-slate-900/60 hover:bg-slate-800 text-slate-400 hover:text-slate-200 border border-slate-800/80'"
        title="战绩搜索首页"
      >
        <Compass class="w-3.5 h-3.5" :class="{ 'text-cyan-400': activeTabId === null }" />
        <span class="text-[11px]">探索</span>
      </button>

      <div class="h-4 w-px bg-slate-800 shrink-0 mx-1"></div>

      <!-- Scrollable Tabs List -->
      <div
        ref="tabContainer"
        class="flex items-center space-x-1 overflow-x-auto no-scrollbar py-1 h-full min-w-0 flex-1"
        @wheel.prevent="handleWheelScroll"
      >
        <div
          v-for="tab in tabs"
          :key="tab.id"
          class="group relative h-7 max-w-[180px] min-w-[110px] px-2 rounded-lg flex items-center justify-between space-x-1.5 cursor-pointer text-xs transition border select-none shrink-0"
          :class="activeTabId === tab.id ? 'bg-slate-800 border-cyan-500/50 text-cyan-200 font-semibold shadow-sm' : 'bg-slate-900/50 hover:bg-slate-800/60 border-slate-800/80 text-slate-400 hover:text-slate-200'"
          @click="$emit('activate', tab.id)"
          @contextmenu="handleContextMenu($event, tab.id)"
        >
          <!-- Profile Icon or Spinner -->
          <div class="flex items-center space-x-1.5 min-w-0 flex-1">
            <div class="relative shrink-0">
              <img
                :src="getProfileIconUrl(tab.profileIconId || 1)"
                class="w-4 h-4 rounded-full border border-slate-700 object-cover"
                @error="(e: any) => { e.target.src = 'https://ddragon.leagueoflegends.com/cdn/14.24.1/img/profileicon/1.png' }"
              />
              <Loader2
                v-if="tab.isLoading"
                class="w-4 h-4 absolute inset-0 text-cyan-400 animate-spin bg-slate-900/80 rounded-full"
              />
            </div>

            <!-- Title -->
            <span class="truncate text-[11px]">
              {{ tab.gameName || '召唤师' }}
            </span>
            <span v-if="tab.tagLine" class="text-[9px] text-slate-500 font-mono shrink-0">
              #{{ tab.tagLine }}
            </span>
          </div>

          <!-- Close Button (X) -->
          <button
            @click.stop="$emit('close', tab.id)"
            class="p-0.5 rounded hover:bg-slate-700/80 text-slate-500 hover:text-slate-200 shrink-0 transition"
            title="关闭标签页"
          >
            <X class="w-3 h-3" />
          </button>
        </div>
      </div>
    </div>

    <!-- Right Side Actions Slot -->
    <div class="flex items-center space-x-2 shrink-0 ml-3">
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, h } from 'vue'
import { Compass, X, Loader2, RotateCw, XCircle, ArrowRight } from 'lucide-vue-next'
import { NDropdown } from 'naive-ui'
import type { DropdownMixedOption } from 'naive-ui/es/dropdown/src/interface'
import type { PlayerTabItem } from '../../../stores/player-tabs'
import { getProfileIconUrl } from '../../match-card/utils/resources'

const props = defineProps<{
  tabs: PlayerTabItem[]
  activeTabId: string | null
}>()

const emit = defineEmits<{
  activate: [id: string]
  close: [id: string]
  refresh: [id: string]
  closeOthers: [id: string]
  closeToRight: [id: string]
  openSearch: []
}>()

const tabContainer = ref<HTMLElement | null>(null)

function handleWheelScroll(e: WheelEvent) {
  if (tabContainer.value) {
    tabContainer.value.scrollLeft += e.deltaY
  }
}

const contextMenu = reactive({
  show: false,
  x: 0,
  y: 0,
  tabId: ''
})

const currentTargetIndex = computed(() => {
  return props.tabs.findIndex((t) => t.id === contextMenu.tabId)
})

const contextMenuOptions = computed<DropdownMixedOption[]>(() => [
  {
    label: '刷新战绩',
    key: 'refresh',
    icon: () => h(RotateCw, { class: 'w-3.5 h-3.5 text-cyan-400' })
  },
  {
    type: 'divider',
    key: 'divider-1'
  },
  {
    label: '关闭标签页',
    key: 'close',
    icon: () => h(X, { class: 'w-3.5 h-3.5 text-rose-400' })
  },
  {
    label: '关闭其他标签页',
    key: 'close-others',
    disabled: props.tabs.length <= 1,
    icon: () => h(XCircle, { class: 'w-3.5 h-3.5 text-slate-400' })
  },
  {
    label: '关闭右侧所有标签页',
    key: 'close-to-right',
    disabled: currentTargetIndex.value < 0 || currentTargetIndex.value >= props.tabs.length - 1,
    icon: () => h(ArrowRight, { class: 'w-3.5 h-3.5 text-slate-400' })
  }
])

function handleContextMenu(e: MouseEvent, tabId: string) {
  e.preventDefault()
  contextMenu.show = false
  contextMenu.x = e.clientX
  contextMenu.y = e.clientY
  contextMenu.tabId = tabId
  setTimeout(() => {
    contextMenu.show = true
  }, 10)
}

function handleContextMenuSelect(key: string) {
  contextMenu.show = false
  const targetId = contextMenu.tabId
  if (!targetId) return

  switch (key) {
    case 'refresh':
      emit('refresh', targetId)
      break
    case 'close':
      emit('close', targetId)
      break
    case 'close-others':
      emit('closeOthers', targetId)
      break
    case 'close-to-right':
      emit('closeToRight', targetId)
      break
  }
}
</script>

<style scoped>
.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
