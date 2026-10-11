@echo off
chcp 65001 >nul
title HexBrain Engine - 英雄联盟海克斯与战绩透视引擎
cd /d "%~dp0"

echo ============================================================
echo   ⚡ HexBrain Engine (Rust 原生重构版) 正在启动... ⚡
echo ============================================================
echo.
echo [1/2] 正在加载嵌入式引擎与原生 Win32 HUD...
echo [2/2] 正在拉起 Web 看板服务: http://127.0.0.1:8765
echo.
echo 💡 提示: 
echo   - 浏览器大屏地址: http://127.0.0.1:8765
echo   - 快捷键: [ 或 【 呼出海克斯推演, ] 或 】 呼出装备克制
echo   - 游戏内请将视频设置设为【无边框窗口化】以获最佳体验
echo ============================================================

start "" http://127.0.0.1:8765

if exist "target\release\lol-hex-rust.exe" (
    "target\release\lol-hex-rust.exe"
) else if exist "dist\HexBrain-Rust-Windows-x64\lol-hex-rust.exe" (
    "dist\HexBrain-Rust-Windows-x64\lol-hex-rust.exe"
) else (
    cargo run --release
)

pause
