<template>
  <div class="p-4 px-6 rounded-2xl bg-slate-900/80 border border-slate-800 backdrop-blur shadow-xl select-none flex items-center justify-between flex-wrap gap-4">
    <!-- Summoner Identity & Ranked Cards -->
    <div class="flex items-center space-x-5">
      <!-- Avatar with Level Badge -->
      <div class="relative shrink-0">
        <img
          :src="getProfileIconUrl(profile?.profile_icon_id || 1)"
          class="w-16 h-16 rounded-2xl border-2 border-[#c8aa6e] object-cover shadow-lg"
          @error="(e: any) => { e.target.src = 'https://ddragon.leagueoflegends.com/cdn/14.24.1/img/profileicon/1.png'; }"
        />
        <span class="absolute -bottom-1 -right-1 px-1.5 py-0.2 bg-slate-950 border border-[#c8aa6e] text-[10px] text-[#c8aa6e] font-bold rounded-full shadow">
          Lv.{{ profile?.summoner_level != null ? profile.summoner_level : '--' }}
        </span>
      </div>

      <!-- Summoner Name & Status -->
      <div class="flex flex-col space-y-1.5">
        <div class="flex items-center space-x-2">
          <StreamerModeMaskedText :puuid="profile?.puuid">
            <template #masked>
              <h1 class="text-xl font-black text-slate-100 tracking-wide font-mono">
                {{ maskedSummonerName }}
              </h1>
              <span v-if="profile?.tag_line" class="text-xs text-slate-400 font-mono">#{{ maskedTagLine }}</span>
            </template>
            <h1 class="text-xl font-black text-slate-100 tracking-wide">
              {{ profile?.game_name || '召唤师' }}
            </h1>
            <span v-if="profile?.tag_line" class="text-xs text-slate-400 font-mono">#{{ profile.tag_line }}</span>
          </StreamerModeMaskedText>
          <span
            class="px-2 py-0.5 rounded text-[10px] font-semibold"
            :class="profile?.is_connected ? 'bg-emerald-950/80 border border-emerald-600/50 text-emerald-300' : (profile?.is_mock ? 'bg-amber-950/80 border border-amber-600/50 text-amber-300' : 'bg-rose-950/80 border border-rose-600/50 text-rose-300')"
          >
            {{ profile?.is_connected ? '实时连接' : (profile?.is_mock ? '模拟演练' : '未连接客户端') }}
          </span>
        </div>

        <!-- Ranked Cards -->
        <RankedPane :profile="profile" />
      </div>
    </div>

    <!-- Actions: Refresh Button & Tag Edit Button -->
    <div class="flex items-center space-x-2.5">
      <slot name="actions">
        <!-- Tag Edit Popover (for other players) -->
        <NPopover
          v-if="!isSelf && profile?.puuid"
          v-model:show="isTagEditPopoverShowing"
          trigger="click"
          placement="bottom-end"
          class="bg-[#0f172a]! border border-slate-800 rounded-xl shadow-2xl p-0!"
          :show-arrow="false"
        >
          <template #trigger>
            <button
              class="p-2 rounded-xl bg-slate-800/80 hover:bg-slate-700 border border-slate-700 text-slate-300 hover:text-white transition active:scale-95 shadow-md flex items-center space-x-1.5 text-xs font-semibold"
              title="标记备注该召唤师"
            >
              <NIcon size="15" class="text-amber-400"><Edit20Filled /></NIcon>
              <span>备注标记</span>
            </button>
          </template>
          <PlayerTagEditPanel
            :puuid="profile.puuid"
            :self-puuid="selfPuuid"
            :summoner="tagSummoner"
            @cancel="isTagEditPopoverShowing = false"
            @saved="handleTagSaved"
          />
        </NPopover>

        <button
          @click="$emit('refresh')"
          class="p-2 rounded-xl bg-slate-800/80 hover:bg-slate-700 border border-slate-700 text-slate-300 hover:text-white transition active:scale-95 shadow-md flex items-center space-x-1.5 text-xs font-semibold"
          title="刷新并感知最新对局"
        >
          <RotateCw class="w-4 h-4" :class="{ 'animate-spin': loading }" />
          <span>刷新</span>
        </button>
      </slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { RotateCw } from 'lucide-vue-next'
import { Edit20Filled } from '@vicons/fluent'
import { NIcon, NPopover } from 'naive-ui'
import { getProfileIconUrl } from '../match-card/utils/resources'
import RankedPane from './widgets/RankedPane.vue'
import StreamerModeMaskedText from '../widgets/StreamerModeMaskedText.vue'
import { useStreamerModeMaskedText } from '../widgets/useStreamerModeMaskedText'
import PlayerTagEditPanel from '../player-tag-edit/PlayerTagEditPanel.vue'

const props = defineProps<{
  profile?: any
  loading?: boolean
  isSelf?: boolean
  selfPuuid?: string
}>()

const emit = defineEmits<{
  refresh: []
  tagSaved: []
}>()

const { summonerName, masked } = useStreamerModeMaskedText()
const isTagEditPopoverShowing = ref(false)

const maskedSummonerName = computed(() => {
  const seed = props.profile?.game_name || props.profile?.puuid || ''
  return summonerName(seed, 0)
})

const maskedTagLine = computed(() => masked(props.profile?.tag_line || '', '#####'))

const tagSummoner = computed(() => {
  if (!props.profile) return null
  return {
    gameName: props.profile.game_name || '',
    tagLine: props.profile.tag_line || '',
    profileIconId: props.profile.profile_icon_id || 1,
    puuid: props.profile.puuid || ''
  }
})

const handleTagSaved = () => {
  isTagEditPopoverShowing.value = false
  emit('tagSaved')
}
</script>

