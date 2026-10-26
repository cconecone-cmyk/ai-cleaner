@echo off
chcp 65001 >nul
title AI-Cleaner Web控制台
echo ===================================================
echo 🚀 正在啟動 AI-Cleaner 網頁控制台...
echo 瀏覽器將會自動開啟：http://localhost:8080
echo (如需關閉控制台，請直接關閉此黑色視窗即可)
echo ===================================================
"%~dp0ai-cleaner.exe" --web
pause
