<template>
  <div class="h-full w-full flex flex-col items-center justify-center p-6 bg-[#090d1a] overflow-y-auto select-none">
    <div class="max-w-xl w-full flex flex-col items-center text-center space-y-7 my-auto py-8">
      <!-- App Brand Logo & Title -->
      <div class="flex flex-col items-center space-y-3">
        <div class="w-20 h-20 rounded-3xl bg-gradient-to-br from-cyan-500/20 via-blue-600/10 to-indigo-500/20 border border-cyan-500/40 flex items-center justify-center shadow-lg shadow-cyan-950/40 relative group">
          <Sparkles class="w-10 h-10 text-cyan-400 group-hover:scale-110 transition duration-300" />
          <div class="absolute -bottom-1 -right-1 px-2 py-0.5 rounded-full bg-cyan-950 border border-cyan-500 text-[9px] font-bold text-cyan-300 font-mono">
            v2.0
          </div>
        </div>

        <div class="space-y-1">
          <h1 class="text-2xl font-black text-slate-100 tracking-wide flex items-center justify-center gap-2">
            <span>HexBrain 战绩探索中心</span>
          </h1>
          <p class="text-xs text-slate-400 max-w-md">
            全维度英雄联盟历史对局复盘 · 多标签秒级对比 · SGP/LCU 无损原生数据
          </p>
        </div>
      </div>

      <!-- Quick Action: Current Connected Summoner or Demo Trigger -->
      <div class="w-full">
        <!-- If connected: Quick Self Profile Card -->
        <div
          v-if="currentProfile && currentProfile.is_connected"
          @click="handleOpenSelf"
          class="p-3.5 px-5 rounded-2xl bg-slate-900/90 border border-cyan-500/30 hover:border-cyan-400/60 transition cursor-pointer flex items-center justify-between shadow-md hover:shadow-cyan-950/40 group active:scale-[0.99]"
        >
          <div class="flex items-center space-x-3.5">
            <div class="relative shrink-0">
              <img
                :src="getProfileIconUrl(currentProfile.profile_icon_id)"
                class="w-12 h-12 rounded-xl border border-cyan-500/50 object-cover"
                @error="(e: any) => { e.target.src = 'https://ddragon.leagueoflegends.com/cdn/14.24.1/img/profileicon/1.png' }"
              />
              <span class="absolute -bottom-1 -right-1 px-1 bg-slate-950 border border-cyan-500/60 text-[9px] text-cyan-300 font-mono rounded">
                Lv.{{ currentProfile.summoner_level }}
              </span>
            </div>
            <div class="text-left">
              <div class="flex items-center space-x-1.5">
                <span class="font-bold text-slate-100 text-sm group-hover:text-cyan-300 transition">
                  {{ currentProfile.game_name }}
                </span>
                <span v-if="currentProfile.tag_line" class="text-xs text-slate-400 font-mono">
                  #{{ currentProfile.tag_line }}
                </span>
                <span class="px-1.5 py-0.2 rounded text-[10px] font-semibold bg-emerald-950 border border-emerald-500/50 text-emerald-300">
                  当前登录
                </span>
              </div>
              <div class="text-xs text-slate-400 mt-0.5">
                单双排位: <span class="font-bold text-slate-200">{{ currentProfile.tier_solo || '未定级' }} {{ currentProfile.division_solo || '' }}</span> · {{ currentProfile.lp_solo ?? 0 }} LP
              </div>
            </div>
          </div>

          <div class="flex items-center space-x-1 text-cyan-400 text-xs font-semibold group-hover:translate-x-1 transition">
            <span>查看我的战绩</span>
            <ChevronRight class="w-4 h-4" />
          </div>
        </div>

        <!-- If disconnected: Offline Alert & Demo Mode Trigger -->
        <div
          v-else
          class="p-3.5 px-5 rounded-2xl bg-slate-900/70 border border-slate-800 flex items-center justify-between text-left"
        >
          <div class="flex items-center space-x-3">
            <div class="w-10 h-10 rounded-xl bg-amber-950/60 border border-amber-600/40 flex items-center justify-center text-amber-400 shrink-0">
              <Gamepad2 class="w-5 h-5" />
            </div>
            <div>
              <div class="text-xs font-bold text-slate-200">客户端未连接</div>
              <div class="text-[11px] text-slate-400">请启动英雄联盟客户端，或使用模拟演练模式体验全部功能</div>
            </div>
          </div>

          <div class="flex items-center space-x-2 shrink-0">
            <button
              @click="emit('refreshConnection')"
              class="px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 text-xs font-semibold transition active:scale-95 flex items-center space-x-1.5"
            >
              <RotateCw class="w-3.5 h-3.5" />
              <span>检测连接</span>
            </button>
            <button
              @click="handleOpenDemo"
              class="px-3 py-1.5 rounded-xl bg-amber-950/80 hover:bg-amber-900 border border-amber-600/60 text-amber-200 text-xs font-semibold transition active:scale-95 shrink-0"
            >
              开启演练体验
            </button>
          </div>
        </div>
      </div>

      <!-- Summoner Search Input -->
      <div class="w-full space-y-2">
        <div class="flex items-center space-x-2">
          <div class="relative flex-1">
            <Search class="w-4 h-4 text-slate-400 absolute left-3.5 top-1/2 -translate-y-1/2 pointer-events-none" />
            <input
              v-model="searchQuery"
              @keydown.enter="handleSearch"
              type="text"
              placeholder="输入 召唤师名称 或 游戏名#TAG (如: TheShy#KR1)..."
              class="w-full pl-10 pr-10 py-2.5 rounded-xl bg-slate-900/90 border border-slate-700/80 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500 text-slate-200 placeholder-slate-500 text-xs transition outline-none"
            />
            <button
              v-if="searchQuery"
              @click="searchQuery = ''"
              class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>

          <button
            @click="handleSearch"
            :disabled="isSearching || !searchQuery.trim()"
            class="px-5 py-2.5 rounded-xl bg-cyan-600 hover:bg-cyan-500 disabled:bg-slate-800 disabled:text-slate-600 disabled:border-slate-800 text-white border border-cyan-400 text-xs font-bold transition flex items-center space-x-1.5 shadow-md active:scale-95 shrink-0"
          >
            <Loader2 v-if="isSearching" class="w-3.5 h-3.5 animate-spin" />
            <span>{{ isSearching ? '查询中...' : '搜索' }}</span>
          </button>
        </div>

        <div v-if="searchError" class="text-xs text-rose-400 text-left pl-1">
          {{ searchError }}
        </div>
      </div>

      <!-- Recent Search History -->
      <div class="w-full space-y-3 pt-2">
        <div class="flex items-center justify-between border-b border-slate-800 pb-2">
          <span class="text-xs font-bold text-slate-300 flex items-center space-x-1.5">
            <History class="w-3.5 h-3.5 text-slate-400" />
            <span>近期搜索历史</span>
          </span>
          <button
            v-if="searchHistory.length > 0"
            @click="playerTabsStore.clearSearchHistory"
            class="text-[11px] text-slate-500 hover:text-slate-300 transition"
          >
            清空历史
          </button>
        </div>

        <!-- History Items Grid -->
        <div v-if="searchHistory.length > 0" class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-left">
          <div
            v-for="item in searchHistory"
            :key="item.id"
            @click="handleOpenHistoryItem(item)"
            class="p-2.5 px-3 rounded-xl bg-slate-900/60 hover:bg-slate-800/80 border border-slate-800/80 hover:border-slate-700 cursor-pointer flex items-center justify-between group transition"
          >
            <div class="flex items-center space-x-2.5 min-w-0">
              <img
                :src="getProfileIconUrl(item.profileIconId || 1)"
                class="w-7 h-7 rounded-lg border border-slate-700 object-cover shrink-0"
                @error="(e: any) => { e.target.src = 'https://ddragon.leagueoflegends.com/cdn/14.24.1/img/profileicon/1.png' }"
              />
              <div class="min-w-0">
                <div class="text-xs font-bold text-slate-200 group-hover:text-cyan-300 truncate transition">
                  {{ item.gameName }}
                </div>
                <div class="text-[10px] text-slate-400 font-mono truncate">
                  #{{ item.tagLine || 'CN' }}
                </div>
              </div>
            </div>

            <button
              @click.stop="playerTabsStore.removeSearchHistory(item.id)"
              class="p-1 rounded hover:bg-slate-700 text-slate-500 hover:text-slate-200 opacity-0 group-hover:opacity-100 transition shrink-0"
              title="删除记录"
            >
              <X class="w-3 h-3" />
            </button>
          </div>
        </div>

        <!-- Empty history placeholder -->
        <div v-else class="py-6 text-center text-xs text-slate-500">
          暂无搜索历史记录，输入召唤师名称开启探索
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Sparkles, Gamepad2, Search, X, Loader2, History, ChevronRight, RotateCw } from 'lucide-vue-next'
import { usePlayerTabsStore, type SearchHistoryItem } from '../../stores/player-tabs'
import { searchSummoner, getSummonerProfile, getMockSummonerProfile } from '../../api/tauri'
import { getProfileIconUrl } from '../match-card/utils/resources'
import type { SummonerProfile } from '../../api/types'

const props = defineProps<{
  currentProfile?: SummonerProfile | null
}>()

const emit = defineEmits<{
  openSummoner: [payload: { puuid: string; gameName?: string; tagLine?: string; profileIconId?: number; isMock?: boolean }]
  refreshConnection: []
}>()

const playerTabsStore = usePlayerTabsStore()
const searchHistory = computed(() => playerTabsStore.searchHistory)

const searchQuery = ref('')
const isSearching = ref(false)
const searchError = ref('')

async function handleSearch() {
  const query = searchQuery.value.trim()
  if (!query) return

  isSearching.value = true
  searchError.value = ''

  try {
    const res = await searchSummoner(query)
    if (res && res.puuid) {
      const gameName = res.gameName || res.displayName || query
      const tagLine = res.tagLine || ''
      const profileIconId = res.profileIconId || 1

      playerTabsStore.addSearchHistory({
        gameName,
        tagLine,
        puuid: res.puuid,
        profileIconId
      })

      emit('openSummoner', {
        puuid: res.puuid,
        gameName,
        tagLine,
        profileIconId,
        isMock: false
      })
      searchQuery.value = ''
    } else {
      // Fallback: search profile directly
      const p = await getSummonerProfile(query)
      if (p && p.puuid) {
        playerTabsStore.addSearchHistory({
          gameName: p.game_name,
          tagLine: p.tag_line,
          puuid: p.puuid,
          profileIconId: p.profile_icon_id
        })
        emit('openSummoner', {
          puuid: p.puuid,
          gameName: p.game_name,
          tagLine: p.tag_line,
          profileIconId: p.profile_icon_id,
          isMock: false
        })
        searchQuery.value = ''
      } else {
        searchError.value = `未查询到名为 "${query}" 的召唤师`
      }
    }
  } catch (err: any) {
    searchError.value = `查询失败: ${err?.message || '未找到该召唤师或客户端离线'}`
  } finally {
    isSearching.value = false
  }
}

function handleOpenSelf() {
  if (props.currentProfile && props.currentProfile.puuid) {
    emit('openSummoner', {
      puuid: props.currentProfile.puuid,
      gameName: props.currentProfile.game_name,
      tagLine: props.currentProfile.tag_line,
      profileIconId: props.currentProfile.profile_icon_id,
      isMock: false
    })
  }
}

function handleOpenDemo() {
  const mock = getMockSummonerProfile()
  emit('openSummoner', {
    puuid: mock.puuid,
    gameName: mock.game_name,
    tagLine: mock.tag_line,
    profileIconId: mock.profile_icon_id,
    isMock: true
  })
}

function handleOpenHistoryItem(item: SearchHistoryItem) {
  emit('openSummoner', {
    puuid: item.puuid,
    gameName: item.gameName,
    tagLine: item.tagLine,
    profileIconId: item.profileIconId,
    isMock: false
  })
}
</script>
