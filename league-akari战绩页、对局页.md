在 LeagueAkari 中，**战绩详情页**（基于玩家标签页的战绩列表与深入的单场战绩卡片）和**对局页**（主窗口对局分析、独立悬浮对局窗口及其实时数据面板）是核心功能模块，二者大量复用了战绩卡片、打野路径推断、Akari 评分等公共组件与数据适配层。

以下是这两个页面涉及到的**所有核心及关联文件清单**（按模块与职责分类）：

---

### 一、 战绩详情页相关文件 (Match History & Match Card)

战绩详情在 UI 上分为两层：**玩家主页战绩列表/筛选器** 与 **展开后的单场战绩卡片详情**。

#### 1. 玩家战绩页容器与列表 (`src/renderer/src-main-window/views/player-tabs/`)
* **页面入口与布局**：
  * `PlayerTabs.vue` —— 玩家标签页视图容器
  * `components/StartupPane.vue` —— 未打开任何战绩时的初始启动页
  * `components/player-tab/PlayerTab.vue` —— 单个玩家的主页面布局（包含头部、侧边栏、滚动容器）
  * `components/player-tab/PlayerTabSidebarContent.vue` —— 战绩侧边栏内容（集成排位、战力、相遇记录等）
  * `components/player-tab/GlobalStateTracker.ts` —— 玩家标签页全局同步监听器
  * `components/player-tab/constants.ts` / `context.ts` / `init-params.ts` —— 玩家标签上下文与初始化参数
* **侧边栏概览卡片**：
  * `components/player-tab/widgets/PlayerTabHeader.vue` —— 召唤师头像、段位、改名遮罩等头部信息
  * `components/player-tab/widgets/SummaryPane.vue` —— 近期胜率、KDA、Akari 评分、英雄胜率汇总
  * `components/player-tab/widgets/ChampionMasteryPane.vue` / `ChampionMasteryModal.vue` —— 英雄熟练度展示及弹窗
  * `components/player-tab/widgets/RecentlyPlayers.vue` —— 近期同队/对手玩家统计
  * `components/player-tab/widgets/PlayerChallenges.vue` —— 资产与成就挑战统计
  * `components/player-tab/widgets/EncounteredGames.vue` —— 与该玩家曾相遇的对局记录
  * `components/player-tab/widgets/NormalTagBlock.vue` —— 玩家标记备注与隐私状态
* **战绩列表与分页控制**：
  * `components/player-tab/widgets/MatchHistoryList.vue` —— 战绩列表循环与平滑滚动定位
  * `components/player-tab/widgets/match-history-card/MatchHistoryCard.vue` —— 单项战绩卡片包装器（带虚拟视口优化与占位）
  * `components/player-tab/widgets/match-history-card/viewport.ts` / `constants.ts` / `types.ts` —— 视口优化与类型
  * `components/player-tab/widgets/match-history-pagination/MatchHistoryPagination.vue` —— 战绩分页主控制器
  * `components/player-tab/widgets/match-history-pagination/NormalPagination.vue` —— 普通翻页控制器
  * `components/player-tab/widgets/match-history-pagination/CollectModePagination.vue` / `CollectModeDescription.vue` —— 收集模式翻页与进度展示
  * `components/player-tab/widgets/match-history-pagination/PageControls.vue` / `PageFilterControl.vue` / `QueueSelect.vue` / `FilterButton.vue` —— 翻页/队列/筛选按钮控件
  * `components/player-tab/widgets/match-history-pagination/use-collect-mode-filter-draft.ts` —— 收集模式草稿逻辑
* **高级战绩筛选器 (Match History Filters)**：
  * `components/player-tab/widgets/match-history-filters/MatchHistoryFilters.vue` —— 筛选弹窗入口
  * `components/player-tab/widgets/match-history-filters/SimpleMatchHistoryFilter.vue` —— 简易筛选面板（胜负、时间、位置、英雄、召唤师）
  * `components/player-tab/widgets/match-history-filters/AdvancedMatchHistoryFilter.vue` —— 组合逻辑高级树状筛选
  * `components/player-tab/widgets/match-history-filters/CombinatorComp.vue` —— 树节点渲染器
  * `components/player-tab/widgets/match-history-filters/CollectModeProgress.vue` —— 深度匹配收集浮动进度条
  * `components/player-tab/widgets/match-history-filters/filter-state.ts` / `context.ts` / `combinator-nodes.ts` / `combinator-runtime.ts` / `combinator-specs.ts` / `combinator-factories.ts` —— 组合器 AST 运行逻辑
  * `components/player-tab/widgets/match-history-filters/combinator-components/*` (包含 `AndOr.vue`, `Game.vue`, `IsChampion.vue`, `HasItem.vue`, `HasPerk.vue`, `NumberBetween.vue`, `WinResult.vue`, `OfMembers.vue` 等 16 个具体原子节点组件)
  * `components/player-tab/widgets/match-history-filters/presets/` —— 预设筛选器模板

#### 2. 单场战绩卡片本体及详情 Tabs (`src/renderer-shared/components/match-card/`)
* **卡片框架**：
  * `MatchCard.vue` —— 单场战绩卡片容器（提供 MatchCardContext）
  * `MatchCardOverview.vue` —— 战绩收起态的总览卡片（KDA、伤害占比、装备、补兵、召唤师技能等）
  * `MatchCardDetails.vue` —— 战绩展开态的详情主容器与 Tab 切换器
  * `context.ts` / `constants.ts` / `types.ts` —— 卡片数据上下文
* **详情各个选项卡 (Tabs)**：
  * `tabs/MatchCardSummaryTab.vue` —— **数据总览 Tab**：两队对局数据对比表格、史诗野怪/推塔统计
  * `tabs/MatchCardDetailsTab.vue` —— **详细数据 Tab**：包含上百个技术指标的详细对比表格（技能命中、插眼排眼、金币转化等）
  * `tabs/MatchCardRunesTab.vue` —— **符文 Tab**：展示全场所有人的主副系符文及局内实际收益数值
  * `tabs/MatchCardBuildsTab.vue` —— **技能/装备构建 Tab**：全场玩家随游戏时间的加点路线与出装时间轴
  * `tabs/MatchCardEventsTab.vue` —— **事件时间轴 Tab**：击杀、推塔、破龙、击杀位置与受击伤害明细
  * `tabs/timeline/MatchCardTimelineTab.vue` —— **时间线图表 Tab**：经济/补兵/伤害曲线主容器
  * `tabs/timeline/MatchCardDiffLineChart.vue` —— 团队/个人经济与伤害走势折线图
  * `tabs/timeline/MatchCardStatsLine.vue` —— 逐帧面板属性回放与地图位置轨迹
* **卡片辅助微件与工具 (`widgets/` & `utils/`)**：
  * `widgets/TeamTable.vue` —— 队伍对阵总览表
  * `widgets/RadarChart.vue` —— 个人六维雷达图（KDA、伤害、承伤、经济、补刀、参团）
  * `widgets/DamageBar.vue` / `DamageBarWithPopover.vue` —— 物/魔/真伤多段比例条
  * `widgets/VictimDamageDetails.vue` —— 死亡受到伤害来源明细
  * `widgets/MapPosition.vue` —— 地图坐标点绘制（召唤师峡谷、极地大乱斗、斗魂竞技场）
  * `widgets/StatsBarChart.vue` / `TabSwitch.vue` / `ManyTags.vue`
  * `utils/details-table/` (`groups.ts`, `raw-details.ts`, `types.ts`) —— 局内详细指标分类与表格解析器
  * `utils/game-map.ts` / `game-details.ts` / `text.ts` / `time.ts` / `theme.ts` —— 地图映射、时间格式化与排版工具

#### 3. 跨页对局弹窗预览 (`src/renderer-shared/components/match-preview/`)
* `MatchPreviewer.vue` —— 战绩弹窗预览 Modal（包装 MatchCard）
* `ConnectedMatchPreviewer.vue` —— 自动通过 SGP/LCU API 异步加载指定 GameId 详情并弹窗展示
* `index.ts` —— 预览状态类型定义

---

### 二、 对局页相关文件 (Ongoing Game)

对局页用于实时展现选人/局内进行中的友方与敌方玩家卡片、段位、熟练度、打野意图推断和战绩回溯。

#### 1. 主窗口对局视图与标题栏
* `src/renderer/src-main-window/views/ongoing-game/OngoingGame.vue` —— 主窗口对局页面容器（接入 ConnectedMatchPreviewer 与 OngoingGamePanel）
* `src/renderer/src-main-window/components/titlebar/OngoingGameTitle.vue` —— 标题栏的对局状态区（地图模式名、红蓝方、草稿模拟退出、一键刷新）

#### 2. 独立对局悬浮窗 (`src/renderer/src-ongoing-game-window/`)
* `App.vue` —— 独立悬浮对局窗口的顶层组件
* `main.ts` / `NaiveUIProviderApp.vue` / `shards/index.ts` —— 独立对局窗口的主进程注入与应用启动文件
* `src/main/shards/window-manager/ongoing-game-window/window.ts` / `state.ts` —— 主进程对局窗口生命周期与快捷键唤出控制

#### 3. 对局面板核心组件 (`src/renderer-shared/components/ongoing-game-panel/`)
* `OngoingGamePanel.vue` —— 对局面板网格主容器（支持队伍分栏、宽度/高度自适应、草稿演练模式模式背景）
* `constants.ts` / `context.ts` —— 对局布局常量（卡片固定宽度 190px 等）与 Context 定义
* `widgets/OngoingGameTeam.vue` —— 单支队伍容器（队伍头部、成员卡片网格）
* `widgets/TeamTagsArea.vue` —— 队伍综合战力指标（队伍平均胜率、平均 KDA、预组队开黑标签、胜率队/车队标识）
* `widgets/TinyPlayerChampionList.vue` —— 队伍开黑成员弹窗悬浮列表
* **玩家信息卡片 (`widgets/player-info-card/`)**：
  * `PlayerInfoCard.vue` —— 核心单人卡片外框（开黑色块标识）
  * `PlayerInfoCardHeader.vue` —— 英雄头像、等级、召唤师名、双排段位简报、玩家操作菜单（去战绩页/打野过滤/位置过滤）
  * `PlayerInfoCardStats.vue` —— 综合胜率、第一率、近期 KDA 及离群值颜色指示、位置分配来源（首选/补位/次选）
  * `PlayerInfoCardChampionUsage.vue` —— 该玩家熟练度/近期最高频英雄图标小条
  * `PlayerInfoCardMatchHistory.vue` —— 该玩家近期的简要战绩迷你列表（虚拟滚动）
  * `PlayerInfoCardJunglePathing.vue` —— 当前玩家若为打野时的路径倾向微件
  * `player-actions.ts` / `jungle-pathing-visibility.ts` —— 玩家行为菜单与打野展示条件推断
* **玩家卡片标签系统 (`widgets/player-info-card/player-card-tags/`)**：
  * `TagsArea.vue` —— 卡片标签展示区
  * `context.ts` / `types.ts` —— 标签计算上下文
  * `tags/index.ts` —— 标签注册（包含连胜/连败、通天代、带崩全队、单杀怪、容易被抓、闪现置位异常、遇到过、标记备注、隐私小号、车队等 20 余种逻辑标签）

---

### 三、 两个页面共用的分析与可视化展示组件

* **打野路径分析 (`src/renderer-shared/components/jungle-pathing-analysis/`)**：
  * `JunglePathingContent.vue` / `JunglePathingSection.vue` —— 打野分析弹层核心结构
  * `GankPreferenceSummary.vue` —— Gank 倾向总结（上中下路热度、野区权重）
  * `FirstClearAndGankSummary.vue` —— 首轮刷野路线（红蓝开、是否入侵、3/4级抓人首杀点）
  * `GankMap.vue` —— 打野地图热力图与击杀标记渲染器
  * `MapPreferenceSummary.vue` / `ObjectivesSummary.vue` —— 地图倾向与史诗野怪控制率（控龙率、先锋率）
  * `preference.ts` / `format.ts` / `types.ts` —— 打野倾向算法权重与格式化
* **Akari 评分面板 (`src/renderer-shared/components/akari-score/`)**：
  * `AkariScorePopover.vue` / `AkariScorePopoverContent.vue` —— 评分浮层弹出框
  * `score-breakdown.ts` —— Akari 评分 9 个维度（KDA、胜率、输出、承伤、治疗、补兵、经济、参团、视野）的拆解条形图
* **英雄局内分析悬浮框 (`src/renderer-shared/components/champion-analysis/`)**：
  * `ChampionAnalysisContent.vue` —— 鼠标悬停英雄头像时弹出的完整深度分析
  * `ChampionMasteryContent.vue` —— 英雄熟练度里程碑与最高评级展示
  * `ChampionPositionChart.vue` —— 英雄分路占比环形图
  * `types.ts` —— 英雄分析相关数据结构

---

### 四、 状态管理、上下文与 Provider

#### 1. 前端 Provider (`src/renderer-shared/providers/`)
* **静态游戏资源 (`akari-resource/`)**：
  * `akari.ts` / `context.ts` / `types.ts` / `AkariResourceProvider.vue` —— 负责为战绩卡片和对局卡片解析英雄名字、技能图标、符文图片、装备合成、队列名称等 LCU/本地静态资源
* **对局上下文 Provider (`ongoing-game/`)**：
  * `akari.ts` / `context.ts` / `types.ts` / `OngoingGameProvider.vue` —— 封装主进程推送到渲染层的全局对局状态，供 `OngoingGamePanel` 解耦调用

#### 2. Renderer Shards (`src/renderer-shared/shards/` & `src/renderer/src-main-window/shards/`)
* **对局 Shard (`renderer-shared/shards/ongoing-game/`)**：
  * `store.ts` —— Pinia Store，管理对局进行时的两队成员、英雄、召唤师技能、战绩缓存、开黑推断数据
  * `store-event-handlers.ts` —— 监听主进程的对局事件推送（如更新成员、批量填充战绩）
  * `index.ts` / `context.ts` —— 暴露 `OngoingGameRenderer` 实例方法（重载玩家、修改查询深度等）
* **玩家战绩标签页 Shard (`src-main-window/shards/player-tabs/`)**：
  * `store.ts` —— Pinia Store，管理所有已打开的召唤师战绩标签页（多 Tab 切换、关闭、刷新）
  * `index.ts` / `init-params.ts` / `page-size-options.ts` / `player-tabs-watcher.ts` / `search-history-service.ts` —— 页面大小控制、自动路由监听、Tab 跳转参数序列化

---

### 五、 数据适配层与分析算法 (`src/shared/data-adapter/`)

战绩卡片与对局卡片的数据计算引擎：

* **战绩基础模型解析 (`match-history/`)**：
  * `match-basic.ts` —— 解析胜负、时长、地图、队列模式等基础字段（统一 LCU 与 SGP）
  * `participants.ts` —— 解析所有玩家指标（KDA、伤害、金币、补刀、出装、符文变量等）
  * `teams.ts` —— 队伍整体指标汇总（大龙/小龙/巢虫击杀数、两队伤害总和、禁选英雄）
  * `frames.ts` —— 时间轴数据帧与事件解析（击杀、技能加点、购装、镀层等）
  * `identities.ts` —— 提取对局参与者身份映射
  * `win-result.ts` —— 判断获胜、失败、重开或中途退出
  * `cherry.ts` —— 斗魂竞技场（2v2v2v2）排名与胜负逻辑解析
* **玩家单场/聚合数据分析 (`analysis/player/`)**：
  * `index.ts` —— `analyzeGames` 入口，将多场比赛聚合为高维画像
  * `single/summary.ts` / `single/details.ts` / `single/jungle.ts` / `single/akari.ts` / `single/early-deaths.ts` —— 单局战绩画像提取
  * `aggregate/summary.ts` / `aggregate/jungle.ts` / `aggregate/positions.ts` / `aggregate/akari.ts` / `aggregate/spells.ts` / `aggregate/win-loss.ts` —— 多场战绩画像聚合
  * `scoring.ts` / `constants.ts` —— Akari 评分计算公式及常量阈值
* **团队分析与组队开黑推断**：
  * `analysis/team/index.ts` —— 计算整支队伍的平均胜率、KDA 及战力分散度
  * `analysis/relationship/index.ts` —— 分析与当前召唤师曾共同游戏的交集历史
  * `src/shared/utils/team-up-calc.ts` —— 并查集与图算法，根据历史重叠场次推断两队中存在的预组队开黑群体
* **数据包裹器**：
  * `wrapper.ts` —— 抹平 LCU 与 SGP 接口返回的数据结构差异 (`LcuOrSgpGameSummary`, `LcuOrSgpGameDetails`)
* **高级筛选器谓词**：
  * `predicates/combinators.ts` —— 战绩高级筛选树执行器

---

### 六、 主进程后台支持 (`src/main/shards/ongoing-game/`)

负责在 Electron 主进程中后台拉取对局、分析并发度并推送到前端：

* `index.ts` —— `OngoingGameMain` 核心 Shard 入口
* `state.ts` —— 对局主进程 MobX 状态管理
* `context.ts` / `ipc-handlers.ts` —— 对局相关 IPC 处理程序
* `computed-state.ts` —— 英雄选择阶段 / 房间阶段 / 游戏内阶段的状态合并计算（如分配位置、当前选定英雄）
* `match-history-loader.ts` —— 战绩批量加载控制器（带 LRU 缓存与并发队列 QueueKeeper）
* `player-data-loader.ts` —— 召唤师基础信息、排位信息、熟练度信息并行加载器
* `analysis-controller.ts` —— 驱动 `analyzeGames` 和预组队推断计算并同步至 State
* `additional-info-controller.ts` —— 通过 SGP GSM 接口在游戏进行时补全玩家天赋与召唤师技能
* `champ-select-handoff-controller.ts` / `champ-select-handoff.ts` / `champ-select-members.ts` —— 将英雄选择阶段探知的匿名信息安全交接到游戏进行阶段
* `side-effects-controller.ts` —— 游戏结束自动存储对局记录、标记玩家提示

---

### 七、 底层通信与 API 客户端

* `src/shared/http-api-axios-helper/league-client/match-history.ts` —— LCU 本地战绩与时间轴 API
* `src/shared/http-api-axios-helper/sgp/match-history-query.ts` —— SGP 远程跨区战绩查询 API
* `src/shared/http-api-axios-helper/sgp/gsm.ts` —— SGP 局内实时游戏状态 API
* `src/shared/http-api-axios-helper/league-client/champ-select.ts` —— 英雄选择会话 API
* `src/shared/http-api-axios-helper/league-client/gameflow.ts` —— 游戏流程阶段 API