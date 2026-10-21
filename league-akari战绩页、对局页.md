如果从**底层原生加解密、特性门控（Feature Gating）、配置版本迁移、设置项跳转联动、主窗口视口注入、下游数据消费（游戏内发送预设）以及内部工具函数**等完整链路来看，**确实还存在以下关键的关联与支撑文件**：

---

### 1. 底层原生模块与对局匿名解密 (Native & Deobfuscation)

在英雄选择阶段和对局初期，国服等大区存在玩家匿名混淆机制，对局模块为了探知玩家身份，直接调用了原生模块进行解密：

* `src/main/native/magic.ts`
  * **作用**：加载各平台专属的原生动态链接库（`magic.win32-x64.node` / `magic.darwin-arm64.node`），导出核心解密函数 `magic(value: string)`。
* `src/main/shards/ongoing-game/champ-select-members.ts`
  * **作用**：直接导入 `magic`，在英雄选择阶段将混淆的 `obfuscatedPuuid` 还原为真实 PUUID，从而让对局页和悬浮窗能提前拉取对手/队友战绩。
* `src/main/native/index.ts` / `src/main/native/addons-win32.ts`
  * **作用**：提供 Windows 提权状态判断、进程前台判定（`isProcessForeground`），用于对局中判定游戏客户端是否在前台。

---

### 2. 对局特性门控系统 (Feature Gating)

对局中的“匿名解密”和“SGP GSM 实时网络透视”属于受控功能，由远程开关和本地门控判定是否激活：

* `src/shared/shards/feature-gating/keys.ts`
  * **核心常量**：
    * `ONGOING_GAME_DEOBFUSCATION_FEATURE_GATE = 'ongoing-game.deobfuscation'`（对局去混淆开关）
    * `ONGOING_GAME_GSM_BY_PUUID_FEATURE_GATE = 'ongoing-game.gsm-by-puuid'`（通过 PUUID 查询实时对局队伍详情开关）
* `src/shared/shards/feature-gating/index.ts`
  * **类**：`FeatureGateEvaluator`，根据大区、版本、操作系统规则评估对局功能是否开放。
* `src/main/shards/feature-gating/index.ts`
  * **作用**：主进程的 `OngoingGameMain` 直接注入该 Shard，在 `state.ts`、`champ-select-handoff-controller.ts` 和 `additional-info-controller.ts` 中根据规则动态启闭对局高级透视。

---

### 3. 对局配置的历史迁移脚本 (Config Migrations)

升级版本时，对局页面的配置与卡片标签存储格式曾发生结构性变更，由以下文件负责平滑转换：

* `src/main/shards/config-migrate/migrations/from-1-4-3.ts`
  * 包含 `migrateOngoingGameSettingsFrom143`（迁移历史对局配置字段）。
* `src/main/shards/config-migrate/migrations/from-1-5-0.ts`
  * 包含 `migrateOngoingGameSettingsFrom150` 和 `repairPlayerCardTagsSetting`（负责修复和补全新版对局卡片 20 余种标签的默认布尔开关）。
* `src/main/shards/config-migrate/migration-runner.ts` / `index.ts` / `context.ts`
  * 数据库启动时按版本顺序运行上述迁移。

---

### 4. 从对局标题栏“直达设置项”的导航系统 (Settings Navigation)

在对局页标题栏（`OngoingGameTitle.vue`）中，点击设置齿轮可以点击“打开完整设置”跳转到主窗口的设置弹窗并**精确定位并闪烁高亮**对应行：

* `src/renderer/src-main-window/settings-navigation/index.ts` —— 导出 `navigateToSetting(...)`。
* `src/renderer/src-main-window/settings-navigation/registry.ts` —— 注册了所有对局相关路由目标（如 `ongoing-game.common`, `ongoing-game.player-card`, `ongoing-game.tags` 等）。
* `src/renderer/src-main-window/settings-navigation/useSettingsNavigationTarget.ts` —— 实现平滑滚动至设置行并播放 CSS 高亮动画。
* `src/renderer/src-main-window/settings-navigation/NavigableSettingsRow.vue` / `NavigableSettingsSection.vue` —— 包装了设置行，使其能够响应对局页发起的锚点跳转。

---

### 5. 主窗口给对局页面的“尺寸与视口注入” (Main Window Context)

对局页卡片采用响应式网格（根据当前主窗口尺寸决定排成 3 列、4 列还是 5 列）：

* `src/renderer/src-main-window/context.ts`
  * 导出 `provideMainWindowAppContext` 和 `useMainWindowAppContext`。
  * 实时监听并输出 `contentWidth` 与 `contentHeight`，由 `src/renderer/src-main-window/views/ongoing-game/OngoingGame.vue` 传给 `OngoingGamePanel` 驱动多列与卡片高度计算。

---

### 6. 战绩详情页内部的“底层数据工具” (Player Tab Data Tools)

在 `src/renderer/src-main-window/views/player-tabs/components/player-tab/` 下还有几个支撑数据加载与页面状态的工具文件：

* `data/source-selection.ts`
  * **作用**：决定当前战绩数据该从 SGP 拉取还是 LCU 拉取（`toLoadStatus`），并在 SGP Token 未就绪或接口不可用时执行降级策略。
* `data/match-history-init-param-collect.ts`
  * **作用**：当在对局页点击“收集该英雄近期对局”或“收集该位置对局”时，负责构造高级战绩过滤器的初始 AST 树状态（`createInitParamCollectFilterState`）。
* `utils/refresh.ts`
  * **作用**：`useRefresh()` 组合式函数，一键将当前战绩 Tab 涉及的 8 组数据（英雄成就、排位、挑战资产、战绩列表、相遇玩家、标签等）统一并发刷新，并汇总 loading 状态。
* `utils/freeze.ts`
  * **作用**：`useFreezeValue()`，在离开战绩 Tab 或切换标签时冻结滚动哨兵状态，防止背景标签页触发不必要的重渲染与视口判断。

---

### 7. 下游消费者：游戏内快捷发送预设 (In-Game Send Presets)

“游戏内发送”功能完全基于对局页分析出的数据来生成局内消息（打野抓人倾向、全队 KDA 评分、开黑车队名单）：

* `src/main/shards/in-game-send/presets/rating.ts` —— 读取 `OngoingGameMain` 产出的 `AggregatedSummaryAnalysis` 和胜率，拼装成游戏内评分发言。
* `src/main/shards/in-game-send/presets/jungle.ts` —— 读取 `OngoingGameMain` 产出的首轮刷野与 Gank 倾向，拼装成提醒队友的打野路径发言。
* `src/main/shards/in-game-send/presets/premade.ts` —— 读取 `OngoingGameMain` 推断出的开黑车队结果，拼装成预组队发言。
* `src/main/shards/in-game-send/preset-selection-controller.ts` —— 实时追踪当前对局页选中的目标玩家（`setRatingPuuids`, `setJunglePuuids`），自动关联对局中的打野或开黑成员。
* `src/main/shards/in-game-send/presets/helpers.ts` / `types.ts` —— 定义以对局玩家数据为入参的数据模型。

---

### 8. 常量与协议路由支撑

* **关键常量 (`src/shared/constants/`)**：
  * `summoner-spells.ts` —— 定义 `SUMMONER_SPELL_SMITE_ID`（惩戒 ID，用于推断打野位置）和 `SUMMONER_SPELL_FLASH_ID`（闪现 ID，用于卡片标签检测“闪现异位/代练”）。
  * `subscribed-lcu-endpoints.ts` —— 定义战绩、英雄选择会话、游戏流的 LCU 事件监听端点。
* **本地与代理协议**：
  * `src/main/shards/akari-protocol/index.ts` / `protocol-router.ts` / `proxy-request-cancellation.ts`
    * 负责在对局页和战绩页高频翻页、切人时，支持取消正在挂起的 `akari://` 本地资源和 SGP 代理 HTTP 请求，避免界面卡顿。
* **对局/战绩路由定义**：
  * `src/renderer/src-main-window/routes/index.ts` —— 注册了 `player-tabs`（支持参数 `:puuid?` 和 `:sgpServerId?`）和 `ongoing-game` 两个一级页面路由。

---

### 总结全景图

将所有相关文件归纳，它们构成了一个完整闭环：
1. **数据源层**：LCU 客户端 + SGP 跨区网络 + 原生 `magic` 模块解密；
2. **主进程服务**：`OngoingGameMain` 状态机与并发队列、SQLite `EncounteredGame` 相遇持久化、特性门控判定；
3. **适配运算层**：`data-adapter` 中的比赛帧解析、打野路径推断、开黑图算法与 Akari 评分系统；
4. **前端展现层**：主窗口 `player-tabs`（含拖拽栏与高级过滤器）、主窗口 `ongoing-game` 面板、以及独立的 `src-ongoing-game-window` 悬浮窗和 `src-cd-timer-window` 技能倒计时窗口；
5. **周边生态**：战绩/对局专属设置页、搜索弹窗、主播假名遮罩，以及下游的“游戏内发送”战术广播。