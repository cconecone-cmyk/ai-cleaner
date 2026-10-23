use colored::*;
use sysinfo::{Pid, ProcessesToUpdate, Signal, System};

pub struct ProcessSummary {
    pub pid: u32,
    pub name: String,
    pub memory_bytes: u64,
}

// Protected system & AI dev processes that must never be terminated
const PROTECTED_PROCESSES: &[&str] = &[
    "system",
    "registry",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "services.exe",
    "lsass.exe",
    "winlogon.exe",
    "dwm.exe",
    "explorer.exe",
    "svchost.exe",
    "fontdrvhost.exe",
    "sihost.exe",
    "taskhostw.exe",
    "ctfmon.exe",
    "conhost.exe",
    "runtimebroker.exe",
    "spoolsv.exe",
    "audiodg.exe",
    // Dev & AI Tools
    "ai-cleaner.exe",
    "cargo.exe",
    "rustc.exe",
    "python.exe",
    "pythonw.exe",
    "ollama.exe",
    "ollama_llama_server.exe",
    "antigravity.exe",
    "code.exe", // VSCode
    "powershell.exe",
    "cmd.exe",
    "wt.exe", // Windows Terminal
];

// Common heavy background apps / game launchers / bloatware
const CANDIDATE_KILL_LIST: &[&str] = &[
    "steam.exe",
    "steamwebhelper.exe",
    "epicgameslauncher.exe",
    "battle.net.exe",
    "riotclientservices.exe",
    "discord.exe",
    "spotify.exe",
    "teams.exe",
    "msteams.exe",
    "slack.exe",
    "telegram.exe",
    "whatsapp.exe",
    "onedrive.exe",
    "phoneexperiencehost.exe",
    "searchapp.exe",
    "widgets.exe",
    "gamebar.exe",
    "gamebarft.exe",
    "xboxapp.exe",
    "yourphone.exe",
    "adobeupdateservice.exe",
    "creative cloud.exe",
    "ccleaner64.exe",
    "googleupdate.exe",
    "microsoftedgeupdate.exe",
];

pub fn get_top_memory_processes(limit: usize) -> Vec<ProcessSummary> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut list: Vec<ProcessSummary> = sys
        .processes()
        .iter()
        .map(|(&pid, p)| ProcessSummary {
            pid: pid.as_u32(),
            name: p.name().to_string_lossy().to_string(),
            memory_bytes: p.memory(),
        })
        .collect();

    // Sort descending by memory usage
    list.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    list.truncate(limit);
    list
}

pub fn print_top_processes(limit: usize) {
    let list = get_top_memory_processes(limit);
    println!("{}", format!("📊 記憶體佔用前 {} 名程式清單：", limit).cyan().bold());
    println!("{:<8} {:<30} {:>15}", "PID", "程式名稱", "記憶體使用量");
    println!("{}", "-".repeat(56));

    for p in list {
        let mb = p.memory_bytes as f64 / 1024.0 / 1024.0;
        let color_name = if mb > 500.0 {
            p.name.red().bold()
        } else if mb > 200.0 {
            p.name.yellow()
        } else {
            p.name.normal()
        };
        println!("{:<8} {:<30} {:>12.1} MB", p.pid, color_name, mb);
    }
}

pub struct KillPlan {
    pub targets: Vec<ProcessSummary>,
    pub total_memory_bytes: u64,
}

pub fn scan_killable_background_processes(include_browsers: bool) -> KillPlan {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut targets = Vec::new();
    let mut total_memory = 0u64;

    for (&pid, p) in sys.processes() {
        let name_lower = p.name().to_string_lossy().to_lowercase();

        // Check if protected
        if PROTECTED_PROCESSES.iter().any(|&prot| name_lower == prot) {
            continue;
        }

        let is_candidate = CANDIDATE_KILL_LIST.iter().any(|&cand| name_lower == cand);
        let is_browser = include_browsers
            && (name_lower == "chrome.exe"
                || name_lower == "msedge.exe"
                || name_lower == "brave.exe");

        if is_candidate || is_browser {
            let mem = p.memory();
            targets.push(ProcessSummary {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().to_string(),
                memory_bytes: mem,
            });
            total_memory += mem;
        }
    }

    KillPlan {
        targets,
        total_memory_bytes: total_memory,
    }
}

pub fn execute_ai_booster_kill(include_browsers: bool, dry_run: bool) -> (usize, u64) {
    let plan = scan_killable_background_processes(include_browsers);

    if plan.targets.is_empty() {
        println!("{}", "✔ 目前沒有檢測到高佔用之非必要背景程序。".green());
        return (0, 0);
    }

    let total_mb = plan.total_memory_bytes as f64 / 1024.0 / 1024.0;
    println!(
        "{}",
        format!(
            "🔍 發現 {} 個非必要背景程序，預計可回收約 {:.1} MB 記憶體：",
            plan.targets.len(),
            total_mb
        )
        .yellow()
        .bold()
    );

    for p in &plan.targets {
        let mb = p.memory_bytes as f64 / 1024.0 / 1024.0;
        println!("  - {:<24} (PID: {:<6}) : {:.1} MB", p.name, p.pid, mb);
    }

    if dry_run {
        println!("{}", "ℹ️ [Dry-Run 模式] 僅預覽掃描，未關閉任何程序。".blue());
        return (plan.targets.len(), plan.total_memory_bytes);
    }

    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut killed_count = 0;
    let mut reclaimed_bytes = 0u64;

    for target in plan.targets {
        let pid = Pid::from_u32(target.pid);
        if let Some(process) = sys.process(pid) {
            if process.kill_with(Signal::Kill).unwrap_or(false) {
                killed_count += 1;
                reclaimed_bytes += target.memory_bytes;
            }
        }
    }

    println!(
        "{}",
        format!(
            "🚀 成功關閉 {} 個背景程序，共為 AI 釋放了約 {:.1} MB 記憶體空間！",
            killed_count,
            reclaimed_bytes as f64 / 1024.0 / 1024.0
        )
        .green()
        .bold()
    );

    (killed_count, reclaimed_bytes)
}
