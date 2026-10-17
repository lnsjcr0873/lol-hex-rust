针对 `lol-hex-rust` 代码库各核心模块的功能逻辑，以下按**致命 Bug（会导致功能错误或崩溃）**、技术陷阱（并发、性能与架构隐患）**以及**边界条件漏洞（极端场景与模式兼容）进行全面审查与定位。

---

## 一、 局内实时感知与客户端通信 (`src/lcu/`)

### 1. 致命 Bug：`skinID` 误作全局皮肤 ID 导致局内英雄被强制识别为“安妮/奥拉夫” (`src/lcu/live_client.rs`)

* **问题定位**：在 `LiveClientDataReader::get_live_stats` 中，代码通过以下逻辑提取 `hero_id`：


```rust
if let Some(skin_id) = p.get("skinID").and_then(|v| v.as_i64()) {
    if skin_id > 0 {
        let cid = if skin_id >= 1000 { skin_id / 1000 } else { skin_id };
        stats.hero_id = cid.to_string();
    }
}

```


* **原因分析**：Riot 2999 端口（`/liveclientdata/allgamedata`）返回的 `allPlayers[i].skinID` **并非 6 位全局皮肤 ID（如 `266001`），而是当前英雄的皮肤槽位偏移序号（原皮为 `0`，第 1 款皮肤为 `1`，第 2 款皮肤为 `2`）**。


* **触发后果**：当玩家使用非原皮（例如 1 号伴生皮肤，`skinID == 1`）时，满足 `0 < skin_id < 1000`，`stats.hero_id` 会被直接赋值为 `"1"`（黑暗之女·安妮）；使用 2 号皮肤则被赋值为 `"2"`（狂战士·奥拉夫）。由于此时 `stats.hero_id` 非空，`main.rs` 中后续通过 `resolve_hero_id(&live_stats.hero_name)` 反查 ID 的补救逻辑被直接跳过，导致所有穿戴皮肤的对局出装与海克斯推荐全部错乱。


* **修复建议**：删除通过 `skinID` 推算 `hero_id` 的逻辑，直接通过 `rawChampionName`（剥离 `game_character_displayname_` 前缀）或 `championName` 结合 `Database::resolve_hero_id` 解析英雄 ID。



### 2. 致命 Bug：Riot ID 同名玩家导致局内装备与阵营张冠李戴 (`src/lcu/live_client.rs`)

* **问题定位**：`check_is_local_player` 在剥离 `#TAG` 后仅比对 `riotIdGameName` 与 `summonerName` 是否一致，且 `get_live_stats` 遍历 `players` 时找到匹配项后未 `break`。


* **原因分析**：Riot ID 机制允许不同玩家拥有完全相同的 `gameName`（仅 `#tagLine` 不同）。若同局内存在两名同名玩家，循环会同时命中两人，将两人的装备全部 `push` 进 `stats.current_items`，且后者的英雄名与阵营会覆盖前者。


* **修复建议**：优先严格全量匹配包含 `#TAG` 的完整 `riotId`，命中后立即處理或跳出本地玩家搜寻循环。



### 3. 技术陷阱：未装游戏时每秒同步拉起 PowerShell 进程造成严重卡顿 (`src/lcu/connector.rs` & `client.rs`)

* **问题定位**：`LcuClient::ensure_credentials` 是异步函数，但内部直接同步调用了 `LcuConnector::sniff_credentials()`。


* **原因分析**：当游戏未启动时，`sniff_from_process_win32` 找不到进程，随即回退到 `sniff_from_commandline_win32`。在 Windows 11 24H2 中 `wmic` 已被系统移除，代码会继续回退并同步执行 `powershell -NoProfile -Command "Get-CimInstance ..."`。由于 `main.rs` 每 1000ms 轮询一次 `get_gameflow_phase()`，这会导致 Tokio 异步工作线程每秒被同步阻塞拉起一次 PowerShell 子进程（耗时 500ms~1500ms），引发严重的 CPU 占用与运行时饥饿。


* **修复建议**：在 `sniff_from_process_win32` 通过 `CreateToolhelp32Snapshot` 遍历进程列表时，若连 `LeagueClientUx.exe` / `LeagueClient.exe` 的 PID 都不存在，应直接返回 `None` 终止嗅探，绝不能继续调用 WMIC 或 PowerShell；此外，进程嗅探应包裹在 `tokio::task::spawn_blocking` 中并增加冷却间隔。



### 4. 技术陷阱：HTTP 回退机制吞掉断连状态导致凭证永不刷新 (`src/lcu/client.rs`)

* **问题定位**：`fetch_match_history_batch` 内部的所有请求分支（包括最终的 HTTP fallback）全部调用了 `handle_response_fallback_candidate` 而非 `handle_response`。


* **原因分析**：`handle_response_fallback_candidate` 遇到连接拒绝或 `401 Unauthorized` 时直接返回 `None`，**既不调用 `record_failure()` 也不调用 `invalidate_credentials()**`。当客户端重启换端口后，调用战绩查询接口永远不会触发重连，且每次请求都会依次经历 2~4 次连接超时。



### 5. 技术陷阱：Singleflight (`in_flight_details`) 在请求失败时的串行重试风暴 (`src/lcu/client.rs`)

* **问题定位**：`get_match_detail` 与 `get_raw_match_timeline` 使用 `in_flight_details` + `Notify` 防止并发穿透。


* **边界条件**：当首个请求因 404 或网络超时返回 `None` 时，`InFlightGuard` 释放并唤醒所有等待协程；等待协程在 `loop` 中检查缓存仍为 `None`，会依次接力再次发起同一个注定失败的 HTTP 请求，导致 N 个并发调用演变为 N 次串行超时等待。



### 6. 边界条件：聊天室 ID 未做 URL 编码导致请求路径被 `#` 截断 (`src/lcu/client.rs`)

* **问题定位**：`get_chat_conversation_participants` 与 `send_chat_message` 直接用 `format!` 拼接 `chat_room_id` 到 URL 路径中。若 `chat_room_id` 含有 `#`、`?` 或空格，会被 URL 解析器当作 Fragment 或 Query 截断，导致请求 404。



---

## 二、 装备合成与数据加载 (`src/data/`)

### 1. 致命 Bug：`normalize_item_id` 字节切片导致中文/多字节输入直接 Panic (`src/data/recipe_data.rs`)

* **问题定位**：
```rust
for pfx_len in [2, 3] {
    if tid.len() > pfx_len {
        let sub = &tid[pfx_len..];
        ...

```



* **原因分析**：在 Rust 中，`&tid[2..]` 是按字节（byte index）切片。若入参包含非 ASCII 字符（如中文字符占 3 字节），切在第 2 或第 3 字节处会直接触发 `byte index is not a char boundary` **线程 Panic 崩溃**！


* **逻辑误伤**：即便全是数字，若传入未收录的 6 位模式专属装备 ID（如 `"223089"`），截断前两位变成 `"3089"`，会错误套用普通模式灭世者的死亡之帽的价格和配方。



### 2. 逻辑 Bug：玩家背包散件未归一化导致带前缀散件无法抵扣金币 (`src/data/recipe_data.rs`)

* **问题定位**：`calculate_item_net_cost` 对目标装备 `target_item_id` 调用了 `normalize_item_id`，但对玩家已拥有的装备 `owned_item_ids` 仅做了 `s.trim().to_string()`，**未调用 `normalize_item_id**`。


* **触发后果**：在大乱斗、斗魂竞技场或海克斯模式中，若客户端返回的背包散件带有模式前缀，在 `satisfy_component` 中执行 `available_owned.iter().position(|x| x == req_item_id)` 时会完全匹配失败，导致散件抵扣金额为 `0`。



### 3. 逻辑 Bug：百分比字符串反序列化量纲不一致 (`src/data/models.rs`)

* **问题定位**：`deserialize_f64_lenient` 将 `"54.2%"` 去除了 `%` 解析为 `54.2`，而代码库中所有胜率（`win_rate`）计算均假设量纲为 `0.0 ~ 1.0`（如 `0.542`）。若云端接口返回带 `%` 的字符串，胜率会被放大 100 倍（显示为 `5420.0%`）。



### 4. 技术陷阱：重复同步 API 导致英雄与海克斯列表无限膨胀 (`src/data/loader.rs`)

* **问题定位**：`Database::sync_from_api` 调用 `populate_heroes` 和 `populate_augments` 时，直接向 `self.heroes_summary` 和 `self.augments_summary` 执行 `.push()`，**未先清空旧 Vec**。若触发重新同步，列表内会出现全量重复元素。



---

## 三、 推荐算法与决策引擎 (`src/engine/archetype.rs`, `counter.rs`, `synergy.rs`, `decision.rs`)

### 1. 致命 Bug：局内出装推荐断路，海克斯强化从未传给推荐引擎 (`src/main.rs` & `src/engine/decision.rs`)

* **问题定位**：
1. 在 `main.rs` 第 631 行调用 `item_bg.update_recommendations_from_live(...)` 时，代表已选海克斯强化的第 9 个参数被硬编码为空切片 `&[]`！


2. 在 `DecisionEngine::get_item_recommendation` 中，构造 `active_aug_values` 时仅生成了 `json!({ "name": a })`，既没有通过 `Database` 将海克斯 ID 解析为中文名称，也**完全缺失了 `"description"` 字段**。




* **触发后果**：
* 局内实时自动出装永远不知道玩家选了什么海克斯强化；


* `HexItemSynergyEngine::rerank_items_with_synergy` 依赖海克斯的 `name` 和 `description` 匹配关键词（如“法术”、“暴击”、“技能急速”），由于没有 `description`（且若传入 ID 则连 `name` 都是数字字符串），**海克斯与装备的联动加分在实际运行中几乎完全失效**！


* `ArchetypeEngine::detect_archetype` 中用 `aug_name.contains(fav)` 匹配流派，但 `favored_tags` 是英文标签（如 `"ABILITY_POWER"`），而 `aug_name` 是中文名或数字 ID，永远不可能匹配成功。





### 2. 致命 Bug：流派检测忽略 `hero_id` 且受 `HashMap` 随机顺序支配 (`src/engine/archetype.rs`)

* **问题定位**：`ArchetypeEngine::detect_archetype` 中 `let _hid = hero_id.trim();` 完全未使用。


* **触发后果**：
1. 调用 `get_hero_archetype_info(hero_id)` 时传入三个空切片 `&[], &[], &[]`，所有流派得分均为 `0`，`arch_scores.into_iter().max_by_key(...)` 会因 `HashMap` 的随机哈希种子而**随机返回一个流派**！


2. 当玩家同时拥有属于两个不同流派的装备时，步骤 1 无序遍历 `&ENGINE_META.archetypes`，谁先被遍历到就锁定谁，结果具有非确定性（Non-deterministic）。





### 3. 逻辑 Bug：敌方威胁分析子串误匹配与克制装遗漏 (`src/engine/counter.rs`)

* **子串误判 (`find_tactical_traits`)**：判断条件 `n.contains(trimmed) || trimmed.contains(n)` 过于宽泛。例如传入短英雄名 `"Vi"`（蔚），会误匹配到 `"Viktor"`（维克托）、`"Sivir"`（希维尔）、`"Anivia"`（艾尼维亚）等，导致敌方威胁特征完全误判。


* **克制装无法注入候选池 (`apply_counter_recommendations`)**：该函数仅遍历英雄原有的 `candidate_items` 并对命中 `urgent` 的装备加分。如果某英雄的常规热门出装池里原本没有收录“巨蛇之牙 (6695)”或“莫雷洛秘典 (3165)”，即便敌方是 5 护盾/5 回血阵容，克制装也无法进入推荐列表。


* **流派标签命名不统一**：`counter.rs` 检测 `"AP"` / `"MAGIC"` / `"MAGIC_RESIST"`，而元数据与 `synergy.rs` / 测试用例中混用了 `"ABILITY_POWER"` / `"MR"` 等不同名称，导致流派过滤失灵。



### 4. 逻辑 Bug：海克斯分级 (`classify_augments`) 将低样本极差强化归为 B 级可用 (`src/engine/decision.rs`)

* **问题定位**：
```rust
if delta < -0.005 && games >= 100 {
    traps.push(a);
} else if delta >= 0.035 {
    s_tier.push(a);
} else if delta >= 0.012 {
    a_tier.push(a);
} else {
    b_tier.push(a);
}

```



* **边界漏洞**：
1. 如果某个垃圾强化胜率暴跌（如 `delta = -0.15`），但因为太弱没人选导致样本数 `games = 80`（`< 100`），它无法进入 `traps`，会直接掉入最后的 `else` 分支被归类为 **`bTier`（常规 B 级可用）**！


2. 反之，`sTier` 和 `aTier` 没有任何最小样本数门槛，仅打过 1 局且获胜的冷门强化会直接封为 `sTier`。


3. 在 `compare_augments` 中，当候选海克斯的 `delta` 处于 `-0.005 <= delta < 0.0` 区间时，单个卡片徽章被标为 `"B" (常规 B 级可用)`，但顶部的 `all_traps`（判定条件为 `delta < 0.0`）却同时触发 `"🚨 全陷阱警告"`，UI 提示自相矛盾。





---

## 四、 战绩解析、对局透视与车队检测 (`src/engine/player_history.rs` & `ranked_draft.rs`)

### 1. 致命 Bug：选人阶段通过聊天室列表按序填充队友导致身份错位 (`src/engine/player_history.rs`)

* **问题定位**：在 `resolve_teammates` 中，当选人阶段 `myTeam` 槽位未暴露 `puuid` 时，代码从 `/lol-chat/v1/conversations/{id}/participants` 拉取聊天室成员列表 `candidate_teammates`，然后用 `c_idx += 1` 顺序填入 `assigned_puuids` 的空槽位。


* **原因分析**：LCU 聊天室参与者列表的排列顺序（按进房时间或字典序）**与选人界面 1~5 楼的槽位顺序（`cellId`）没有任何对应关系**。


* **触发后果**：导致 1~5 楼除自己外的 4 名队友名字、段位、战绩与他们所选的英雄、分路发生错位（例如把 5 楼辅助玩家的战绩安到 2 楼打野英雄头上，并错误给出“⚠️ 试英雄(0场)”警告）。



### 2. 致命 Bug：国服战绩回退扫描 (`discover_game_ids`) 将时间戳误认为 `gameId` 引发 404 风暴 (`src/engine/player_history.rs`)

* **问题定位**：`discover_game_ids` 将整个 `LeagueAkari.db`（SQLite 二进制文件）和最多 40 个客户端日志文件一次性读入内存，用正则 `r"(\d{7,11})"` 提取所有 7~11 位数字，按数值**降序排列**后用于请求 `/lol-match-history/v1/games/{id}`。


* **原因分析与后果**：
1. **正则误匹配时间戳**：数据库和日志中存在海量 10 位 Unix 秒级时间戳（如 `1729000000`）或账号 ID、伤害数值，完全符合 7~11 位数字及 `<= 99_999_999_999` 的条件。


2. **排序优先级倒挂**：若提取出的 10 位伪造数字大于真实 `gameId`，降序排列后这些无效数字会排在最前面，导致 `fetch_and_cache_player` 回退拉取时连续发起大量无效的 404 请求。


3. **阻塞异步线程**：在 async 函数中直接用同步 `std::fs::read` 读取可能高达数百 MB 的 SQLite 文件和 40 个日志文件，并每次重新编译正则表达式，会严重卡死 Tokio 运行时。





### 3. 逻辑 Bug：分页查询战绩时缓存更新但返回值未合并，污染近期画像 (`src/engine/player_history.rs`)

* **问题定位**：`fetch_and_cache_player` 支持传入 `beg_index` 和 `end_index`。当查询第 2 页（如 `20..39`）时，虽然代码将新战绩合并进了 `raw_cache` 的 `existing.matches` 中，但函数末尾返回的是只包含本次分页（`20..39`）的局部变量 `Some(record)`。


* **触发后果**：调用方拿到返回的 `record` 去执行 `compute_tactical_profile` 时，会把第 21~40 场历史旧对局当作“最近 20 场”来计算近期胜率、连胜连败和最近 5 场红绿点。



### 4. 逻辑 Bug：预组队开黑检测 (`detect_premade_teams`) 的传递性误判与跨模式越界 (`src/engine/player_history.rs`)

* **连通分量传递性误判**：算法只要发现两人共同作为队友 `>= 2` 场就建立无向边，最后用 DFS 求连通分量。在高分段或熟人局中，若 A 与 B 曾双排 2 场，B 与 C 曾双排 2 场（A 与 C 从不认识），当三人同队时会被合并为大小 3 的连通分量，误判为 **`"🚗 三排车队"`**。


* **忽视队列人数上限**：在单双排模式（`queue_id == 420`，规则上最多只允许双排）中，该算法同样可能输出 `"三排车队"`、`"四排车队"` 甚至 `"五排车队"`。


* **Fallback 分支敌我不分**：当 `teammate_puuids` 为空进入 fallback 分支时，仅凭 `game_ids` 交集 `>= 2` 即判定为车队，若两人前两局是**对手**而非队友，也会被误判为双排车队。



### 5. 边界条件：分路推断 (`assign_team_positions`) 惩戒优先级高于官方分配分路 (`src/engine/player_history.rs`)

* **问题定位**：`assign_team_positions` 的 Pass 1 遍历槽位，只要发现有人带惩戒（`spell_id == 11`）就强制设为 `"JUNGLE"` 并 `break`，Pass 2 才检查 LCU 官方给出的 `selectedPosition`。


* **边界漏洞**：
1. 若 1 楼上单/中单玩家带了惩戒（或选人初期尚未更换上一把残留的惩戒），1 楼会抢走 `"JUNGLE"` 标签，导致排在后面的真打野在 Pass 2 因 `"JUNGLE"` 已被占用而被迫分配到其他错误分路。


2. 在大乱斗（ARAM）或斗魂竞技场进入 `GameStart` / `InProgress` 时，`resolve_gameflow_10_players` 无条件调用了 `assign_team_positions`，将大乱斗 5 人强行标为“上单/打野/中单/射手/辅助”。





---

## 五、 主控循环、自动化与配置管理 (`src/main.rs`, `src/lib.rs`, `src/config.rs`)

### 1. 致命 Bug：选人阶段“预选亮英雄 (`championPickIntent`)”即触发自动改符文与召唤师技能 (`src/main.rs` & `ranked_draft.rs`)

* **问题定位**：`RankedDraftManager::parse_session_with_db_and_opgg` 提取英雄 ID 时使用了 `m.get("championId").or_else(|| m.get("championPickIntent"))`。而在 `main.rs` 中，只要 `!local.champion_id.is_empty()` 且与上次不同，就立即调用 `lcu_bg.do_apply_runes_and_spells(&db_bg, cid_i64)`。


* **并且 `do_apply_runes_and_spells` 存在三大缺陷** (`src/lcu/client.rs`)：
1. **每次调用都 `OpggClient::new()**`：完全没有复用全局 `opgg_client` 缓存，每次都重新发起网络请求；


2. **分路硬编码为 `"all"**`：`opgg.get_champion_detail(&cid_str, "all")` 忽略了玩家实际分配的分路；


3. **覆盖打野惩戒**：玩家在预选阶段鼠标滑过亮出几个英雄，程序就会连续自动改几次符文，并把召唤师技能直接改为常规闪现+点燃，**把打野玩家的惩戒直接覆盖掉**！




* **修复建议**：仅当玩家真正锁定英雄（或至少 `championId > 0` 而非 `championPickIntent`）时才触发自动配置，且必须传入玩家的实际分配分路 `local.position`；若玩家分路为 `"JUNGLE"`，绝不能将召唤师技能中的惩戒（`11`）覆盖掉。



### 2. 逻辑 Bug：大乱斗备战席自动秒抢 (`auto_swap_bench`) 导致玩家无法手动换人 (`src/main.rs`)

* **问题定位**：在 `ChampSelect` 阶段，若玩家开启了 `auto_swap_bench` 并设置了心愿英雄列表 `[HeroA, HeroB]`。


* **触发后果**：当程序帮玩家秒抢到 `HeroA` 后，如果玩家本局不想玩 `HeroA`，手动从备战席换成了普通英雄 `HeroC`，此时 `HeroA` 回到备战席；下一秒主循环轮询发现玩家手里不是 `HeroA` 且 `HeroA` 在备战席上，**会立即把 `HeroA` 重新换回来**，导致玩家被死死锁在 `HeroA` 上无法更换！


* **修复建议**：记录本局已自动秒抢过的英雄 ID 集合（如 `swapped_in_session`），同一局选人内对同一心愿英雄仅自动抢夺一次。



### 3. 逻辑 Bug：普通极地大乱斗与海克斯大乱斗未做区分，出生 3 级即误报语音 (`src/main.rs` & `live_client.rs`)

* **问题定位**：`live_client.rs` 将所有 `mapNumber == 12` 或 `gameMode == "ARAM"` 均设为 `is_aram = true`。而在 `main.rs` 中，只要 `live_stats.is_aram` 为真且等级达到 `3 | 4 | 7 | 11 | 15`（大乱斗出门即为 3 级），就会自动播报 TTS 语音 `"已升至3级，海克斯强化节点已就绪"` 并启动屏幕 OCR 扫描。


* **边界漏洞**：
1. 在普通极地大乱斗（无海克斯强化）中，玩家一进游戏及每次升到 7/11/15 级都会遭到错误的语音播报和截图扫描干扰。


2. 若玩家在团战中从 6 级直接连升 2 级到 8 级（跳过 7 级），`matches!(current_lvl, 3 | 7 | 11 | 15)` 为 `false`，会漏触发 7 级节点检测。





### 4. 技术陷阱：主轮询循环内串行等待外部网络请求与 `sleep` (`src/main.rs`)

* **问题定位**：`main.rs` 的后台监控任务是一个单协程 `loop`：


1. 在 `ReadyCheck` 自动接受对局时，直接在主循环内 `tokio::time::sleep(Duration::from_secs(delay)).await`（最高阻塞 10 秒）；


2. 在 `ChampSelect` 阶段，每当双方任何一人亮英雄或换分路（`current_sig != last_draft_sig`），主循环就会在循环体内串行等待 `resolve_teammates`（含最高 1.4 秒退避重试）以及多达 10 个槽位的 `opgg_bg` 网络请求。




* **触发后果**：在网络波动或 OP.GG 响应较慢时，主循环单次迭代可卡顿数秒至十余秒，导致大乱斗秒抢英雄延迟、阶段切换滞后以及游戏开始后选人浮窗久久不消失。



### 5. 技术陷阱：`lib.rs` 与 `main.rs` 重复定义全局静态句柄且模块挂载不一致 (`src/lib.rs` & `src/main.rs`)

* **问题定位**：`src/main.rs` 未使用 `lol_hex_rust` 库导出，而是重新声明了一遍 `mod ...` 并定义了自己的 `GLOBAL_TOKIO_HANDLE`，且 `main.rs` 中遗漏了 `mod sgp;`。这导致编译时所有模块被编译两次，且 `lib.rs` 与 `main.rs` 的 `OnceLock` 静态变量完全隔离。



### 6. 技术陷阱：`save_atomic` 固定临时文件名并发冲突与同步刷盘阻塞 (`src/config.rs`)

* **问题定位**：`save_atomic` 始终使用固定路径 `{filename}.tmp` 并调用 `file.sync_all()`。


* **触发后果**：`ConfigManager::save_to_disk` 仅获取读锁（`self.config.read().await`）；若多个异步任务并发调用 `save_to_disk`，会同时争抢创建和重命名同一个 `.tmp` 文件，在 Windows 下触发文件占用错误（`os error 32`）；且在持有 `tokio::sync::RwLock` 锁期间执行同步 `sync_all()` 会阻塞异步执行器线程。



---

## 六、 核心问题优先级修复清单

| 优先级 | 模块 / 文件 | 核心缺陷简述 | 修复方向 |
| --- | --- | --- | --- |
| **P0 (紧急)** | `src/lcu/live_client.rs` | `skinID` 当作英雄 ID 导致穿皮肤时英雄变成安妮/奥拉夫

 | 移除 `skinID` 换算逻辑，统一用 `championName` / `rawChampionName` 查表解析

 |
| **P0 (紧急)** | `src/main.rs` / `decision.rs` | 局内出装推荐传入空海克斯 `&[]`，且未解析海克斯描述导致联动失效

 | 传入局内已选海克斯列表，并在引擎中从 `Database` 补全 `name` 与 `description`<br> |
| **P0 (紧急)** | `src/data/recipe_data.rs` | `normalize_item_id` 字节切片 Panic 隐患，且背包散件未归一化

 | 校验 `is_char_boundary` 或仅对纯数字 ID 脱敏；对 `owned_item_ids` 同步执行 `normalize_item_id`<br> |
| **P0 (紧急)** | `src/main.rs` / `lcu/client.rs` | 预选亮英雄即触发改符文/技能，覆盖打野惩戒，且每次新建 `OpggClient`<br> | 仅在锁定英雄后触发，复用全局 `OpggClient`，传入真实分路并保护打野惩戒

 |
| **P1 (高危)** | `src/engine/player_history.rs` | 聊天室成员按序强填选人槽位导致队友张冠李戴；正则扫库将时间戳当 `gameId`<br> | 无法精确关联槽位时勿强行绑定错误英雄；严格校验 `gameId` 来源格式并异步化文件读取

 |
| **P1 (高危)** | `src/lcu/connector.rs` | 未开游戏时每秒在异步线程同步拉起 PowerShell 进程

 | 快照中无客户端进程时立即返回 `None`，并将嗅探放入 `spawn_blocking`<br> |
| **P2 (中危)** | `src/engine/archetype.rs` | `detect_archetype` 忽略 `hero_id`，空输入时按 `HashMap` 随机顺序返回流派

 | 接入英雄基础定位与预设流派映射，并对流派匹配得分使用确定性排序

 |
| **P2 (中危)** | `src/main.rs` | `auto_swap_bench` 死循环强抢心愿英雄；主循环内串行等待 OP.GG 请求与 `sleep`<br> | 记录单局已抢过的英雄避免重复触发；将耗时网络请求与延迟任务拆分为独立异步任务

 |
