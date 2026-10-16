<template>
  <div class="rounded-xl bg-slate-900/80 border border-slate-800 p-3.5 text-xs text-slate-300 select-none shadow-sm" v-if="players.length > 0">
    <div class="mb-2.5 text-sm font-extrabold text-slate-100 flex items-center justify-between">
      <div class="flex items-center space-x-1.5">
        <span 
          class="w-2 h-2 rounded-full" 
          :class="side === 'ally' ? 'bg-blue-400' : 'bg-red-400'"
        ></span>
        <span>{{ side === 'ally' ? '近期同队队友' : '近期宿敌对手' }}</span>
      </div>
      <span class="text-[10px] text-slate-400 font-mono">{{ players.length }} 人</span>
    </div>

    <div class="flex flex-col gap-1.5">
      <div
        v-for="p of players"
        :key="p.targetPuuid"
        class="flex items-center justify-between p-1.5 rounded-lg bg-slate-950/60 border border-slate-800/80 hover:border-slate-700 hover:bg-slate-800/50 transition cursor-pointer group"
        @click="$emit('selectSummoner', p.targetPuuid)"
        :title="`点击查看 ${p.targetGameName} 的对局战绩`"
      >
        <div class="flex items-center space-x-2 min-w-0">
          <img
            class="w-6 h-6 rounded-md object-cover border border-slate-700 shrink-0"
            :src="getProfileIconUrl(p.targetProfileIconId || 1)"
            @error="(e: any) => { e.target.src = 'https://ddragon.leagueoflegends.com/cdn/14.24.1/img/profileicon/532.png'; }"
          />
          <div class="flex items-baseline space-x-1 truncate">
            <span class="truncate text-xs font-semibold text-slate-200 group-hover:text-cyan-400 transition">
              {{ p.targetGameName }}
            </span>
            <span v-if="p.targetTagLine" class="text-[10px] text-slate-400 font-mono shrink-0">
              #{{ p.targetTagLine }}
            </span>
          </div>
        </div>

        <span class="text-[11px] font-mono shrink-0 pl-2">
          <span class="text-emerald-400 font-bold">{{ p.win }}胜</span>
          <span class="text-slate-500 mx-0.5">/</span>
          <span class="text-rose-400 font-bold">{{ p.lose }}负</span>
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { getProfileIconUrl } from '../match-card/utils/resources'

interface EncounteredPlayer {
  targetPuuid: string
  targetGameName: string
  targetTagLine: string
  targetProfileIconId: number
  win: number
  lose: number
  gamesCount?: number
}

const props = withDefaults(
  defineProps<{
    side?: 'ally' | 'enemy'
    playersList?: EncounteredPlayer[]
    matches?: any[]
    selfPuuid?: string
  }>(),
  {
    side: 'ally'
  }
)

defineEmits<{
  selectSummoner: [puuid: string]
}>()

const players = computed<EncounteredPlayer[]>(() => {
  if (props.playersList && props.playersList.length > 0) {
    return props.playersList
  }

  // If matches are passed, compute dynamically from match history
  if (props.matches && props.matches.length > 0) {
    const isAlly = props.side === 'ally'
    const playerStatsMap = new Map<string, {
      puuid: string
      name: string
      tag: string
      icon: number
      wins: number
      losses: number
      count: number
    }>()

    for (const m of props.matches) {
      const participants = m.participants_preview || m.participants || m.raw?.participants || []
      const isWin = !!(m.is_win ?? m.win)
      const myTeam = m.team_id ? (m.team_id === 100 ? 'blue' : 'red') : (m.is_blue !== undefined ? (m.is_blue ? 'blue' : 'red') : (m.team || 'blue'))

      for (const p of participants) {
        const pPuuid = p.puuid || p.player?.puuid || ''
        if (pPuuid && props.selfPuuid && pPuuid === props.selfPuuid) continue

        const pTeam = p.team || (p.team_id ? (p.team_id === 100 ? 'blue' : 'red') : (p.teamId === 100 ? 'blue' : 'red'))
        const isSameTeam = pTeam === myTeam

        if ((isAlly && isSameTeam) || (!isAlly && !isSameTeam)) {
          const key = pPuuid || p.summoner_name || p.gameName || 'unknown'
          const existing = playerStatsMap.get(key) || {
            puuid: pPuuid,
            name: p.game_name || p.summoner_name || p.gameName || '玩家',
            tag: p.tag_line || p.tagLine || '',
            icon: p.profile_icon_id || p.profileIcon || 1,
            wins: 0,
            losses: 0,
            count: 0
          }

          existing.count++
          if (isWin) {
            if (isAlly) existing.wins++
            else existing.losses++
          } else {
            if (isAlly) existing.losses++
            else existing.wins++
          }

          playerStatsMap.set(key, existing)
        }
      }
    }

    const list: EncounteredPlayer[] = []
    for (const item of playerStatsMap.values()) {
      if (item.count >= 2) {
        list.push({
          targetPuuid: item.puuid,
          targetGameName: item.name,
          targetTagLine: item.tag,
          targetProfileIconId: item.icon,
          win: item.wins,
          lose: item.losses,
          gamesCount: item.count
        })
      }
    }

    if (list.length > 0) {
      return list.sort((a, b) => (b.gamesCount || 0) - (a.gamesCount || 0)).slice(0, 5)
    }
  }

  return []
})
</script>
