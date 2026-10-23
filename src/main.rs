mod daemon;
mod disk;
mod memory;
mod processes;
mod web;
mod win_api;

use clap::Parser;
use colored::*;
use std::io::{self, Write};
use sysinfo::System;

#[derive(Parser, Debug)]
#[command(
    name = "ai-cleaner",
    author = "AI Booster Team",
    version = "1.0.0",
    about = "專為 AI 推論與高負載打造的 Windows 記憶體與磁碟極速清理工具"
)]
struct Cli {
    /// 一鍵執行 AI 加速 (釋放記憶體 + 清理背景程序 + 磁碟暫存清理)
    #[arg(short = 'b', long = "boost")]
    boost: bool,

    /// 僅釋放記憶體 (修剪 Working Set + 清空 Standby List)
    #[arg(short = 'm', long = "mem")]
    mem: bool,

    /// 執行磁碟與快取清理
    #[arg(short = 'c', long = "clean")]
    clean: bool,

    /// 模擬預覽模式 (不實際刪除檔案)
    #[arg(short = 'd', long = "dry-run")]
    dry_run: bool,

    /// 關閉佔用記憶體之後台程式 (Steam, Discord, 閒置瀏覽器等)
    #[arg(short = 'k', long = "kill")]
    kill_bg: bool,

    /// 啟動 Web 網頁控制台
    #[arg(short = 'w', long = "web")]
    web: bool,

    /// Web 控制台通訊埠 (預設 8080)
    #[arg(long = "port", default_value = "8080")]
    port: u16,

    /// 啟動背景定時守護模式
    #[arg(long = "daemon")]
    daemon: bool,

    /// 定時守護模式之間隔 (分鐘，預設 30)
    #[arg(long = "interval", default_value = "30")]
    interval: u64,

    /// 定時守護觸發之記憶體門檻百分比 (預設 80.0%)
    #[arg(long = "threshold", default_value = "80.0")]
    threshold: f32,
}

fn print_banner() {
    println!("{}", "==================================================================".bright_blue());
    println!("{}", "   🚀 AI-Cleaner 極速記憶體釋放與磁碟深度優化工具 v1.0".bright_cyan().bold());
    println!("{}", "      專為本地 AI (LLM / Stable Diffusion / 深度學習) 釋放極限效能".white());
    println!("{}", "==================================================================".bright_blue());
}

fn print_dashboard() {
    let mut sys = System::new_all();
    let stats = memory::MemoryStats::current(&mut sys);

    println!("\n{}", "─── 🖥️  系統即時狀態 ────────────────────────────".cyan().bold());
    let pct_color = if stats.usage_percentage > 85.0 {
        format!("{:.1}% (嚴重吃緊)", stats.usage_percentage).red().bold()
    } else if stats.usage_percentage > 70.0 {
        format!("{:.1}% (偏高)", stats.usage_percentage).yellow().bold()
    } else {
        format!("{:.1}% (良好)", stats.usage_percentage).green()
    };

    println!(
        "  記憶體使用率: {} | 已用: {:.2} GB / 總量: {:.2} GB (剩餘 {:.2} GB)",
        pct_color,
        memory::MemoryStats::to_gb(stats.used_bytes),
        memory::MemoryStats::to_gb(stats.total_bytes),
        memory::MemoryStats::to_gb(stats.available_bytes)
    );

    println!();
    processes::print_top_processes(8);
    println!("{}", "──────────────────────────────────────────────────".cyan());
}

fn run_interactive_menu() {
    loop {
        print_banner();
        println!("【功能選單】");
        println!("  {} 🌐 啟動網頁儀表板控制台 (WebUI - 瀏覽器即時操作/手機連線)", "[W]".bright_cyan().bold());
        println!("  {} 一鍵 AI 極限加速 (記憶體修剪 + Standby 清空 + 釋放背景 + 磁碟暫存)", "[1]".green().bold());
        println!("  {} 快閃/實體記憶體釋放 (清空 Standby List 快取 + 工作集修剪)", "[2]".cyan().bold());
        println!("  {} 關閉佔用記憶體之非必要後台程式 (釋放 RAM 給 AI)", "[3]".yellow().bold());
        println!("  {} 系統磁碟與快取深度清理 (預覽或刪除暫存、更新包與瀏覽器快取)", "[4]".blue().bold());
        println!("  {} 啟動定時自動守護模式 (背景監控 RAM 並自動最佳化)", "[5]".magenta().bold());
        println!("  {} 查看系統狀態儀表板與 Top 記憶體佔用程序", "[6]".white().bold());
        println!("  {} 離開", "[0]".red().bold());
        print!("\n請輸入選項 [W/0-6]: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let choice = input.trim();
        println!();

        match choice {
            "w" | "W" | "7" => {
                web::start_web_server(8080, true);
            }
            "1" => {
                println!("{}", "⚡ 正在執行一鍵 AI 極限加速...".green().bold());
                processes::execute_ai_booster_kill(false, false);
                memory::optimize_memory(true);
                disk::clean_all_targets(false, true);
                println!("\n{}", "🎉 AI 加速與清理全數完成！現在可以順暢運行 AI 模型了！".bright_green().bold());
            }
            "2" => {
                memory::optimize_memory(true);
            }
            "3" => {
                println!("是否連同可能在後台閒置的瀏覽器 (Chrome / Edge) 也一併關閉？");
                print!("關閉瀏覽器後台？(y/N): ");
                let _ = io::stdout().flush();
                let mut ans = String::new();
                let _ = io::stdin().read_line(&mut ans);
                let close_browsers = ans.trim().eq_ignore_ascii_case("y");
                processes::execute_ai_booster_kill(close_browsers, false);
            }
            "4" => {
                println!("請選擇清理方式：");
                println!("  [1] 模擬預覽模式 (Dry-Run，僅統計可釋放空間與檔案數)");
                println!("  [2] 正式清理模式 (執行刪除，安全跳過被鎖定檔案)");
                print!("請輸入 [1-2]: ");
                let _ = io::stdout().flush();
                let mut sub = String::new();
                let _ = io::stdin().read_line(&mut sub);
                match sub.trim() {
                    "1" => {
                        disk::clean_all_targets(true, true);
                    }
                    "2" => {
                        disk::clean_all_targets(false, true);
                    }
                    _ => {
                        println!("{}", "無效輸入，取消操作。".yellow());
                    }
                }
            }
            "5" => {
                println!("請輸入守護檢查間隔時間（分鐘，直接按 Enter 預設為 30 分鐘）：");
                print!("間隔分鐘數 [預設 30]: ");
                let _ = io::stdout().flush();
                let mut int_input = String::new();
                let _ = io::stdin().read_line(&mut int_input);
                let mins: u64 = int_input.trim().parse().unwrap_or(30);

                println!("請輸入記憶體觸發門檻百分比（直接按 Enter 預設為 80%）：");
                print!("記憶體門檻 % [預設 80]: ");
                let _ = io::stdout().flush();
                let mut thresh_input = String::new();
                let _ = io::stdin().read_line(&mut thresh_input);
                let thresh: f32 = thresh_input.trim().parse().unwrap_or(80.0);

                daemon::start_daemon_loop(mins, thresh, false);
            }
            "6" => {
                print_dashboard();
            }
            "0" | "q" | "exit" => {
                println!("{}", "謝謝使用，再見！".bright_green());
                break;
            }
            _ => {
                println!("{}", "無效選項，請重新輸入！".red());
            }
        }

        println!("\n按 Enter 鍵返回主選單...");
        let mut pause = String::new();
        let _ = io::stdin().read_line(&mut pause);
        println!("\n{}\n", "=".repeat(60).dimmed());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // If no CLI flags were passed, enter interactive menu directly
    if args.len() <= 1 {
        run_interactive_menu();
        return;
    }

    let cli = Cli::parse();

    print_banner();

    if cli.web {
        web::start_web_server(cli.port, true);
        return;
    }

    if cli.daemon {
        daemon::start_daemon_loop(cli.interval, cli.threshold, cli.kill_bg);
        return;
    }

    if cli.boost {
        println!("{}", "⚡ 正在執行一鍵 AI 極限加速...".green().bold());
        processes::execute_ai_booster_kill(false, cli.dry_run);
        memory::optimize_memory(true);
        disk::clean_all_targets(cli.dry_run, true);
        return;
    }

    if cli.kill_bg {
        processes::execute_ai_booster_kill(false, cli.dry_run);
    }

    if cli.mem {
        memory::optimize_memory(true);
    }

    if cli.clean || cli.dry_run {
        disk::clean_all_targets(cli.dry_run, true);
    }
}
