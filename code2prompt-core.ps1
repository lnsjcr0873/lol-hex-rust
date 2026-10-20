# ==============================================================================
# HexBrain 核心业务代码打包脚本 (过滤静态字典/样板/类型/构建等周边文件)
# ==============================================================================

$ts = Get-Date -Format "yyyyMMdd_HHmmss"

$params = @(
  # 包含核心代码
  "-i", "src/**/*.rs",
  "-i", "src-tauri/src/**/*.rs",
  "-i", "frontend/src/**/*.{vue,ts,js}",

  # 1. 静态资源、旧版脚本与样式
  "-e", "static/*",
  "-e", "frontend/src/style.css",
  "-e", "frontend/postcss.config.js",
  "-e", "frontend/tailwind.config.js",
  "-e", "frontend/index.html",

  # 2. Mock 假数据与离线验证测试
  "-e", "frontend/src/api/mockData.ts",
  "-e", "frontend/src/shared/data-adapter/adversarial-suite.ts",
  "-e", "frontend/src/shared/data-adapter/verify-adapter.ts",

  # 3. 静态超长字典与死数据表
  "-e", "frontend/src/shared/constants/champion-names.ts",
  "-e", "frontend/src/shared/constants/summoner-spells.ts",
  "-e", "frontend/src/shared/constants/subscribed-lcu-endpoints.ts",
  "-e", "frontend/src/shared/constants/common.ts",
  "-e", "frontend/src/shared/utils/platform-names.ts",
  "-e", "frontend/src/shared/utils/yuriyuri-names.ts",
  "-e", "frontend/src/shared/utils/text-replacer.ts",
  "-e", "frontend/src/shared/utils/map-where.ts",
  "-e", "frontend/src/shared/utils/keyboard-shortcuts.ts",
  "-e", "frontend/src/shared/data-adapter/tactical-tags/constants.ts",
  "-e", "frontend/src/components/match-card/constants.ts",
  "-e", "frontend/src/components/match-card/utils/resources.ts",
  "-e", "frontend/src/components/match-card/utils/text.ts",
  "-e", "frontend/src/components/match-card/utils/details-table/groups.ts",
  "-e", "frontend/src/components/ongoing-game-panel/constants.ts",
  "-e", "frontend/src/components/player-tab/widgets/match-history-card/constants.ts",

  # 4. 纯 TS 类型定义 (所有 types.ts 与 d.ts)
  "-e", "frontend/src/api/types.ts",
  "-e", "frontend/src/env.d.ts",
  "-e", "frontend/src/shared/types/*",
  "-e", "frontend/src/shared/data-adapter/tactical-tags/types.ts",
  "-e", "frontend/src/shared/data-adapter/analysis/player/types/*",
  "-e", "frontend/src/components/champion-analysis/types.ts",
  "-e", "frontend/src/components/jungle-pathing-analysis/types.ts",
  "-e", "frontend/src/components/match-card/types.ts",
  "-e", "frontend/src/components/match-card/utils/details-table/types.ts",
  "-e", "frontend/src/components/match-preview/types.ts",
  "-e", "frontend/src/components/ongoing-game-panel/types.ts",
  "-e", "frontend/src/components/player-tab/widgets/match-history-card/types.ts",
  "-e", "frontend/src/**/types.ts",
  "-e", "frontend/src/**/*.d.ts",

  # 5. 纯 SVG 图标与纯展示包裹组件
  "-e", "frontend/src/components/icons/*",
  "-e", "frontend/src/components/match-card/icons/*",
  "-e", "frontend/src/components/match-card/widgets/DamageBar.vue",
  "-e", "frontend/src/components/match-card/widgets/PositionIcon.vue",
  "-e", "frontend/src/components/match-card/widgets/TabSwitch.vue",
  "-e", "frontend/src/components/match-card/tabs/MatchCardTimelineTab.vue",
  "-e", "frontend/src/components/jungle-pathing-analysis/JunglePathingSection.vue",
  "-e", "frontend/src/components/akari-score/AkariScorePopover.vue",
  "-e", "frontend/src/components/player-tab/PlayerTabSidebarContent.vue",
  "-e", "frontend/src/components/widgets/ChampionIcon.vue",
  "-e", "frontend/src/components/RadarChart.vue",
  "-e", "frontend/src/components/Sidebar.vue",
  "-e", "frontend/src/components/Titlebar.vue",
  "-e", "frontend/src/App.vue",

  # 6. 纯转发/导出 Barrel 文件
  "-e", "frontend/src/components/*/index.ts",
  "-e", "frontend/src/shared/data-adapter/index.ts",
  "-e", "frontend/src/shared/data-adapter/tactical-tags/index.ts",
  "-e", "frontend/src/shared/data-adapter/opgg-aram-balance/index.ts",
  "-e", "frontend/src/shared/data-adapter/summoner/index.ts",
  "-e", "frontend/src/shared/data-adapter/match-history/identities.ts",
  "-e", "frontend/src/shared/data-adapter/wrapper.ts",
  "-e", "frontend/src/components/match-card/utils/game-map.ts",
  "-e", "src/**/mod.rs",

  # 7. 纯辅助工具函数与周边表单视图
  "-e", "frontend/src/shared/utils/format.ts",
  "-e", "frontend/src/shared/utils/sleep.ts",
  "-e", "frontend/src/shared/utils/uri.ts",
  "-e", "frontend/src/shared/utils/name.ts",
  "-e", "frontend/src/shared/data-adapter/analysis/player/utils/empty.ts",
  "-e", "frontend/src/views/Settings.vue",
  "-e", "frontend/src/views/Automation.vue",
  "-e", "src/crawler/exporter.rs",

  # 8. 构建配置与说明文档
  "-e", "Cargo.toml",
  "-e", "README.md",
  "-e", "PROJECT.md",
  "-e", "frontend/package.json",
  "-e", "frontend/tsconfig.json",
  "-e", "frontend/vite.config.ts",
  "-e", "frontend/src/main.ts",
  "-e", "src-tauri/Cargo.toml",
  "-e", "src-tauri/build.rs",
  "-e", "src-tauri/capabilities/default.json",
  "-e", "src-tauri/tauri.conf.json",
  "-e", "src-tauri/src/main.rs",
  "-e", "src/web/assets.rs",

  "--encoding", "cl100k",
  "-O", "prompt_$ts.md"
)

code2prompt @params
