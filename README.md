# 🚀 AI-Cleaner 極速記憶體釋放與磁碟深度清理工具 v1.0

> 專為本地 AI 運算（LLM、Stable Diffusion、PyTorch、Ollama）打造的輕量、極速、零廣告原生 Rust 系統優化與記憶體釋放工具。
> 支援 **桌面 CLI 選單** 與 **現代化 WebUI 網頁儀表板控制台**！

---

## 🌟 核心特色

1. **🌐 現代化 WebUI 控制台（瀏覽器操作 / 支援手機區網連線）**
   - 內建極輕量 Web 伺服器，直接在瀏覽器開啟漂亮的即時動態儀表板（`http://localhost:8080`）。
   - **即時記憶體儀表盤**：動態展示 RAM 使用百分比、已用量、可用量與總量（每 2 秒自動刷新）。
   - **耗電/耗記憶體前 6 名程序清單**：精準掌控背景程式資源消耗。
   - **一鍵觸發按鈕**：直覺點擊「⚡ 一鍵 AI 加速」、「🧠 實體記憶體釋放」、「🧹 磁碟暫存清理」、「🛑 關閉吃 RAM 後台程式」。

2. **⚡ 一鍵 AI 極限加速 (AI Booster Mode)**
   - 自動修剪數百個應用程式之工作集 (Working Set)。
   - 清空 Windows 系統備用快取 (Standby List / File Cache Purge)。
   - 智慧掃描並關閉常駐背景吃 RAM 的非必要程序（如 Steam、Discord、Xbox、Widgets、閒置瀏覽器）。
   - 快速清理系統暫存與垃圾檔案。

3. **🧠 快閃 / 實體記憶體深層釋放 (RAM Optimizer)**
   - 呼叫 Windows 底層 Native API（`EmptyWorkingSet` & `NtSetSystemInformation`）。
   - 實測單次可瞬間釋放 **500 MB ~ 2 GB+** 的記憶體空間給 AI 模型使用。

4. **🧹 磁碟與快取深度清理 (Disk & Cache Cleaner)**
   - 支援 **Dry-Run 模擬預覽模式**（先統計可省空間與檔案數，絕不盲目刪除）。
   - 精準清理：使用者暫存 (`%TEMP%`)、系統暫存 (`C:\Windows\Temp`)、Windows 更新下載快取 (`SoftwareDistribution\Download`)、當機傾印檔 (`CrashDumps`)、Pip 快取。
   - 瀏覽器快取清理**完整保留 Cookie 與登入狀態**，不影響日常生活。

5. **⏱️ 定時背景自動守護模式 (Daemon Watchdog)**
   - 可自訂檢查頻率（如每 15/30 分鐘）與記憶體觸發門檻（如 80%）。
   - 當記憶體吃緊時自動釋放，避免 AI 推論時因 OOM (Out Of Memory) 崩潰或卡頓。

6. **🛡️ 系統安全白名單保護**
   - 嚴格保護 Windows 核心進程 (`csrss`, `dwm`, `explorer`, `svchost`) 與 AI 開發工具 (`python`, `ollama`, `cargo`, `code`)，絕不誤殺關鍵程式。

---

## 💻 使用方式

### 🌐 方式一：啟動 WebUI 網頁控制台（最推薦！）
直接在終端機輸入：
```powershell
.\ai-cleaner.exe --web
```
或雙擊執行檔輸入 `W`，系統會**自動在你的瀏覽器中開啟控制台**（`http://localhost:8080`），你可以用滑鼠點擊按鈕一鍵清理！
同 Wi-Fi 區域網路下的手機或另一台電腦，也可以開啟 `http://<你的電腦IP>:8080` 進行遠端監控與釋放！

---

### 🚀 方式二：PowerShell 線上一鍵執行（免手動安裝）
在任何 Windows 電腦上打開 PowerShell，貼上指令即可線上啟動：
```powershell
irm https://raw.githubusercontent.com/cconecone-cmyk/ai-cleaner/main/run.ps1 | iex
```

---

### ⌨️ 方式三：命令列 (CLI) 快速調用
可在終端機中透過參數直接執行特定任務：

```powershell
# 1. 一鍵 AI 加速 (釋放記憶體 + 清理背景 + 磁碟清理)
.\ai-cleaner.exe --boost

# 2. 僅釋放記憶體 (極速修剪 RAM)
.\ai-cleaner.exe --mem

# 3. 磁碟清理模擬預覽 (Dry-Run，只看能清多少，不刪除)
.\ai-cleaner.exe --dry-run

# 4. 正式清理磁碟暫存
.\ai-cleaner.exe --clean

# 5. 關閉佔用記憶體之背景程序
.\ai-cleaner.exe --kill

# 6. 啟動定時背景守護模式 (每 30 分鐘檢查，RAM > 80% 自動觸發)
.\ai-cleaner.exe --daemon --interval 30 --threshold 80
```

> [!TIP]
> 若要完整清空 Windows 系統層級的 Standby List 備用快取，建議**以系統管理員身分執行**。
