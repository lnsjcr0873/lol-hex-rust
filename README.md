# HexBrain (Rust 原生微核版)

**英雄联盟海克斯大乱斗全量数据实时同步与决策辅助系统 (HexBrain Esports Edition)**

基于 1157万+ 局真实对局大数据驱动的海克斯大乱斗全流程实时决策支持系统，采用 **100% 原生 Rust** 高性能微核重构。
全面剔除本地庞大静态数据包袱与切屏 Web 看板，全面转向 **Hexdata 云端 API 实时驱动 + 局内原生免切屏 HUD** 体系。

---

## ⚡ 核心优势与性能表现

| 指标 | Python 旧版本 | Rust 原生微核版 | 提升幅度 |
| :--- | :--- | :--- | :--- |
| **可执行文件大小** | ~350MB (带环境) | **< 6MB** 独立微单文件 | **体积缩小 98%** |
| **本地数据占用** | ~300MB 本地静态缓存 | **0 磁盘文件依赖** (全内存热缓存) | **瘦身 99.5%** |
| **运行时内存占用** | 350MB ~ 600MB | **8MB ~ 15MB** | **内存降低 97%** |
| **启动耗时** | 2.5s ~ 4.0s | **< 15ms** 瞬开 | **快 150 倍** |
| **数据时效性** | 易过期，需经常同步 | **Hexdata 云端实时大数据直出** | 永远保持最新 |
| **用户体验** | 需切屏查看 Web | **100% 局内原生免激活置顶 HUD** | 0 切屏 / 0 干扰 |

---

## 🏛️ 系统架构设计

```text
lol-hex-rust/
├── Cargo.toml                  # 极简依赖清单与构建优化配置 (已剔除 Web 依赖)
├── .github/workflows/ci.yml    # GitHub Actions 自动化构建与测试工作流
├── recommendation_config.json  # 算法偏好预设 (SMART, HEX_FUN, IRON_COUNTER)
├── data/                       # 核心推演规则库
│   ├── engine_metadata.json    # 英雄流派画像、针对出装与梦幻联动规则库
│   └── recipe_metadata.json    # 装备合成树、散件抵扣与净金币费用表
└── src/
    ├── main.rs                 # 主程序入口、后台守护轮询与优雅停机
    ├── config.rs               # 算法偏好与冲突协调管理器
    ├── data/                   # 纯内存数据库与按需在线热缓存 (零磁盘写污染)
    ├── engine/                 # 威尔逊置信度、流派锁定、针对装与三选一推演
    ├── lcu/                    # 进程命令行嗅探、锁文件解析、LCU 与 2999 端口通讯
    ├── vision/                 # 超宽屏视口补偿、局部快速捕获与原生 WinRT OCR 识别
    ├── overlay/                # Win32 Layered Window 置顶 HUD、热键与语音播报
    └── crawler/                # Hexdata 云端 API 交互客户端与 OP.GG 排位天梯拉取
```

---

## 🎮 游戏内置顶 HUD 体系

采用 Windows 原生无焦点分层置顶窗口 (`WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`)，通过 GDI 双缓冲与色键透明穿透：
1. **大乱斗备选席胜率胶囊 (`BenchOverlay`)**：贴合游戏客户端顶部 10 个英雄头像下方，实时呈现候补英雄 T1/T2 评级与胜率，支持快捷换人。
2. **排位征召全景辅助 (`RankedOverlay`)**：选人阶段两侧展示双方对位克制榜、队友战术画像与一键配置最优符文。
3. **局内三选一决策横幅 (`InGameOverlay`)**：按快捷键 **`[`** 瞬间置顶弹出推演卡片并附带耳机语音播报。
4. **动态针对出装悬浮卡 (`ItemOverlay`)**：按快捷键 **`]`** 弹出，结合敌方实际出装推演针对装，并自动扣除已有散件净金币。

---

## 🛠️ 构建与测试

本项目的所有编译、检查与单元测试均通过 **GitHub Actions** 自动化完成：
* 触发推送后，工作流会自动在 `windows-latest` 环境运行 `cargo check`、`cargo test` 与 `cargo build --release`。
* 编译产物 `lol-hex-rust.exe` 自动归档在 Actions Artifacts 中供随时下载。
