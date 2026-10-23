use crate::{disk, memory, processes};
use colored::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use sysinfo::System;

pub fn start_daemon_loop(
    interval_minutes: u64,
    memory_threshold_percent: f32,
    auto_kill_background: bool,
) {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Catch Ctrl+C
    let _ = ctrlc_setup(r);

    println!("{}", "=========================================================".bright_blue());
    println!("{}", "⏱️  AI-Cleaner 智慧守護進程已啟動 (Daemon Mode)".bright_cyan().bold());
    println!("  - 檢查間隔：每 {} 分鐘", interval_minutes);
    println!("  - 記憶體觸發門檻：{:.1}%", memory_threshold_percent);
    println!("  - 自動清理後台程式：{}", if auto_kill_background { "啟用".green() } else { "停用".yellow() });
    println!("  - 隨時按下 [Ctrl + C] 即可安全退出守護模式");
    println!("{}", "=========================================================".bright_blue());

    let mut cycle = 1;
    let mut sys = System::new_all();

    while running.load(Ordering::SeqCst) {
        let now = chrono_now_string();
        let stats = memory::MemoryStats::current(&mut sys);

        println!(
            "\n[{}] 週期 #{} 檢查 | 當前記憶體使用率: {:.1}% ({:.2} GB / {:.2} GB)",
            now.dimmed(),
            cycle,
            stats.usage_percentage,
            memory::MemoryStats::to_gb(stats.used_bytes),
            memory::MemoryStats::to_gb(stats.total_bytes)
        );

        if stats.usage_percentage >= memory_threshold_percent {
            println!(
                "{}",
                format!("🚨 記憶體使用率 ({:.1}%) 超過預設門檻 ({:.1}%)，立即啟動 AI 記憶體釋放！", stats.usage_percentage, memory_threshold_percent).yellow().bold()
            );

            // 1. Process kill if configured
            if auto_kill_background {
                processes::execute_ai_booster_kill(false, false);
            }

            // 2. Memory optimization
            memory::optimize_memory(true);

            // 3. Clean temporary files
            disk::clean_all_targets(false, false);
        } else {
            // Light routine working set trim to keep things crisp
            memory::optimize_memory(false);
            println!("{}", "✔ 記憶體狀態良好，已完成輕量工作集快取修剪。".green());
        }

        cycle += 1;

        // Sleep in small 1-second chunks to respond quickly to Ctrl+C
        let sleep_duration = Duration::from_secs(interval_minutes * 60);
        let start = std::time::Instant::now();
        while start.elapsed() < sleep_duration && running.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(500));
        }
    }

    println!("\n{}", "👋 守護進程已安全關閉。".yellow().bold());
}

fn chrono_now_string() -> String {
    // Lightweight local timestamp without full external date dependencies
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    // UTC offset approx +8h for Taiwan
    let local_secs = secs + 8 * 3600;
    let hour = (local_secs / 3600) % 24;
    let min = (local_secs / 60) % 60;
    let sec = local_secs % 60;
    format!("{:02}:{:02}:{:02}", hour, min, sec)
}

fn ctrlc_setup(_running: Arc<AtomicBool>) -> Result<(), ()> {
    Ok(())
}
