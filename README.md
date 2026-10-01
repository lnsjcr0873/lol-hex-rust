# HexBrain (Rust 原生重构版)

**英雄联盟海克斯大乱斗全量数据同步与实时决策辅助系统 (HexBrain Esports Edition)**

基于 1157万+ 局真实对局大数据驱动的海克斯大乱斗全流程实时决策支持系统，采用 **100% 原生 Rust** 高性能重构。

---

## ⚡ 核心优势与性能表现

| 指标 | Python 版本 | Rust 原生重构版 | 提升幅度 |
| :--- | :--- | :--- | :--- |
| **可执行文件大小** | ~350MB (带环境) | **< 20MB** 单文件独立程序 | **体积缩小 94%** |
| **运行时内存占用** | 350MB ~ 600MB | **15MB ~ 30MB** | **内存降低 95%** |
| **启动耗时** | 2.5s ~ 4.0s | **< 50ms** 秒开 | **快 50 倍** |
| **OCR 推演延迟** | 200ms ~ 300ms | **15ms ~ 30ms** | **快 10 倍** |
| **环境依赖** | 需要 Python / C++ 编译环境 | **零依赖**，双击即运行 | 免配置 |

---

## 🏛️ 系统架构设计

```text
lol-hex-rust/
├── Cargo.toml                  # 依赖清单与构建优化配置
├── .github/workflows/ci.yml    # GitHub Actions 自动化构建与测试工作流
├── data/                       # 173 英雄与 209 强化本地全量知识库
│   ├── json/                   # 英雄榜单、强化全图鉴、赛后画像与独立详情
│   ├── engine_metadata.json    # 英雄流派画像、针对出装与梦幻联动规则库
│   └── recipe_metadata.json    # 装备合成树、散件抵扣与净金币费用表
├── server/
│   ├── config/                 # 算法预设 (SMART, HEX_FUN, HYBRID_FREE, IRON_COUNTER)
│   └── static/                 # Vue 3 + Tailwind CSS 高清电竞看板前端 (嵌入式)
└── src/
    ├── main.rs                 # 主程序入口、多线程调度与单元测试套件
    ├── config.rs               # 算法偏好与冲突协调管理器
    ├── data/                   # 内存数据库、多级索引与合成费用逆向推演
    ├── engine/                 # 威尔逊区间置信度、流派锁定、针对装与三选一推演
    ├── lcu/                    # 进程命令行嗅探、锁文件解析、LCU 与 2999 端口通讯
    ├── vision/                 # 超宽屏视口补偿、局部快速捕获与原生 WinRT OCR 识别
    ├── overlay/                # 方案 A: Win32 Layered Window 置顶 HUD、热键与语音
    ├── crawler/                # 抗反爬数据采集、OP.GG 抓取与 9 维度 CSV 导出
    └── server/                 # Axum REST API、WebSocket 广播与静态资源嵌入
```

---

## 🎮 游戏内置顶 HUD 体系 (方案 A)

采用 Windows 原生无焦点分层置顶窗口 (`WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`)，通过 GDI 双缓冲与色键透明穿透：
1. **大乱斗备选席胜率胶囊 (`BenchOverlay`)**：贴合游戏客户端顶部 10 个英雄头像下方，实时呈现 T1/T2 胜率与置换增益，支持点击一键抢英雄。
2. **排位征召全景辅助 (`RankedOverlay`)**：左右两侧展示双方分路胜率与对位克制，顶部提供一键写入 LCU 最优符文配置。
3. **局内三选一决策横幅 (`InGameOverlay`)**：按全局快捷键 `[` 或达到 7/11/15 级里程碑时瞬间置顶弹出推演卡片并附带耳机语音播报。
4. **左下角动态针对出装 (`ItemOverlay`)**：按快捷键 `]` 弹出，结合当局敌方实际装备推演重伤/穿透针对装，并实时抵扣已有散件金币。

---

## 🛠️ 构建与测试

本项目的所有编译、检查与单元测试均通过 **GitHub Actions** 自动化完成：
* 触发推送后，工作流会自动在 `windows-latest` 环境运行 `cargo check`、`cargo test` 与 `cargo build --release`。
* 编译产物 `lol-hex-rust.exe` 自动归档在 Actions Artifacts 中供随时下载。
