# HexBrain (Rust 原生微核 & Tauri 桌面版)

**英雄联盟海克斯大乱斗全量数据实时同步与决策辅助系统 (HexBrain Esports Edition)**

基于 1157万+ 局真实对局大数据驱动的海克斯大乱斗与排位全流程实时决策支持系统，采用 **Rust 核心引擎 + Tauri v2 桌面容器 + Vue 3 / TypeScript 前端 + Win32 原生置顶 HUD** 的高性能混合架构。

---

## ⚡ 核心优势与架构特色

| 指标 / 特性 | 说明 |
| :--- | :--- |
| **双模式自由运行** | 支持 **Tauri 2.0 现代桌面客户端** 与 **极简无头微核 CLI / 内嵌 Web 控制中心** |
| **原生零切屏置顶 HUD** | 选人阶段大乱斗备战席评级胶囊、排位 10 人全景对位克制榜、局内快捷键海克斯三选一横幅与自适应出装悬浮卡 |
| **轻量高性能** | 核心采用全异步 Tokio 运行时，支持无子进程瞬态嗅探与 Win32 GDI 硬件级透明穿透，低 CPU / 零掉帧 |
| **数据与规则引擎** | 整合 CommunityDragon / SGP 权威战绩、OP.GG 权威天梯胜率与 Hexdata 海量实战推演大数据 |
| **全自动化辅助** | 对局就绪自动接受（支持防挂机安全延迟）、大乱斗心愿英雄秒抢（防抖冷却保护）、版本最优符文与召唤师技能秒级配置 |

---

## 🏛️ 系统架构设计

```text
lol-hex-rust/
├── Cargo.toml                  # Rust 核心引擎依赖清单与构建优化配置
├── recommendation_config.json  # 算法偏好预设 (SMART, HEX_FUN, IRON_COUNTER)
├── data/                       # 核心推演规则库 (英雄画像、装备合成树、散件抵扣费用表)
├── src/                        # Rust 核心库与 CLI 宿主
│   ├── main.rs                 # 命令行无头守护入口与 Axum 内嵌 Web 服务启动器
│   ├── lib.rs                  # 核心库导出与跨线程 Tokio 运行时分发中心
│   ├── runtime.rs              # 统一后台引擎宿主 (LCU 监控、局内 2999 探针、置顶 HUD 与全局热键)
│   ├── config.rs               # 算法偏好与自动化配置管理器 (原子读写与落盘防损坏保护)
│   ├── data/                   # 纯内存数据库与按需在线热缓存
│   ├── engine/                 # 威尔逊置信度算法、流派判定、克制推演与 SGP 战绩聚合器
│   ├── lcu/                    # 进程命令行嗅探、锁文件解析、LCU 与 2999 实时端口通讯
│   ├── vision/                 # 视口补偿、局部快速捕获与原生 WinRT OCR 识别
│   ├── overlay/                # Win32 Layered Window 置顶 HUD、热键与语音播报
│   ├── crawler/                # Hexdata 云端 API 交互客户端与 OP.GG 排位天梯拉取
│   ├── sgp/                    # SGP 官方分布式网关客户端 (战绩/挑战/资产查询)
│   └── web/                    # 内嵌轻量 Web 控制台路由与 WebSocket 状态广播器
├── src-tauri/                  # Tauri v2 桌面客户端宿主
│   ├── Cargo.toml              # Tauri 依赖与 Windows 清单配置
│   ├── tauri.conf.json         # 桌面窗口尺寸、安全白名单 CSP 与权限定义
│   └── src/
│       ├── main.rs             # Tauri 桌面应用可执行文件入口
│       ├── lib.rs              # Tauri 状态挂载与生命周期管理
│       └── cmd.rs              # 前后端 IPC 命令路由桥接层
└── frontend/                   # Vue 3 + TypeScript + Tailwind CSS 现代前端
    ├── package.json            # 前端依赖配置 (Pinia, Lucide, Vitest)
    └── src/                    # 页面组件、战绩复盘、雷达图、对局实时面板与设置视图
```

---

## 🎮 游戏内置顶 HUD 与全局热键

采用 Windows 原生无焦点分层置顶窗口 (`WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`)，通过 GDI 双缓冲与色键透明穿透：
1. **大乱斗备选席胜率胶囊 (`BenchOverlay`)**：贴合游戏客户端顶部英雄头像下方，实时呈现候补英雄 T1/T2 评级与胜率，支持秒抢心愿英雄。
2. **排位征召全景辅助 (`RankedOverlay`)**：选人阶段两侧展示双方对位克制榜、队友战术画像与一键配置版本最优符文。
3. **局内三选一决策横幅 (`InGameOverlay`)**：按快捷键 **`[`** 瞬间置顶弹出推演卡片并附带耳机语音播报。
4. **动态针对出装悬浮卡 (`ItemOverlay`)**：按快捷键 **`]`** 弹出，结合敌方实际出装推演针对装，并自动扣除已有散件净金币。

---

## 🛠️ 构建与测试

本项目的所有编译、检查与单元测试均通过 **GitHub Actions** 自动化完成：
* **桌面端打包工作流** (`.github/workflows/build-tauri.yml`)：构建前端资源并由 Tauri 打包为独立的 Windows 安装包与免安装单文件应用。
* **CI 验证工作流** (`.github/workflows/ci.yml`)：在 `windows-latest` 环境运行前端构建、`cargo check`、`cargo test` 与 `cargo build --release`。
