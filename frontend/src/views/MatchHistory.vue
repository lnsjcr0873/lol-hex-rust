<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-[#090d1a] relative select-none">
    <!-- Top Summoner Career Profile Banner -->
    <div class="p-4 px-6 border-b border-[#162238] bg-[#070b14]/85 backdrop-blur shrink-0 z-10">
      <div class="flex items-center justify-between">
        <!-- Summoner Info & Dual Rank Cards -->
        <div class="flex items-center space-x-5">
          <!-- Avatar with Level -->
          <div class="relative shrink-0">
            <img 
              :src="`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/profile-icons/${profile.profile_icon_id}.jpg`"
              class="w-16 h-16 rounded-2xl border-2 border-[#c8aa6e] object-cover shadow-lg"
              @error="(e: any) => { e.target.src = 'https://ddragon.leagueoflegends.com/cdn/14.24.1/img/profileicon/532.png'; }"
            />
            <span class="absolute -bottom-1 -right-1 px-1.5 py-0.2 bg-[#090d1a] border border-[#c8aa6e] text-[10px] text-[#c8aa6e] font-bold rounded-full shadow">
              Lv.{{ profile.summoner_level }}
            </span>
          </div>

          <!-- Summoner Name & Dual Rank Badges -->
          <div class="flex flex-col space-y-1.5">
            <div class="flex items-center space-x-2">
              <h1 class="text-xl font-extrabold text-slate-100 tracking-wide">{{ profile.game_name }}</h1>
              <span class="text-xs text-slate-400 font-mono">#{{ profile.tag_line }}</span>
              <span 
                class="px-2 py-0.5 rounded text-[10px] font-semibold shadow-sm"
                :class="profile.is_connected ? 'bg-emerald-950/80 border border-emerald-600/50 text-emerald-300' : 'bg-amber-950/80 border border-amber-600/50 text-amber-300'"
              >
                {{ profile.is_connected ? '实时在线' : '离线示例' }}
              </span>
            </div>

            <!-- Dual Rank Cards -->
            <div class="flex items-center space-x-3">
              <!-- Solo / Duo Rank Card -->
              <div class="flex items-center space-x-2 px-2.5 py-1 rounded-lg bg-[#0e182a] border border-[#1e304d] shadow-sm">
                <span class="text-[10px] font-bold px-1 py-0.2 rounded bg-[#162742] text-amber-300">单双</span>
                <span class="text-xs font-black text-[#c8aa6e]">{{ profile.tier_solo }} {{ profile.division_solo }}</span>
                <span class="text-[11px] text-slate-300 font-mono font-medium">{{ profile.lp_solo }} LP</span>
                <span class="text-[10px] font-semibold text-[#0ac8b9]">({{ profile.win_rate_solo }}%)</span>
                <span class="text-[9px] text-slate-400 font-mono">{{ profile.wins_solo }}胜 {{ profile.losses_solo }}负</span>
              </div>

              <!-- Flex Rank Card -->
              <div class="flex items-center space-x-2 px-2.5 py-1 rounded-lg bg-[#0e182a] border border-[#1e304d] shadow-sm">
                <span class="text-[10px] font-bold px-1 py-0.2 rounded bg-[#162742] text-[#38bdf8]">灵活</span>
                <span class="text-xs font-black text-[#38bdf8]">{{ profile.tier_flex }} {{ profile.division_flex }}</span>
                <span class="text-[11px] text-slate-300 font-mono font-medium">{{ profile.lp_flex }} LP</span>
                <span class="text-[10px] font-semibold text-[#38bdf8]">({{ profile.win_rate_flex }}%)</span>
                <span class="text-[9px] text-slate-400 font-mono">{{ profile.wins_flex }}胜 {{ profile.losses_flex }}负</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 20-Game Summary, Top Champions & 6-Axis Radar -->
        <div class="flex items-center space-x-6">
          <div class="flex items-center space-x-4">
            <!-- 20-Game Stat Pill -->
            <div class="text-right flex flex-col justify-center">
              <div class="text-[11px] text-slate-400 font-medium">近 20 场排位表现</div>
              <div class="text-2xl font-black text-[#0ac8b9] leading-tight">
                {{ profile.recent_summary.win_rate }}%
              </div>
              <div class="text-[10px] text-slate-300 font-mono mt-0.5">
                KDA {{ profile.recent_summary.avg_kda }}
              </div>
            </div>

            <!-- Top 3 Signature Champions -->
            <div class="flex items-center space-x-2.5 pl-4 border-l border-[#1a2942]">
              <div 
                v-for="ch in (profile.recent_summary.top_champions || [])" 
                :key="ch.champion_id"
                class="flex flex-col items-center group cursor-pointer"
                :title="`${ch.champion_name} (近${ch.games}场 胜率${ch.win_rate}% KDA ${ch.kda})`"
              >
                <div class="relative">
                  <img 
                    :src="ch.champion_icon || getChampIconFallback(ch.champion_id)" 
                    class="w-9 h-9 rounded-lg border border-[#1e2e4a] group-hover:scale-105 transition-transform object-cover" 
                  />
                  <span class="absolute -bottom-1 -right-1 text-[8px] px-1 bg-[#070b14] rounded border border-[#1e2e4a] text-slate-300 font-mono">
                    {{ ch.games }}场
                  </span>
                </div>
                <span class="text-[10px] text-[#0ac8b9] font-bold mt-1 font-mono">{{ ch.win_rate }}%</span>
              </div>
            </div>
          </div>

          <!-- 6-Axis SVG Radar Chart -->
          <div class="w-24 h-24 flex items-center justify-center pl-2 border-l border-[#1a2942]">
            <RadarChart :stats="profile.recent_radar" :size="84" />
          </div>
        </div>
      </div>
    </div>

    <!-- Dual Filters Bar: Queue Filter & Lane Filter -->
    <div class="h-11 px-5 border-b border-[#162238] flex items-center justify-between bg-[#070b14]/60 backdrop-blur shrink-0">
      <div class="flex items-center space-x-4">
        <!-- Queue Mode Filter -->
        <div class="flex items-center space-x-1.5">
          <span class="text-[11px] text-slate-400 font-medium">模式:</span>
          <button
            v-for="tab in queueFilterTabs"
            :key="tab.id"
            @click="activeQueueFilter = tab.id"
            :class="activeQueueFilter === tab.id ? 'bg-[#0f233a] text-[#0ac8b9] border-[#0ac8b9]' : 'text-slate-400 hover:text-slate-200 border-transparent'"
            class="px-2.5 py-1 rounded text-xs font-semibold border transition"
          >
            {{ tab.name }}
          </button>
        </div>

        <div class="h-3.5 w-px bg-[#192840]"></div>

        <!-- Lane Role Filter -->
        <div class="flex items-center space-x-1.5">
          <span class="text-[11px] text-slate-400 font-medium">分路:</span>
          <button
            v-for="lane in laneFilterTabs"
            :key="lane.id"
            @click="activeLaneFilter = lane.id"
            :class="activeLaneFilter === lane.id ? 'bg-[#1b2b48] text-amber-300 border-amber-500/60 font-bold' : 'text-slate-400 hover:text-slate-200 border-transparent'"
            class="px-2 py-0.8 rounded text-xs font-medium border flex items-center space-x-1 transition"
          >
            <span class="text-[11px]">{{ lane.icon }}</span>
            <span>{{ lane.name }}</span>
          </button>
        </div>
      </div>

      <div class="flex items-center space-x-3 text-[11px] text-slate-400">
        <span>共 {{ filteredMatches.length }} 场对局</span>
        <span>· 点击左侧比赛查看 10 人深度复盘</span>
      </div>
    </div>

    <!-- Dual-Pane View: Left Matches List, Right 10-Player Scoreboard -->
    <div class="flex-1 overflow-hidden flex">
      <!-- Left: Matches Scroll Flow -->
      <div class="w-96 border-r border-[#162238] overflow-y-auto p-3 space-y-2.5 shrink-0">
        <div
          v-if="filteredMatches.length === 0"
          class="p-8 text-center text-xs text-slate-500 rounded-xl bg-[#0a0f1d] border border-[#162238]"
        >
          暂无匹配筛选条件的对局记录
        </div>

        <div
          v-for="m in filteredMatches"
          :key="m.game_id"
          @click="selectMatch(m.game_id)"
          class="p-3 rounded-xl border transition-all cursor-pointer relative group"
          :class="[
            m.is_win ? 'bg-[#091526]/85 hover:bg-[#0e213b] border-[#1d3557]' : 'bg-[#1a0c10]/85 hover:bg-[#261218] border-[#441a22]',
            selectedGameId === m.game_id ? 'ring-1 ring-[#0ac8b9]' : ''
          ]"
        >
          <!-- Match Header: Result, Queue, Duration, MVP/SVP -->
          <div class="flex items-center justify-between text-xs mb-2">
            <div class="flex items-center space-x-2">
              <span class="font-bold text-xs" :class="m.is_win ? 'text-blue-400' : 'text-red-400'">
                {{ m.is_win ? '胜利' : '失败' }}
              </span>
              <span class="text-slate-400 text-[11px]">{{ m.queue_name }}</span>
            </div>
            <div class="flex items-center space-x-1.5 text-[11px] text-slate-400">
              <span class="font-mono">{{ formatDuration(m.game_duration_seconds) }}</span>
              <span 
                v-if="m.is_mvp"
                class="px-1.5 py-0.2 rounded bg-gradient-to-r from-amber-500 to-yellow-400 text-slate-950 text-[9px] font-black shadow-[0_0_8px_rgba(245,158,11,0.4)]"
              >
                👑 MVP
              </span>
              <span 
                v-if="m.is_svp"
                class="px-1.5 py-0.2 rounded bg-gradient-to-r from-purple-500 to-indigo-500 text-white text-[9px] font-black shadow-[0_0_8px_rgba(168,85,247,0.4)]"
              >
                👑 SVP
              </span>
            </div>
          </div>

          <!-- Center Content: Hero & Performance -->
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-2.5">
              <div class="relative shrink-0">
                <img 
                  :src="m.champion_icon || getChampIconFallback(m.champion_id)" 
                  class="w-11 h-11 rounded-lg border border-[#1e2e4a] object-cover" 
                />
                <span v-if="m.role" class="absolute -bottom-1 -left-1 px-1 py-0.2 bg-[#070b14] border border-[#1e2e4a] text-[8px] font-bold text-slate-300 rounded">
                  {{ formatParticipantRole(m.role) }}
                </span>
              </div>

              <div>
                <div class="text-xs font-bold text-slate-100">{{ m.champion_name }}</div>
                <div class="text-xs font-bold text-slate-200 mt-0.5">
                  {{ m.kills }} / <span class="text-red-400">{{ m.deaths }}</span> / {{ m.assists }}
                </div>
                <div class="text-[10px] text-slate-400 font-mono">KDA {{ m.kda_ratio.toFixed(1) }}</div>
              </div>
            </div>

            <div class="text-right">
              <div class="text-xs text-slate-200 font-semibold font-mono">{{ ((m.total_damage || 0) / 1000).toFixed(1) }}k 输出</div>
              <div class="text-[10px] text-[#0ac8b9] font-mono">占比 {{ m.damage_share_pct }}%</div>
              <div class="text-[10px] text-slate-400 font-mono">{{ m.minions_killed }} 刀</div>
            </div>
          </div>

          <!-- Items Row (All 7 Slots Preview) -->
          <div class="flex items-center space-x-1 mt-2.5 pt-2 border-t border-[#162238]/60">
            <div 
              v-for="(item, idx) in (m.items || []).slice(0, 7)" 
              :key="idx" 
              class="w-5 h-5 rounded bg-[#070b14] border border-[#162238] overflow-hidden flex items-center justify-center shrink-0"
              :class="idx === 6 ? 'rounded-full border-[#253959]' : ''"
            >
              <img 
                v-if="item > 0" 
                :src="getItemIconUrl(item)" 
                @error="(e: any) => handleItemImgError(e, item)"
                class="w-full h-full object-cover"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Right: In-depth 10-Player Panoramic Scoreboard -->
      <div class="flex-1 overflow-y-auto p-4 flex flex-col space-y-4">
        <div v-if="matchDetail" class="space-y-4">
          <!-- Game Matchup Head-to-Head Banner (Team Objectives & Gold) -->
          <div class="p-3.5 bg-[#080e1b] rounded-xl border border-[#17253d] flex items-center justify-between shadow-lg">
            <!-- Blue Team Summary -->
            <div class="flex items-center space-x-3">
              <span 
                class="px-2 py-0.5 rounded text-xs font-black uppercase tracking-wider"
                :class="matchDetail.blue_team_win ? 'bg-blue-950/90 text-blue-300 border border-blue-500/50' : 'bg-slate-900 text-slate-400 border border-slate-700'"
              >
                {{ matchDetail.blue_team_win ? '蓝方 胜利' : '蓝方 失败' }}
              </span>
              <span class="text-sm font-black text-slate-100 font-mono">{{ matchDetail.blue_team_kills }} 击杀</span>
              <span class="text-xs text-amber-300 font-mono font-bold">{{ ((blueTeamTotalGold || 0) / 1000).toFixed(1) }}k 金</span>
              <span class="text-xs text-slate-400 font-medium">
                {{ matchDetail.blue_team_turrets }} 塔 · {{ matchDetail.blue_team_dragons }} 龙 · {{ matchDetail.blue_team_barons }} 男爵
              </span>
            </div>

            <!-- Center Match Info -->
            <div class="text-center">
              <div class="font-extrabold text-xs text-slate-200">
                {{ matchDetail.queue_name || matchDetail.game_mode_cn || matchDetail.game_mode || '对局详情' }}
              </div>
              <div class="text-[11px] text-slate-400 font-mono mt-0.5">
                时长 {{ formatDuration(matchDetail.game_duration) }}
              </div>
            </div>

            <!-- Red Team Summary -->
            <div class="flex items-center space-x-3">
              <span class="text-xs text-slate-400 font-medium">
                {{ matchDetail.red_team_turrets }} 塔 · {{ matchDetail.red_team_dragons }} 龙 · {{ matchDetail.red_team_barons }} 男爵
              </span>
              <span class="text-xs text-amber-300 font-mono font-bold">{{ ((redTeamTotalGold || 0) / 1000).toFixed(1) }}k 金</span>
              <span class="text-sm font-black text-slate-100 font-mono">{{ matchDetail.red_team_kills }} 击杀</span>
              <span 
                class="px-2 py-0.5 rounded text-xs font-black uppercase tracking-wider"
                :class="matchDetail.red_team_win ? 'bg-red-950/90 text-red-300 border border-red-500/50' : 'bg-slate-900 text-slate-400 border border-slate-700'"
              >
                {{ matchDetail.red_team_win ? '红方 胜利' : '红方 失败' }}
              </span>
            </div>
          </div>

          <!-- Blue Team 5 Players Scoreboard Table -->
          <div class="bg-[#0b1322] rounded-xl border border-[#1c304f] overflow-hidden shadow-md">
            <!-- Table Header -->
            <div class="px-3 py-1.5 bg-[#0e1b30] flex items-center justify-between text-xs font-bold text-blue-300 border-b border-[#1c304f]">
              <span class="flex items-center space-x-2">
                <span class="w-2 h-2 rounded-full bg-blue-400"></span>
                <span>蓝方阵容 (我方队伍)</span>
              </span>
              <div class="flex items-center space-x-8 text-[11px] text-slate-400 pr-2">
                <span class="w-20 text-center">战损 KDA</span>
                <span class="w-44 text-center">输出 / 承伤对比</span>
                <span class="w-24 text-right">经济 / 补刀</span>
                <span class="w-48 text-left pl-3">出装 (6装备 + 饰品)</span>
              </div>
            </div>
            
            <!-- Blue Participants List -->
            <div class="divide-y divide-[#14233a]">
              <div 
                v-for="p in blueParticipants" 
                :key="p.puuid"
                class="p-2.5 px-3 flex items-center justify-between hover:bg-[#101e36] transition"
              >
                <!-- Hero, Spells & Player Info -->
                <div class="flex items-center space-x-2.5 w-48 shrink-0">
                  <div class="relative shrink-0">
                    <img 
                      :src="p.champion_icon || getChampIconFallback(p.champion_id)" 
                      class="w-10 h-10 rounded-lg border border-[#1e2e4a] object-cover" 
                    />
                    <span class="absolute -bottom-1 -right-1 text-[8px] bg-black/90 px-1 rounded text-slate-300 font-bold font-mono">
                      {{ p.champion_level }}
                    </span>
                  </div>

                  <!-- Spells Stack -->
                  <div class="flex flex-col space-y-0.5 shrink-0">
                    <img :src="p.spell1_icon" class="w-4 h-4 rounded border border-[#1e2e4a]" />
                    <img :src="p.spell2_icon" class="w-4 h-4 rounded border border-[#1e2e4a]" />
                  </div>

                  <!-- Name, Badges & Tier -->
                  <div class="min-w-0 flex flex-col">
                    <div class="text-xs font-bold text-slate-100 truncate flex items-center space-x-1.5">
                      <span class="truncate">{{ p.summoner_name || p.game_name || '召唤师' }}</span>
                      <!-- MVP Badge -->
                      <span 
                        v-if="p.is_mvp" 
                        class="px-1 py-0.2 rounded bg-gradient-to-r from-amber-500 to-yellow-400 text-slate-950 font-black text-[9px] shrink-0 shadow-[0_0_8px_rgba(245,158,11,0.5)]"
                      >
                        👑 MVP
                      </span>
                      <!-- SVP Badge -->
                      <span 
                        v-if="p.is_svp" 
                        class="px-1 py-0.2 rounded bg-gradient-to-r from-purple-500 to-indigo-500 text-white font-black text-[9px] shrink-0 shadow-[0_0_8px_rgba(168,85,247,0.5)]"
                      >
                        👑 SVP
                      </span>
                    </div>
                    <div class="text-[10px] text-slate-400 mt-0.5 flex items-center space-x-1 font-mono">
                      <span>{{ formatParticipantRole(p.role) }}</span>
                      <span>·</span>
                      <span class="text-[#c8aa6e]">{{ p.tier }}</span>
                    </div>
                  </div>
                </div>

                <!-- KDA -->
                <div class="w-20 text-center shrink-0">
                  <div class="text-xs font-bold text-slate-100 font-mono">
                    {{ p.kills }}/<span class="text-red-400">{{ p.deaths }}</span>/{{ p.assists }}
                  </div>
                  <div class="text-[10px] text-slate-400 font-mono">
                    {{ p.kda_ratio !== undefined ? p.kda_ratio.toFixed(1) : (p.kda_str || '0.0') }}
                  </div>
                </div>

                <!-- Dual Damage Comparison Bars (Dealt & Taken) -->
                <div class="w-44 flex flex-col justify-center space-y-1 shrink-0">
                  <!-- Output Damage Bar -->
                  <div>
                    <div class="flex items-center justify-between text-[10px] leading-tight mb-0.5">
                      <span class="flex items-center space-x-1">
                        <span class="text-slate-400 text-[9px]">输出</span>
                        <span class="text-slate-200 font-mono font-medium">{{ ((p.total_damage || 0) / 1000).toFixed(1) }}k</span>
                        <span v-if="p.is_highest_damage" class="text-amber-400 font-bold" title="全场顶伤王者">
                          👑
                        </span>
                      </span>
                      <span class="text-rose-400 font-mono font-semibold">{{ p.damage_share_pct || 0 }}%</span>
                    </div>
                    <div class="h-1.5 bg-[#070b14] rounded-full overflow-hidden border border-white/5">
                      <div 
                        class="h-full bg-gradient-to-r from-amber-500 to-rose-500 rounded-full" 
                        :style="{ width: `${Math.min(100, (p.damage_share_pct || 0) * 2.5)}%` }"
                      ></div>
                    </div>
                  </div>

                  <!-- Damage Taken Bar -->
                  <div>
                    <div class="flex items-center justify-between text-[10px] leading-tight mb-0.5">
                      <span class="flex items-center space-x-1">
                        <span class="text-slate-400 text-[9px]">承伤</span>
                        <span class="text-slate-200 font-mono font-medium">{{ ((p.total_damage_taken || p.damage_taken || 0) / 1000).toFixed(1) }}k</span>
                      </span>
                      <span class="text-indigo-400 font-mono font-semibold">{{ p.damage_taken_share_pct || 0 }}%</span>
                    </div>
                    <div class="h-1.5 bg-[#070b14] rounded-full overflow-hidden border border-white/5">
                      <div 
                        class="h-full bg-gradient-to-r from-blue-500 to-purple-500 rounded-full" 
                        :style="{ width: `${Math.min(100, (p.damage_taken_share_pct || 0) * 2.5)}%` }"
                      ></div>
                    </div>
                  </div>
                </div>

                <!-- Gold & CS -->
                <div class="w-24 text-right text-[11px] shrink-0">
                  <div class="text-amber-300 font-mono font-bold">{{ ((p.gold_earned || 0) / 1000).toFixed(1) }}k 金</div>
                  <div class="text-slate-400 text-[10px] font-mono">
                    {{ p.minions_killed || 0 }} 刀 ({{ computeCsPerMin(p.minions_killed || 0, matchDetail.game_duration) }}/分)
                  </div>
                </div>

                <!-- Items (6 Inventory + 1 Trinket) -->
                <div class="w-48 flex items-center space-x-1 shrink-0 pl-2">
                  <!-- 6 Inventory Items -->
                  <div class="flex items-center space-x-1">
                    <div 
                      v-for="(item, idx) in getItemList(p).inventory" 
                      :key="idx" 
                      class="w-5 h-5 rounded bg-[#070b14] border border-[#1a2d48] overflow-hidden flex items-center justify-center shrink-0"
                    >
                      <img 
                        v-if="item > 0" 
                        :src="getItemIconUrl(item)" 
                        @error="(e: any) => handleItemImgError(e, item)"
                        class="w-full h-full object-cover"
                      />
                    </div>
                  </div>

                  <!-- Trinket Divider -->
                  <div class="w-px h-4 bg-[#1f3554] mx-0.5"></div>

                  <!-- 1 Trinket Ward Item -->
                  <div class="w-5 h-5 rounded-full bg-[#070b14] border border-[#27446d] overflow-hidden flex items-center justify-center shrink-0 shadow-inner" title="饰品眼位">
                    <img 
                      v-if="getItemList(p).trinket > 0" 
                      :src="getItemIconUrl(getItemList(p).trinket)" 
                      @error="(e: any) => handleItemImgError(e, getItemList(p).trinket)"
                      class="w-full h-full object-cover"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Red Team 5 Players Scoreboard Table -->
          <div class="bg-[#1c0e12] rounded-xl border border-[#4a1f28] overflow-hidden shadow-md">
            <!-- Table Header -->
            <div class="px-3 py-1.5 bg-[#261218] flex items-center justify-between text-xs font-bold text-red-300 border-b border-[#4a1f28]">
              <span class="flex items-center space-x-2">
                <span class="w-2 h-2 rounded-full bg-red-400"></span>
                <span>红方阵容 (敌方队伍)</span>
              </span>
              <div class="flex items-center space-x-8 text-[11px] text-slate-400 pr-2">
                <span class="w-20 text-center">战损 KDA</span>
                <span class="w-44 text-center">输出 / 承伤对比</span>
                <span class="w-24 text-right">经济 / 补刀</span>
                <span class="w-48 text-left pl-3">出装 (6装备 + 饰品)</span>
              </div>
            </div>
            
            <!-- Red Participants List -->
            <div class="divide-y divide-[#2a131b]">
              <div 
                v-for="p in redParticipants" 
                :key="p.puuid" 
                class="p-2.5 px-3 flex items-center justify-between hover:bg-[#26131c] transition"
              >
                <!-- Hero, Spells & Player Info -->
                <div class="flex items-center space-x-2.5 w-48 shrink-0">
                  <div class="relative shrink-0">
                    <img 
                      :src="p.champion_icon || getChampIconFallback(p.champion_id)" 
                      class="w-10 h-10 rounded-lg border border-[#3b1924] object-cover" 
                    />
                    <span class="absolute -bottom-1 -right-1 text-[8px] bg-black/90 px-1 rounded text-slate-300 font-bold font-mono">
                      {{ p.champion_level }}
                    </span>
                  </div>

                  <!-- Spells Stack -->
                  <div class="flex flex-col space-y-0.5 shrink-0">
                    <img :src="p.spell1_icon" class="w-4 h-4 rounded border border-[#3b1924]" />
                    <img :src="p.spell2_icon" class="w-4 h-4 rounded border border-[#3b1924]" />
                  </div>

                  <!-- Name, Badges & Tier -->
                  <div class="min-w-0 flex flex-col">
                    <div class="text-xs font-bold text-slate-100 truncate flex items-center space-x-1.5">
                      <span class="truncate">{{ p.summoner_name || p.game_name || '召唤师' }}</span>
                      <!-- MVP Badge -->
                      <span 
                        v-if="p.is_mvp" 
                        class="px-1 py-0.2 rounded bg-gradient-to-r from-amber-500 to-yellow-400 text-slate-950 font-black text-[9px] shrink-0 shadow-[0_0_8px_rgba(245,158,11,0.5)]"
                      >
                        👑 MVP
                      </span>
                      <!-- SVP Badge -->
                      <span 
                        v-if="p.is_svp" 
                        class="px-1 py-0.2 rounded bg-gradient-to-r from-purple-500 to-indigo-500 text-white font-black text-[9px] shrink-0 shadow-[0_0_8px_rgba(168,85,247,0.5)]"
                      >
                        👑 SVP
                      </span>
                    </div>
                    <div class="text-[10px] text-slate-400 mt-0.5 flex items-center space-x-1 font-mono">
                      <span>{{ formatParticipantRole(p.role) }}</span>
                      <span>·</span>
                      <span class="text-[#c8aa6e]">{{ p.tier }}</span>
                    </div>
                  </div>
                </div>

                <!-- KDA -->
                <div class="w-20 text-center shrink-0">
                  <div class="text-xs font-bold text-slate-100 font-mono">
                    {{ p.kills }}/<span class="text-red-400">{{ p.deaths }}</span>/{{ p.assists }}
                  </div>
                  <div class="text-[10px] text-slate-400 font-mono">
                    {{ p.kda_ratio !== undefined ? p.kda_ratio.toFixed(1) : (p.kda_str || '0.0') }}
                  </div>
                </div>

                <!-- Dual Damage Comparison Bars (Dealt & Taken) -->
                <div class="w-44 flex flex-col justify-center space-y-1 shrink-0">
                  <!-- Output Damage Bar -->
                  <div>
                    <div class="flex items-center justify-between text-[10px] leading-tight mb-0.5">
                      <span class="flex items-center space-x-1">
                        <span class="text-slate-400 text-[9px]">输出</span>
                        <span class="text-slate-200 font-mono font-medium">{{ ((p.total_damage || 0) / 1000).toFixed(1) }}k</span>
                        <span v-if="p.is_highest_damage" class="text-amber-400 font-bold" title="全场顶伤王者">
                          👑
                        </span>
                      </span>
                      <span class="text-rose-400 font-mono font-semibold">{{ p.damage_share_pct || 0 }}%</span>
                    </div>
                    <div class="h-1.5 bg-[#070b14] rounded-full overflow-hidden border border-white/5">
                      <div 
                        class="h-full bg-gradient-to-r from-amber-500 to-rose-500 rounded-full" 
                        :style="{ width: `${Math.min(100, (p.damage_share_pct || 0) * 2.5)}%` }"
                      ></div>
                    </div>
                  </div>

                  <!-- Damage Taken Bar -->
                  <div>
                    <div class="flex items-center justify-between text-[10px] leading-tight mb-0.5">
                      <span class="flex items-center space-x-1">
                        <span class="text-slate-400 text-[9px]">承伤</span>
                        <span class="text-slate-200 font-mono font-medium">{{ ((p.total_damage_taken || p.damage_taken || 0) / 1000).toFixed(1) }}k</span>
                      </span>
                      <span class="text-indigo-400 font-mono font-semibold">{{ p.damage_taken_share_pct || 0 }}%</span>
                    </div>
                    <div class="h-1.5 bg-[#070b14] rounded-full overflow-hidden border border-white/5">
                      <div 
                        class="h-full bg-gradient-to-r from-blue-500 to-purple-500 rounded-full" 
                        :style="{ width: `${Math.min(100, (p.damage_taken_share_pct || 0) * 2.5)}%` }"
                      ></div>
                    </div>
                  </div>
                </div>

                <!-- Gold & CS -->
                <div class="w-24 text-right text-[11px] shrink-0">
                  <div class="text-amber-300 font-mono font-bold">{{ ((p.gold_earned || 0) / 1000).toFixed(1) }}k 金</div>
                  <div class="text-slate-400 text-[10px] font-mono">
                    {{ p.minions_killed || 0 }} 刀 ({{ computeCsPerMin(p.minions_killed || 0, matchDetail.game_duration) }}/分)
                  </div>
                </div>

                <!-- Items (6 Inventory + 1 Trinket) -->
                <div class="w-48 flex items-center space-x-1 shrink-0 pl-2">
                  <!-- 6 Inventory Items -->
                  <div class="flex items-center space-x-1">
                    <div 
                      v-for="(item, idx) in getItemList(p).inventory" 
                      :key="idx" 
                      class="w-5 h-5 rounded bg-[#070b14] border border-[#3b1c25] overflow-hidden flex items-center justify-center shrink-0"
                    >
                      <img 
                        v-if="item > 0" 
                        :src="getItemIconUrl(item)" 
                        @error="(e: any) => handleItemImgError(e, item)"
                        class="w-full h-full object-cover"
                      />
                    </div>
                  </div>

                  <!-- Trinket Divider -->
                  <div class="w-px h-4 bg-[#4a222f] mx-0.5"></div>

                  <!-- 1 Trinket Ward Item -->
                  <div class="w-5 h-5 rounded-full bg-[#070b14] border border-[#5c2736] overflow-hidden flex items-center justify-center shrink-0 shadow-inner" title="饰品眼位">
                    <img 
                      v-if="getItemList(p).trinket > 0" 
                      :src="getItemIconUrl(getItemList(p).trinket)" 
                      @error="(e: any) => handleItemImgError(e, getItemList(p).trinket)"
                      class="w-full h-full object-cover"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { SummonerProfile, MatchHistoryItem, MatchDetailFull, MatchDetailParticipant } from '../api/types';
import { getSummonerProfile, getMatchHistory, getMatchDetail } from '../api/tauri';
import { mockSummonerProfile, mockMatchHistory, mockMatchDetail } from '../api/mockData';
import RadarChart from '../components/RadarChart.vue';

const profile = ref<SummonerProfile>(mockSummonerProfile);
const matches = ref<MatchHistoryItem[]>(mockMatchHistory);
const matchDetail = ref<MatchDetailFull | null>(mockMatchDetail);
const selectedGameId = ref<number>(mockMatchHistory[0].game_id);

const activeQueueFilter = ref('all');
const activeLaneFilter = ref('all');

const queueFilterTabs = [
  { id: 'all', name: '全部模式' },
  { id: 'ranked_solo', name: '排位单双' },
  { id: 'ranked_flex', name: '排位灵活' },
  { id: 'aram', name: '极地大乱斗' },
];

const laneFilterTabs = [
  { id: 'all', name: '全部', icon: '🌐' },
  { id: 'top', name: '上路', icon: '🛡️' },
  { id: 'jungle', name: '打野', icon: '🌲' },
  { id: 'mid', name: '中路', icon: '⚡' },
  { id: 'bottom', name: '下路', icon: '🏹' },
  { id: 'support', name: '辅助', icon: '❤️' },
];

function isMatchLaneMatch(matchRole: string, filterLane: string): boolean {
  if (filterLane === 'all') return true;
  const r = (matchRole || '').toUpperCase();
  if (filterLane === 'top') return r.includes('TOP') || r.includes('上');
  if (filterLane === 'jungle') return r.includes('JUNGLE') || r.includes('野');
  if (filterLane === 'mid') return r.includes('MID') || r.includes('中');
  if (filterLane === 'bottom') return r.includes('BOT') || r.includes('ADC') || r.includes('下') || r.includes('射');
  if (filterLane === 'support') return r.includes('SUP') || r.includes('UTIL') || r.includes('辅');
  return true;
}

const filteredMatches = computed(() => {
  return matches.value.filter(m => {
    // 1. Queue Filter
    if (activeQueueFilter.value === 'ranked_solo' && m.queue_id !== 420) return false;
    if (activeQueueFilter.value === 'ranked_flex' && m.queue_id !== 440) return false;
    if (activeQueueFilter.value === 'aram' && m.queue_id !== 450) return false;

    // 2. Lane Filter
    if (activeLaneFilter.value !== 'all' && !isMatchLaneMatch(m.role, activeLaneFilter.value)) return false;

    return true;
  });
});

const blueParticipants = computed(() => {
  return matchDetail.value?.participants?.filter(p => p.team === 'blue') || [];
});

const redParticipants = computed(() => {
  return matchDetail.value?.participants?.filter(p => p.team === 'red') || [];
});

const blueTeamTotalGold = computed(() => {
  return blueParticipants.value.reduce((acc, p) => acc + (p.gold_earned || 0), 0);
});

const redTeamTotalGold = computed(() => {
  return redParticipants.value.reduce((acc, p) => acc + (p.gold_earned || 0), 0);
});

function formatDuration(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m}:${s < 10 ? '0' : ''}${s}`;
}

function computeCsPerMin(cs: number, sec: number): string {
  if (!sec || sec <= 0) return '0.0';
  const min = sec / 60;
  return (cs / min).toFixed(1);
}

function formatParticipantRole(role?: string): string {
  if (!role) return '';
  const r = role.toUpperCase();
  if (r === 'TOP' || r === '上单') return '上路';
  if (r === 'JUNGLE' || r === '打野') return '打野';
  if (r === 'MIDDLE' || r === 'MID' || r === '中单') return '中路';
  if (r === 'BOTTOM' || r === 'ADC' || r === 'BOT' || r === '下路') return '下路';
  if (r === 'UTILITY' || r === 'SUPPORT' || r === 'SUP' || r === '辅助') return '辅助';
  return role;
}

function getItemList(p: MatchDetailParticipant): { inventory: number[]; trinket: number } {
  const items = p.items || [];
  const inventory: number[] = [];
  for (let i = 0; i < 6; i++) {
    inventory.push(items[i] || 0);
  }
  const trinket = p.item_trinket || items[6] || 0;
  return { inventory, trinket };
}

function getItemIconUrl(itemId: number): string {
  if (!itemId || itemId <= 0) return '';
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/item-icons/${itemId}.png`;
}

function handleItemImgError(e: Event, itemId: number) {
  const img = e.target as HTMLImageElement;
  if (img && !img.dataset.fallback) {
    img.dataset.fallback = '1';
    img.src = `https://ddragon.leagueoflegends.com/cdn/14.24.1/img/item/${itemId}.png`;
  }
}

function getChampIconFallback(champId: number): string {
  if (!champId) return '';
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/${champId}.png`;
}

async function selectMatch(gameId: number) {
  selectedGameId.value = gameId;
  try {
    matchDetail.value = await getMatchDetail(gameId);
  } catch (err) {
    console.error('getMatchDetail error:', err);
  }
}

onMounted(async () => {
  try {
    profile.value = await getSummonerProfile();
    matches.value = await getMatchHistory();
    if (matches.value.length > 0) {
      selectedGameId.value = matches.value[0].game_id;
      matchDetail.value = await getMatchDetail(matches.value[0].game_id);
    }
  } catch (err) {
    console.error('MatchHistory init error:', err);
  }
});
</script>
