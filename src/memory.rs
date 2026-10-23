use crate::win_api;
use colored::*;
use sysinfo::{ProcessesToUpdate, System};

#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percentage: f32,
}

impl MemoryStats {
    pub fn current(sys: &mut System) -> Self {
        sys.refresh_memory();
        let total = sys.total_memory();
        let used = sys.used_memory();
        let available = sys.available_memory();
        let pct = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        Self {
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            usage_percentage: pct,
        }
    }

    pub fn to_mb(bytes: u64) -> f64 {
        bytes as f64 / 1024.0 / 1024.0
    }

    pub fn to_gb(bytes: u64) -> f64 {
        bytes as f64 / 1024.0 / 1024.0 / 1024.0
    }
}

#[allow(dead_code)]
pub struct OptimizationResult {
    pub before: MemoryStats,
    pub after: MemoryStats,
    pub freed_mb: f64,
    pub standby_purged: bool,
    pub processes_trimmed: usize,
    pub processes_failed: usize,
}

pub fn optimize_memory(verbose: bool) -> OptimizationResult {
    let mut sys = System::new_all();
    let before = MemoryStats::current(&mut sys);

    if verbose {
        println!("{}", "⚡ 開始執行快閃與實體記憶體深層釋放...".cyan().bold());
        println!(
            "  當前記憶體：已用 {:.2} GB / 總量 {:.2} GB ({:.1}%)",
            MemoryStats::to_gb(before.used_bytes),
            MemoryStats::to_gb(before.total_bytes),
            before.usage_percentage
        );
    }

    // Step 1: Trim working set of all accessible processes
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut trimmed_count = 0;
    let mut failed_count = 0;

    for (&pid, _process) in sys.processes() {
        let pid_u32 = pid.as_u32();
        if win_api::trim_process_working_set(pid_u32) {
            trimmed_count += 1;
        } else {
            failed_count += 1;
        }
    }

    // Also trim self
    unsafe {
        win_api::K32EmptyWorkingSet(win_api::GetCurrentProcess());
    }

    // Step 2: Global System Working Sets Flush
    let _ = win_api::empty_system_working_sets();

    // Step 3: Purge OS Standby List (Reclaims file cache memory)
    let standby_result = win_api::purge_standby_list();
    let standby_purged = standby_result.is_ok();

    if verbose {
        if standby_purged {
            println!("  {}", "✔ 成功清空 Windows 系統備用快取 (Standby List)".green());
        } else {
            println!(
                "  {}",
                "⚠ 備用快取清理略過（若需完整清空 Standby List，請以系統管理員權限執行本程式）".yellow()
            );
        }
        println!(
            "  {}",
            format!("✔ 成功修剪 {} 個應用程式之工作集 (Working Set)", trimmed_count).green()
        );
    }

    // Small sleep for OS metrics to refresh
    std::thread::sleep(std::time::Duration::from_millis(150));
    let after = MemoryStats::current(&mut sys);

    let freed_mb = if before.used_bytes > after.used_bytes {
        MemoryStats::to_mb(before.used_bytes - after.used_bytes)
    } else {
        0.0
    };

    if verbose {
        println!(
            "{}",
            format!(
                "✨ 記憶體最佳化完成！釋放了約 {:.1} MB 記憶體 (使用率從 {:.1}% 降至 {:.1}%)",
                freed_mb, before.usage_percentage, after.usage_percentage
            )
            .bright_green()
            .bold()
        );
    }

    OptimizationResult {
        before,
        after,
        freed_mb,
        standby_purged,
        processes_trimmed: trimmed_count,
        processes_failed: failed_count,
    }
}
