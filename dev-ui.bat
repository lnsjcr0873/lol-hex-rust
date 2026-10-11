@echo off
title HexBrain Desktop UI - Local Dev Mode
echo ========================================================
echo  HexBrain Desktop (League-Akari Style) 本地极速开发预览
echo ========================================================
echo 正在启动 Vite 极速热更新服务 (http://localhost:5173)...
cd /d "%~dp0\frontend"
npm run dev
