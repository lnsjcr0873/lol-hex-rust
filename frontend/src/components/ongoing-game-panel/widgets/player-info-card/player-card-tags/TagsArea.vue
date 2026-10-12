<template>
  <div class="mb-1.5 flex flex-wrap gap-1 select-none items-center min-h-[22px]">
    <template v-for="tag in tags" :key="tag.id">
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
            <span>{{ getTagEmoji(tag.id) }}</span>
            <span>{{ tag.label }}</span>
          </span>
        </template>

        <!-- Popover Content for Tactical Tag -->
        <div class="max-w-56 text-xs text-slate-200 p-1">
          <div class="font-bold text-slate-100 flex items-center gap-1.5 mb-1">
            <span>{{ getTagEmoji(tag.id) }}</span>
            <span>{{ tag.label }}</span>
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
            前 15 分钟敌方打野参与阵亡: {{ tag.popoverData.times?.toFixed(1) }} 次/场
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
        <span>{{ getTagEmoji(tag.id) }}</span>
        <span>{{ tag.label }}</span>
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
import { useOngoingGamePanel } from '../../../context'

const props = defineProps<{
  player: EnhancedPlayerCard
}>()

const { selfPuuid } = useOngoingGamePanel()

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

const getTagEmoji = (id: string) => tagEmojiMap[id] || '🏷️'

const hasPopover = (tag: EvaluatedTag) => {
  return Boolean(tag.description || tag.popoverData)
}

const tags = computed<EvaluatedTag[]>(() => {
  const analysis = adaptPlayerCardToAggregatedAnalysis(props.player)
  const ctx: TacticalTagContext = {
    puuid: props.player.puuid,
    selfPuuid: selfPuuid.value,
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
      if (!result.some((r) => r.label.includes(badge.label) || badge.label.includes(r.label))) {
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

  // Cap to top 6 tags to keep card height clean and readable
  return result.slice(0, 6)
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
