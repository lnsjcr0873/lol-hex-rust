<template>
  <div class="mb-1.5 flex flex-wrap gap-1 select-none items-center min-h-[22px]">
    <template v-for="tag in tags" :key="`${tag.id}-${tag.label}`">
      <NPopover
        v-if="hasPopover(tag)"
        :delay="50"
        :keep-alive-on-hover="true"
        placement="top"
      >
        <template #trigger>
          <span
            class="px-1.5 py-0.5 rounded text-[10px] font-medium leading-none cursor-pointer transition-transform hover:scale-105 inline-flex items-center gap-1 shadow-sm"
            :class="tag.tagClass"
          >
            <span>{{ getTagEmoji(tag) }}</span>
            <span>{{ cleanTagLabel(tag.label) }}</span>
          </span>
        </template>

        <!-- Popover Content for Tactical Tag -->
        <div class="max-w-56 text-xs text-slate-200 p-1">
          <div class="font-bold text-slate-100 flex items-center gap-1.5 mb-1">
            <span>{{ getTagEmoji(tag) }}</span>
            <span>{{ cleanTagLabel(tag.label) }}</span>
          </div>
          <p class="text-slate-300 leading-relaxed text-[11px]">
            {{ tag.description || '战术画像判定指标' }}
          </p>

          <!-- Specific Data Breakdowns -->
          <div v-if="tag.id === 'suspicious-flash-position' && tag.popoverData" class="mt-2 pt-1.5 border-t border-slate-700/60 text-[10px] flex justify-between">
            <span class="text-amber-400">D 闪现: {{ tag.popoverData.flashOnD }} 场</span>
            <span class="text-blue-400">F 闪现: {{ tag.popoverData.flashOnF }} 场</span>
          </div>

          <div v-else-if="tag.id === 'easy-gank' && tag.popoverData" class="mt-2 pt-1.5 border-t border-slate-700/60 text-[10px] text-slate-300">
            前 15 分钟敌方打野参与阵亡: {{ Number.isFinite(tag.popoverData.times) ? tag.popoverData.times.toFixed(1) : '0.0' }} 次/场
          </div>

          <div v-else-if="tag.id === 'high-win-rate' && tag.popoverData" class="mt-2 pt-1.5 border-t border-slate-700/60 text-[10px] text-slate-300 font-mono">
            {{ tag.popoverData.wins }} 胜 / {{ tag.popoverData.count }} 场
          </div>
        </div>
      </NPopover>

      <span
        v-else
        class="px-1.5 py-0.5 rounded text-[10px] font-medium leading-none inline-flex items-center gap-1 shadow-sm"
        :class="tag.tagClass"
      >
        <span>{{ getTagEmoji(tag) }}</span>
        <span>{{ cleanTagLabel(tag.label) }}</span>
      </span>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NPopover } from 'naive-ui'
import type { EnhancedPlayerCard } from '../../../types'
import { evaluateTacticalTags, type EvaluatedTag, type TacticalTagContext } from '../../../../../shared/data-adapter/tactical-tags'
import { adaptPlayerCardToAggregatedAnalysis } from '../../../utils/player-adapter'
import { useOngoingGamePanelOptional } from '../../../context'

const props = defineProps<{
  player: EnhancedPlayerCard
  selfPuuid?: string
}>()

const ongoingContext = useOngoingGamePanelOptional()
const resolvedSelfPuuid = computed(() => {
  if (props.selfPuuid) return props.selfPuuid
  if (ongoingContext?.selfPuuid?.value) return ongoingContext.selfPuuid.value
  return ''
})

const tagEmojiMap: Record<string, string> = {
  'self': '👤',
  'tagged': '📝',
  'premade-team': '🚗',
  'high-win-rate': '👑',
  'privacy': '🔒',
  'met': '🤝',
  'winning-streak': '🔥',
  'losing-streak': '❄️',
  'great-performance': '⚡',
  'suspicious-flash-position': '⚠️',
  'easy-gank': '🎯',
  'solo-kills': '⚔️',
  'average-team-damage': '💥',
  'average-team-damage-taken': '🛡️',
  'average-team-gold': '💰',
  'average-cs-per-minute': '🌾',
  'average-damage-gold-efficiency': '📈',
  'average-enemy-missing-pings': '❓',
  'average-vision-score': '👁️',
  'average-kill-damage-efficiency': '🔪',
  'akari-score': '🌟'
}

const cleanTagLabel = (label: string) => {
  return label.replace(/^[\p{Emoji}\u200d\uFE0F\s]+/u, '').trim()
}

const getTagEmoji = (tag: EvaluatedTag) => {
  if (tagEmojiMap[tag.id]) {
    return tagEmojiMap[tag.id]
  }
  const match = tag.label.match(/^([\p{Emoji}\u200d\uFE0F]+)/u)
  if (match) {
    return match[1]
  }
  if (tag.label.includes('连胜')) return '🔥'
  if (tag.label.includes('连败')) return '❄️'
  if (tag.label.includes('绝活')) return '👑'
  if (tag.label.includes('通天代')) return '⚡'
  if (tag.label.includes('车队')) return '🚗'
  if (tag.label.includes('试英雄')) return '⚠️'
  return '🏷️'
}

const hasPopover = (tag: EvaluatedTag) => {
  return Boolean(tag.description || tag.popoverData)
}

const tags = computed<EvaluatedTag[]>(() => {
  if (!props.player) return []
  const analysis = adaptPlayerCardToAggregatedAnalysis(props.player)
  const ctx: TacticalTagContext = {
    puuid: props.player.puuid,
    selfPuuid: resolvedSelfPuuid.value,
    analysis,
    summoner: {
      gameName: props.player.game_name,
      tagLine: props.player.tag_line
    },
    savedInfo: props.player.savedInfo,
    premadeTeamId: props.player.premadeTeamId,
    assignedPosition: props.player.assigned_position,
    spells: {
      spell1Id: props.player.spell1_id,
      spell2Id: props.player.spell2_id
    }
  }

  const evaluated = evaluateTacticalTags(ctx)

  // Merge any extra badges on the player if not duplicate
  const result: EvaluatedTag[] = [...evaluated]
  if (props.player.badges && props.player.badges.length > 0) {
    for (const badge of props.player.badges) {
      const cleanBadge = cleanTagLabel(badge.label)
      const isDuplicate = result.some((r) => {
        if (r.id === badge.id) return true
        if (r.id === 'tagged' || badge.id === 'tagged') return false
        const cleanR = cleanTagLabel(r.label)
        return cleanR === cleanBadge
      })
      if (!isDuplicate) {
        result.push({
          id: badge.id,
          label: badge.label,
          tagClass: getBadgeColorClass(badge.color),
          description: badge.description,
          priority: 40
        })
      }
    }
  }

  // Mutual exclusion filtering
  const winStreakVal = analysis.winLoss?.all?.winningStreak ?? 0
  const lossStreakVal = analysis.winLoss?.all?.losingStreak ?? 0
  const hasWinStreak = result.some((t) => t.id === 'winning-streak' || t.label.includes('连胜'))
  const hasLossStreak = result.some((t) => t.id === 'losing-streak' || t.label.includes('连败'))
  const hasSmurf = result.some((t) => t.id === 'great-performance' || t.label.includes('通天代'))
  const hasTestingChamp = result.some((t) => t.label.includes('试英雄'))
  const winRate = props.player.recent_win_rate ?? 50

  let filtered = result.filter((t) => {
    // If both streak tags exist, strictly compare winningStreak vs losingStreak counts
    if (hasWinStreak && hasLossStreak) {
      if (winStreakVal > lossStreakVal) {
        if (t.id === 'losing-streak' || t.label.includes('连败')) return false
      } else if (lossStreakVal > winStreakVal) {
        if (t.id === 'winning-streak' || t.label.includes('连胜')) return false
      } else {
        if (winRate < 50 && (t.id === 'winning-streak' || t.label.includes('连胜'))) return false
        if (winRate >= 50 && (t.id === 'losing-streak' || t.label.includes('连败'))) return false
      }
    }
    // If smurf and testing-champ collide, testing-champ invalidates smurf
    if (hasSmurf && hasTestingChamp) {
      if (t.id === 'great-performance' || t.label.includes('通天代')) return false
    }
    // Low winrate cannot be a smurf
    if (winRate < 55 && (t.id === 'great-performance' || t.label.includes('通天代'))) {
      return false
    }
    return true
  })

  // Deduplicate tags: combine tag.id with cleanTagLabel to avoid collisions between distinct categories (e.g. custom note vs premade team)
  const seenKeys = new Set<string>()
  filtered = filtered.filter((t) => {
    const key = `${t.id}-${cleanTagLabel(t.label)}`
    if (seenKeys.has(key)) return false
    seenKeys.add(key)
    return true
  })

  // Cap to top 6 tags to keep card height clean and readable
  return filtered.slice(0, 6)
})

function getBadgeColorClass(color?: string): string {
  switch (color) {
    case 'gold': return 'bg-amber-950/90 text-amber-300 border border-amber-600/50'
    case 'cyan': return 'bg-cyan-950/90 text-cyan-300 border border-cyan-600/50'
    case 'red': return 'bg-rose-950/90 text-rose-300 border border-rose-600/50'
    case 'blue': return 'bg-blue-950/90 text-blue-300 border border-blue-600/50'
    case 'purple': return 'bg-purple-950/90 text-purple-300 border border-purple-600/50'
    case 'green': return 'bg-emerald-950/90 text-emerald-300 border border-emerald-600/50'
    default: return 'bg-slate-800 text-slate-200 border border-slate-700'
  }
}
</script>
