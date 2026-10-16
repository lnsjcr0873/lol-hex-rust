# HexBrain 1:1 复刻 LeagueAkari 实施路线图 (todo.md)

本清单以对齐 `league-akari` 战绩页与对局页的完整体验为目标，按依赖关系由底层向顶层推进，首期聚焦**数据源与底层架构差距**。

---

## 阶段一：数据源与底层架构改造（已完成 · 100% 验收通过）

消除数据传递断层，建立无损的原生 LCU/SGP 契约与高并发数据分发体系。

### 1.1 原生 LCU 数据无损透传接口（消除压平与拼装损耗）
- [x] **Rust 后端透传原生 LCU 战绩与时间线**：
  - 在 `src/lcu/client.rs` 与 `src-tauri/src/cmd.rs` 中新增 `get_raw_match_history(puuid, beg_index, end_index)`，直接返回原始 Riot LCU JSON（不经过 `player_history.rs` 压平处理）。
  - 新增/优化 `get_raw_match_timeline(game_id)`，直接返回原始逐分钟帧与逐事件时间轴。
- [x] **前端废除 `convertMatchItemToLcuGame` 反向拼装**：
  - 修改 `frontend/src/views/MatchHistory.vue`，直接接收原生 LCU 数据并使用 `frontend/src/shared/data-adapter/wrapper.ts` 的 `LcuGameSummary` / `LcuGameTimeline` 进行统一包装。
  - 彻底打通 18 大类数据指标、全部符文局内收益（`var1`, `var2`, `var3`）、技能加点序列（Q/W/E/R 时间轴）与原生事件流。
- [x] **数据无损验证**：
  - 编写自动化比对脚本或测试用例，验证真实对局在经过 `data-adapter` 后指标无任何 `undefined` / 假值丢失（前端测试 104/104 全绿）。

### 1.2 并发加载控制器与请求限流保护
- [x] **并发信号量控制 (Semaphore) 与 Singleflight**：
  - 在 Rust 后端对连续拉取对局详情加入信号量并发控制，并实现 `InFlightGuard` 彻底杜绝并发缓存击穿。
- [x] **内存二级热缓存 (LRU Cache)**：
  - 实现了 `LruStore<u64, Value>`（默认容量 256 场），以 `game_id` 为键快速命中，瞬间完成重复查询，避免打崩 LCU。

### 1.3 SGP (Riot Remote Gateway) 远程跨区客户端骨架
- [x] **SGP 身份凭据提取与会话感知**：
  - 新增 `src/sgp/tokens.rs`，实现从 LCU `/entitlements/v1/token` 自动获取 Token，并由 `src/sgp/servers.rs` 完成国服与各海外服 SGP 节点解析。
- [x] **SGP 客户端抽象层**：
  - 新增 `src/sgp/client.rs`，提供跨区战绩查询通道。
- [x] **SGP GSM (Game State Management) 局内数据补充**：
  - 新增 `src/sgp/gsm.rs`，对齐 Akari 的局内 GSM 模型。

---

## 阶段二：跨页面交互与弹窗系统 (已完成 · 100% 验收通过)

- [x] **战绩长列表虚拟视口与 DOM 性能支撑 (1:1 对标 Akari)**：
  - 引入 `viewport.ts` 并在 `PlayerTabs.vue` 提供 `provideMatchHistoryCardViewport` 全局滚动监听；
  - 引入 `MatchHistoryCard.vue` 包装层，未进入视口时使用 96px 骨架占位（DOM 节点从数万直降至 15~20 个，消除滚动卡顿与内存泄漏），滚出视口自动回收（Recycle）。
- [x] **移植跨页战绩弹窗组件 (Connected Match Previewer)**：
  - 创建 `frontend/src/components/match-preview/`（包含 `types.ts`, `MatchPreviewer.vue`, `ConnectedMatchPreviewer.vue`）；
  - 包装 `MatchCard.vue` 成为全局通用的对局深度预览弹窗 Modal。
- [x] **跨页面无缝弹窗调用**：
  - 在对局页（`OngoingGamePanel.vue`）的迷你近期战绩、侧边栏曾相遇对局（`EncounteredGames.vue`）中接入点击弹窗预览，无需离开当前页面即可查看全场 10 人深度卡片。

---

## 阶段三：多召唤师标签页系统重构 (已完成 · 100% 验收通过)

- [x] **多 Tab 状态管理与路由解耦 (Pinia Store & TabStrip)**：
  - 构建专用的 `player-tabs` Pinia Store（`frontend/src/stores/player-tabs.ts`），管理打开的召唤师标签页数组 (`tabs: [{ id, puuid, sgpServerId, gameName, tagLine, profileIconId, ... }]`)。
  - 实现顶部玩家 Tab 栏（`PlayerTabStrip.vue`），支持 5~10 位召唤师多开，右键上下文菜单（刷新战绩、关闭标签页、关闭其他标签页、关闭右侧标签页）及横向平滑滚轮切换。
  - 提取 `PlayerTab.vue` 隔离每位召唤师的独立滚动视口、分页状态与对局详情缓存，Tab 切换保持 0ms 瞬间响应且无重复网络请求。
- [x] **启动页与搜索引导 (`StartupPane.vue`)**：
  - 补齐未打开任何战绩时的空状态面板，包含品牌标识与介绍、当前客户端登录账号快速卡片（一键打开本人战绩）、模拟演练快捷通道。
  - 集成召唤师实时搜索框与近期搜索历史芯片列表（支持本地持久化、快速重入与单项删除）。
- [x] **全量英雄熟练度弹窗 (`ChampionMasteryModal.vue`)**：
  - 实现全量英雄成就与熟练度弹窗，支持按名称/称号实时输入模糊筛选、按点数/等级/评价三维度排序。
  - 在 `ChampionMasteryPane.vue` 中集成“查看全部”入口与卡片点击唤起，打通 157+ 英雄成就展示。
- [x] **后端召唤师检索与跨玩家 Profile 修复**：
  - 在 `src-tauri/src/cmd.rs` 中新增 `search_summoner` IPC 命令，并修复 `get_summoner_profile` 传入非当前玩家 puuid 时元数据被自身覆盖的缺陷。
  - 前端测试 118/118 全绿，Rust 187+ 单元/集成测试全绿，`npm run build` 0 错误。

---

## 阶段四：高级战绩筛选器与深度收集模式 (已完成 · 100% 验收通过)

- [x] **简易筛选器面板 (`SimpleMatchHistoryFilter.vue`)**：
  - 提供胜负、时间范围（最近3小时/12小时/24小时/3天/7天/30天）、出场分路/位置（上/野/中/下/辅）、指定英雄搜索与多选等快捷组合过滤。
- [x] **AST 树状组合逻辑高级筛选器 (`AdvancedMatchHistoryFilter.vue` & `CombinatorNodeRenderer.vue`)**：
  - 引入可视化树状节点编辑器，支持 `AND`/`OR`/`NOT` 分组，`isChampion`, `isPosition`, `isWin`, `isLoss`, `hasItem`, `hasPerk`, `numberBetween`（KDA、击杀、死亡、助攻、参团率、伤害、承受伤害、金币、补刀）等原子谓词。
  - 基于现有的 `combinators.ts` 谓词解析引擎（`buildPredicateFromSimple` / `buildPredicateFromAst`），实现无损纯函数式过滤。
  - 附带 5 组预设模版（高击杀对局、极高伤害 Carry、败方 MVP/SVP 表现、中单阿狸专精局、高补刀运营局）一键套用。
- [x] **深度收集模式 (Collect Mode)**：
  - 实现后台静默自动翻页抓取多页历史战绩，根据过滤谓词连续扫描，支持自定义每轮抓取量、目标收集场数与最大扫描轮次。
  - 实时悬浮进度条（`CollectModeProgress.vue`）展示当前轮次、命中场数与一键中断（Stop）。
- [x] **完整测试与构建验证**：
  - 新增 `frontend/tests/phase4-filters-collect-mode.test.ts`，涵盖简易/高级 AST 谓词编译与收集模式分页模拟，全部 128/128 前端测试通过，Rust 编译 0 警告 0 错误，`npm run build` 0 错误。

---

## 阶段五：独立对局悬浮窗 (已完成 · 100% 验收通过)

- [x] **Tauri 2 多窗口架构配置**：
  - 在 `src-tauri/tauri.conf.json` 中配置独立的 `ongoing-overlay` 悬浮子窗口（无边框 `decorations: false`、背景透明 `transparent: true`、默认置顶 `alwaysOnTop: true`、启动默认隐藏 `visible: false`）。
- [x] **悬浮窗原生 IPC 命令与生命周期管理**：
  - 在 `src-tauri/src/cmd.rs` 与 `src-tauri/src/lib.rs` 中实现并注册 `toggle_ongoing_overlay`, `set_ongoing_overlay_visible`, `set_ongoing_overlay_always_on_top`, `is_ongoing_overlay_visible`。
  - 在 `frontend/src/api/tauri.ts` 中封装安全的类型化调用与无报错 Fallback 兼容层。
- [x] **悬浮窗独立渲染与路由解耦 (`OngoingOverlayView.vue`)**：
  - 在 `frontend/src/App.vue` 中基于窗口 URL 参数（`?window=overlay`）实现极速入口路由分发，彻底剥离主窗口侧边栏与普通主标题栏开销。
  - 打造独立深色赛博玻璃悬浮窗界面（`OngoingOverlayView.vue`），包含可拖拽区域 (`data-tauri-drag-region`)、置顶锁定开关、最小化隐藏、演练与实时监听无缝切换，并 100% 复用 `OngoingGamePanel.vue` 完整 5v5 分路对位与对阵排布。
  - 在主程序对局页（`OngoingMatch.vue`）右上角增设“独立悬浮窗”快捷呼出/切换动作按钮。
- [x] **全量自动化测试与构建验收**：
  - 新增 `frontend/tests/phase5-standalone-overlay.test.ts`，涵盖多窗口配置校验、路由判别、IPC 健壮性与组件模板源码契约。
  - 前端 10 大测试套件 137/137 项测试通过，Rust 核心测试 187+ 项通过，`cargo check` 0 错误，`npm run build` 0 错误。

---
