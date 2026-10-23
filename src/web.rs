use crate::{disk, memory, processes};
use colored::*;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sysinfo::System;
use tiny_http::{Header, Response, Server, StatusCode};

#[derive(Serialize, Deserialize)]
struct StatusResponse {
    total_gb: f64,
    used_gb: f64,
    available_gb: f64,
    usage_percent: f32,
    top_processes: Vec<ProcessItem>,
}

#[derive(Serialize, Deserialize)]
struct ProcessItem {
    pid: u32,
    name: String,
    memory_mb: f64,
}

#[derive(Serialize, Deserialize)]
struct ActionResponse {
    success: bool,
    message: String,
    freed_mb: f64,
}

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-TW">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AI-Cleaner 智慧記憶體與系統加速控制台</title>
    <style>
        :root {
            --bg-primary: #0f172a;
            --bg-card: rgba(30, 41, 59, 0.7);
            --border-color: rgba(255, 255, 255, 0.1);
            --text-primary: #f8fafc;
            --text-secondary: #94a3b8;
            --accent-cyan: #06b6d4;
            --accent-green: #10b981;
            --accent-purple: #8b5cf6;
            --accent-red: #ef4444;
            --accent-orange: #f59e0b;
        }
        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
        }
        body {
            background-color: var(--bg-primary);
            background-image: 
                radial-gradient(at 0% 0%, rgba(6, 182, 212, 0.15) 0px, transparent 50%),
                radial-gradient(at 100% 100%, rgba(139, 92, 246, 0.15) 0px, transparent 50%);
            min-height: 100vh;
            color: var(--text-primary);
            padding: 24px 16px;
        }
        .container {
            max-width: 900px;
            margin: 0 auto;
        }
        header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 24px;
            padding-bottom: 16px;
            border-bottom: 1px solid var(--border-color);
        }
        .logo-area h1 {
            font-size: 24px;
            font-weight: 700;
            display: flex;
            align-items: center;
            gap: 10px;
            background: linear-gradient(135deg, #38bdf8, #818cf8);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        .badge {
            background: rgba(16, 185, 129, 0.2);
            color: var(--accent-green);
            padding: 4px 10px;
            border-radius: 999px;
            font-size: 12px;
            font-weight: 600;
            border: 1px solid rgba(16, 185, 129, 0.3);
        }
        .grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin-bottom: 24px;
        }
        @media (max-width: 768px) {
            .grid { grid-template-columns: 1fr; }
        }
        .card {
            background: var(--bg-card);
            backdrop-filter: blur(12px);
            border: 1px solid var(--border-color);
            border-radius: 16px;
            padding: 20px;
            box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.3);
        }
        .card-title {
            font-size: 16px;
            font-weight: 600;
            color: var(--text-secondary);
            margin-bottom: 16px;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .ram-visual {
            display: flex;
            align-items: center;
            justify-content: space-around;
            padding: 10px 0;
        }
        .circle-gauge {
            position: relative;
            width: 140px;
            height: 140px;
        }
        svg.gauge-svg {
            transform: rotate(-90deg);
            width: 140px;
            height: 140px;
        }
        .gauge-bg {
            fill: none;
            stroke: rgba(255, 255, 255, 0.08);
            stroke-width: 12;
        }
        .gauge-fill {
            fill: none;
            stroke: var(--accent-cyan);
            stroke-width: 12;
            stroke-linecap: round;
            stroke-dasharray: 377;
            stroke-dashoffset: 100;
            transition: stroke-dashoffset 0.6s ease, stroke 0.4s ease;
        }
        .gauge-text {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            text-align: center;
        }
        .gauge-percent {
            font-size: 28px;
            font-weight: 800;
            color: var(--text-primary);
        }
        .gauge-label {
            font-size: 12px;
            color: var(--text-secondary);
        }
        .ram-details {
            display: flex;
            flex-direction: column;
            gap: 12px;
        }
        .stat-item {
            display: flex;
            flex-direction: column;
        }
        .stat-label {
            font-size: 12px;
            color: var(--text-secondary);
        }
        .stat-val {
            font-size: 18px;
            font-weight: 700;
        }
        /* Buttons area */
        .actions-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 12px;
        }
        .btn {
            border: none;
            border-radius: 12px;
            padding: 14px 16px;
            font-size: 15px;
            font-weight: 600;
            color: white;
            cursor: pointer;
            transition: all 0.2s ease;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
        }
        .btn:hover {
            transform: translateY(-2px);
            filter: brightness(1.1);
        }
        .btn:active {
            transform: translateY(0);
        }
        .btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
            transform: none;
        }
        .btn-boost {
            grid-column: span 2;
            background: linear-gradient(135deg, #06b6d4, #3b82f6);
            font-size: 17px;
            padding: 16px;
            box-shadow: 0 4px 20px rgba(6, 182, 212, 0.4);
        }
        .btn-mem {
            background: rgba(139, 92, 246, 0.2);
            border: 1px solid rgba(139, 92, 246, 0.4);
            color: #c4b5fd;
        }
        .btn-disk {
            background: rgba(16, 185, 129, 0.2);
            border: 1px solid rgba(16, 185, 129, 0.4);
            color: #6ee7b7;
        }
        .btn-kill {
            grid-column: span 2;
            background: rgba(245, 158, 11, 0.15);
            border: 1px solid rgba(245, 158, 11, 0.4);
            color: #fcd34d;
        }
        /* Process table */
        table {
            width: 100%;
            border-collapse: collapse;
            font-size: 13px;
        }
        th, td {
            padding: 10px 8px;
            text-align: left;
        }
        th {
            color: var(--text-secondary);
            font-weight: 600;
            border-bottom: 1px solid var(--border-color);
        }
        tr:not(:last-child) td {
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        }
        .pid-tag {
            color: var(--text-secondary);
            font-family: monospace;
        }
        .mem-tag {
            font-weight: 700;
            color: #38bdf8;
            text-align: right;
        }
        /* Toast notification */
        #toast {
            position: fixed;
            bottom: 24px;
            right: 24px;
            background: #1e293b;
            border: 1px solid var(--accent-cyan);
            color: white;
            padding: 14px 20px;
            border-radius: 12px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.5);
            display: none;
            align-items: center;
            gap: 10px;
            z-index: 1000;
            font-weight: 500;
            animation: slideIn 0.3s ease;
        }
        @keyframes slideIn {
            from { transform: translateY(20px); opacity: 0; }
            to { transform: translateY(0); opacity: 1; }
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <div class="logo-area">
                <h1>⚡ AI-Cleaner 控制台</h1>
            </div>
            <div class="badge">線上即時監控</div>
        </header>

        <div class="grid">
            <!-- Memory Gauge Card -->
            <div class="card">
                <div class="card-title">🧠 記憶體即時狀態</div>
                <div class="ram-visual">
                    <div class="circle-gauge">
                        <svg class="gauge-svg" viewBox="0 0 140 140">
                            <circle class="gauge-bg" cx="70" cy="70" r="60"></circle>
                            <circle class="gauge-fill" id="gaugeCircle" cx="70" cy="70" r="60"></circle>
                        </svg>
                        <div class="gauge-text">
                            <div class="gauge-percent" id="gaugePct">0%</div>
                            <div class="gauge-label">已使用</div>
                        </div>
                    </div>
                    <div class="ram-details">
                        <div class="stat-item">
                            <span class="stat-label">已用記憶體</span>
                            <span class="stat-val" id="usedGb">0.00 GB</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-label">可用記憶體</span>
                            <span class="stat-val" id="availGb" style="color: var(--accent-green);">0.00 GB</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-label">總實體記憶體</span>
                            <span class="stat-val" id="totalGb">0.00 GB</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- One-Click Action Card -->
            <div class="card">
                <div class="card-title">🚀 效能加速動作</div>
                <div class="actions-grid">
                    <button class="btn btn-boost" onclick="triggerAction('/api/boost', '⚡ 正在執行一鍵 AI 極限加速...')">
                        ⚡ 一鍵 AI 極速釋放
                    </button>
                    <button class="btn btn-mem" onclick="triggerAction('/api/trim-mem', '🧠 正在修剪記憶體工作集與備用快取...')">
                        🧠 實體記憶體釋放
                    </button>
                    <button class="btn btn-disk" onclick="triggerAction('/api/clean-disk', '🧹 正在深度清理系統與快取暫存...')">
                        🧹 磁碟暫存清理
                    </button>
                    <button class="btn btn-kill" onclick="triggerAction('/api/kill-bg', '🛑 正在關閉非必要背景程式...')">
                        🛑 關閉吃 RAM 後台程式
                    </button>
                </div>
            </div>
        </div>

        <!-- Top Processes Card -->
        <div class="card">
            <div class="card-title">📊 記憶體佔用前 6 名程序</div>
            <table>
                <thead>
                    <tr>
                        <th>PID</th>
                        <th>程序名稱</th>
                        <th style="text-align: right;">記憶體使用量</th>
                    </tr>
                </thead>
                <tbody id="processTableBody">
                    <tr><td colspan="3" style="text-align: center; color: var(--text-secondary);">載入中...</td></tr>
                </tbody>
            </table>
        </div>
    </div>

    <div id="toast"></div>

    <script>
        const circumference = 2 * Math.PI * 60; // ~377

        function showToast(text, isSuccess = true) {
            const toast = document.getElementById('toast');
            toast.innerText = text;
            toast.style.borderColor = isSuccess ? 'var(--accent-green)' : 'var(--accent-red)';
            toast.style.display = 'flex';
            setTimeout(() => { toast.style.display = 'none'; }, 4000);
        }

        async function fetchStatus() {
            try {
                const res = await fetch('/api/status');
                const data = await res.json();

                // Update RAM gauge
                const pct = data.usage_percent;
                document.getElementById('gaugePct').innerText = `${pct.toFixed(1)}%`;
                document.getElementById('usedGb').innerText = `${data.used_gb.toFixed(2)} GB`;
                document.getElementById('availGb').innerText = `${data.available_gb.toFixed(2)} GB`;
                document.getElementById('totalGb').innerText = `${data.total_gb.toFixed(2)} GB`;

                const circle = document.getElementById('gaugeCircle');
                const offset = circumference - (pct / 100) * circumference;
                circle.style.strokeDashoffset = offset;

                if (pct > 85) circle.style.stroke = 'var(--accent-red)';
                else if (pct > 70) circle.style.stroke = 'var(--accent-orange)';
                else circle.style.stroke = 'var(--accent-cyan)';

                // Update Process table
                const tbody = document.getElementById('processTableBody');
                tbody.innerHTML = data.top_processes.map(p => `
                    <tr>
                        <td class="pid-tag">${p.pid}</td>
                        <td><strong>${p.name}</strong></td>
                        <td class="mem-tag">${p.memory_mb.toFixed(1)} MB</td>
                    </tr>
                `).join('');
            } catch (err) {
                console.error(err);
            }
        }

        async function triggerAction(endpoint, loadingText) {
            showToast(loadingText);
            const btns = document.querySelectorAll('button');
            btns.forEach(b => b.disabled = true);
            try {
                const res = await fetch(endpoint, { method: 'POST' });
                const json = await res.json();
                showToast(json.message, json.success);
                await fetchStatus();
            } catch (e) {
                showToast("執行失敗，請確認伺服器連線", false);
            } finally {
                btns.forEach(b => b.disabled = false);
            }
        }

        setInterval(fetchStatus, 2000);
        fetchStatus();
    </script>
</body>
</html>
"#;

pub fn start_web_server(port: u16, open_browser: bool) {
    let addr = format!("0.0.0.0:{}", port);
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", format!("❌ 無法在 {} 啟動 Web 伺服器: {}", addr, e).red());
            return;
        }
    };

    let local_url = format!("http://localhost:{}", port);
    println!("{}", "==================================================================".bright_blue());
    println!("{}", "🌐 AI-Cleaner Web 控制台已成功啟動！".bright_green().bold());
    println!("  - 本機訪問網址：{}", local_url.bright_cyan().underline());
    println!("  - 同網域裝置（如手機/其他電腦）：http://<電腦區域IP>:{}", port);
    println!("  - 隨時按下 [Ctrl + C] 即可關閉 Web 伺服器");
    println!("{}", "==================================================================".bright_blue());

    if open_browser {
        let _ = Command::new("cmd").args(["/c", "start", &local_url]).spawn();
    }

    let running = Arc::new(AtomicBool::new(true));

    for request in server.incoming_requests() {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        let url = request.url().to_string();
        let method = request.method().as_str().to_uppercase();

        match (method.as_str(), url.as_str()) {
            ("GET", "/") | ("GET", "/index.html") => {
                let response = Response::from_string(DASHBOARD_HTML)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap());
                let _ = request.respond(response);
            }
            ("GET", "/api/status") => {
                let mut sys = System::new_all();
                let stats = memory::MemoryStats::current(&mut sys);
                let top_p = processes::get_top_memory_processes(6);

                let status = StatusResponse {
                    total_gb: memory::MemoryStats::to_gb(stats.total_bytes),
                    used_gb: memory::MemoryStats::to_gb(stats.used_bytes),
                    available_gb: memory::MemoryStats::to_gb(stats.available_bytes),
                    usage_percent: stats.usage_percentage,
                    top_processes: top_p
                        .into_iter()
                        .map(|p| ProcessItem {
                            pid: p.pid,
                            name: p.name,
                            memory_mb: p.memory_bytes as f64 / 1024.0 / 1024.0,
                        })
                        .collect(),
                };

                let json = serde_json::to_string(&status).unwrap_or_default();
                let response = Response::from_string(json)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                let _ = request.respond(response);
            }
            ("POST", "/api/boost") => {
                let (_, killed_bytes) = processes::execute_ai_booster_kill(false, false);
                let mem_res = memory::optimize_memory(false);
                let disk_res = disk::clean_all_targets(false, false);

                let total_freed_mb = mem_res.freed_mb
                    + (killed_bytes as f64 / 1024.0 / 1024.0)
                    + (disk_res.total_bytes_freed as f64 / 1024.0 / 1024.0);

                let action_res = ActionResponse {
                    success: true,
                    message: format!(
                        "⚡ 一鍵 AI 加速成功！釋放約 {:.1} MB 記憶體與暫存！",
                        total_freed_mb
                    ),
                    freed_mb: total_freed_mb,
                };

                let json = serde_json::to_string(&action_res).unwrap_or_default();
                let response = Response::from_string(json)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                let _ = request.respond(response);
            }
            ("POST", "/api/trim-mem") => {
                let mem_res = memory::optimize_memory(false);
                let action_res = ActionResponse {
                    success: true,
                    message: format!(
                        "🧠 記憶體釋放成功！修剪了 {} 個程序，釋放約 {:.1} MB 記憶體！",
                        mem_res.processes_trimmed, mem_res.freed_mb
                    ),
                    freed_mb: mem_res.freed_mb,
                };
                let json = serde_json::to_string(&action_res).unwrap_or_default();
                let response = Response::from_string(json)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                let _ = request.respond(response);
            }
            ("POST", "/api/clean-disk") => {
                let disk_res = disk::clean_all_targets(false, false);
                let freed_mb = disk_res.total_bytes_freed as f64 / 1024.0 / 1024.0;
                let action_res = ActionResponse {
                    success: true,
                    message: format!(
                        "🧹 磁碟暫存清理完成！刪除 {} 個檔案，釋放約 {:.1} MB 空間！",
                        disk_res.total_files_deleted, freed_mb
                    ),
                    freed_mb,
                };
                let json = serde_json::to_string(&action_res).unwrap_or_default();
                let response = Response::from_string(json)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                let _ = request.respond(response);
            }
            ("POST", "/api/kill-bg") => {
                let (count, bytes) = processes::execute_ai_booster_kill(false, false);
                let freed_mb = bytes as f64 / 1024.0 / 1024.0;
                let action_res = ActionResponse {
                    success: true,
                    message: format!(
                        "🛑 成功關閉 {} 個後台程式，回收約 {:.1} MB 記憶體！",
                        count, freed_mb
                    ),
                    freed_mb,
                };
                let json = serde_json::to_string(&action_res).unwrap_or_default();
                let response = Response::from_string(json)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                let _ = request.respond(response);
            }
            _ => {
                let response = Response::new(
                    StatusCode(404),
                    vec![],
                    Cursor::new("Not Found"),
                    Some(9),
                    None,
                );
                let _ = request.respond(response);
            }
        }
    }
}
