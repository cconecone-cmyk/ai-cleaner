use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct CleanTarget {
    pub name: &'static str,
    pub path: PathBuf,
    pub description: &'static str,
}

#[derive(Default, Debug)]
pub struct DiskCleanReport {
    pub total_files_scanned: u64,
    pub total_files_deleted: u64,
    pub total_bytes_freed: u64,
    pub total_errors: u64,
}

pub fn get_default_targets() -> Vec<CleanTarget> {
    let mut targets = Vec::new();

    // 1. User Temp
    if let Ok(temp) = std::env::var("TEMP") {
        targets.push(CleanTarget {
            name: "使用者暫存檔 (User Temp)",
            path: PathBuf::from(temp),
            description: "應用程式運行產生的暫存資料與日誌",
        });
    }

    // 2. System Temp
    targets.push(CleanTarget {
        name: "Windows 系統暫存 (System Temp)",
        path: PathBuf::from(r"C:\Windows\Temp"),
        description: "系統層級安裝與維護殘留之暫存檔",
    });

    // 3. Windows Update Download Cache
    targets.push(CleanTarget {
        name: "Windows 更新下載快取",
        path: PathBuf::from(r"C:\Windows\SoftwareDistribution\Download"),
        description: "已安裝完成之系統更新安裝包檔案",
    });

    // 4. Local AppData CrashDumps
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(&local_app_data);
        targets.push(CleanTarget {
            name: "系統當機傾印檔 (Crash Dumps)",
            path: p.join("CrashDumps"),
            description: "應用程式崩潰時產生的除錯傾印檔案",
        });
        targets.push(CleanTarget {
            name: "Windows 錯誤回報暫存 (WER)",
            path: p.join(r"Microsoft\Windows\WER"),
            description: "系統自動生成的錯誤報告佇列快取",
        });
        // 5. Browser Caches (Only cache folders, cookies & logins are safely preserved!)
        targets.push(CleanTarget {
            name: "Google Chrome 快取 (保留 Cookie)",
            path: p.join(r"Google\Chrome\User Data\Default\Cache"),
            description: "網頁暫存快取（不影響登入狀態與歷史紀錄）",
        });
        targets.push(CleanTarget {
            name: "Google Chrome 程式碼快取",
            path: p.join(r"Google\Chrome\User Data\Default\Code Cache"),
            description: "已編譯之 JavaScript/WASM 網頁快取",
        });
        targets.push(CleanTarget {
            name: "Microsoft Edge 快取 (保留 Cookie)",
            path: p.join(r"Microsoft\Edge\User Data\Default\Cache"),
            description: "Edge 瀏覽器暫存快取（不影響登入狀態）",
        });
        targets.push(CleanTarget {
            name: "Pip Python 套件安裝快取",
            path: p.join(r"pip\cache"),
            description: "Python pip 下載暫存的 whl 安裝包",
        });
    }

    // Filter only existing paths
    targets.into_iter().filter(|t| t.path.exists()).collect()
}

fn clean_path_recursive(
    dir: &Path,
    dry_run: bool,
    report: &mut DiskCleanReport,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => {
            report.total_errors += 1;
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => {
                report.total_errors += 1;
                continue;
            }
        };

        if metadata.is_dir() {
            clean_path_recursive(&path, dry_run, report);
            if !dry_run {
                // Try removing empty directory, ignore error if locked or not empty
                let _ = fs::remove_dir(&path);
            }
        } else if metadata.is_file() {
            let size = metadata.len();
            report.total_files_scanned += 1;

            if dry_run {
                report.total_bytes_freed += size;
            } else {
                match fs::remove_file(&path) {
                    Ok(_) => {
                        report.total_files_deleted += 1;
                        report.total_bytes_freed += size;
                    }
                    Err(_) => {
                        // File is currently in use/locked by OS, safely skip
                        report.total_errors += 1;
                    }
                }
            }
        }
    }
}

pub fn clean_all_targets(dry_run: bool, verbose: bool) -> DiskCleanReport {
    let targets = get_default_targets();
    let mut grand_report = DiskCleanReport::default();

    if verbose {
        let mode_tag = if dry_run {
            "[Dry-Run 模擬預覽模式]".yellow()
        } else {
            "[執行清理模式]".green()
        };
        println!("{} 正在掃描並清理磁碟空間...", mode_tag);
    }

    for target in targets {
        let mut target_report = DiskCleanReport::default();
        clean_path_recursive(&target.path, dry_run, &mut target_report);

        grand_report.total_files_scanned += target_report.total_files_scanned;
        grand_report.total_files_deleted += target_report.total_files_deleted;
        grand_report.total_bytes_freed += target_report.total_bytes_freed;
        grand_report.total_errors += target_report.total_errors;

        if verbose && target_report.total_bytes_freed > 0 {
            let mb = target_report.total_bytes_freed as f64 / 1024.0 / 1024.0;
            println!(
                "  ✔ {:<32} : {:.1} MB ({} 個檔案)",
                target.name,
                mb,
                if dry_run {
                    target_report.total_files_scanned
                } else {
                    target_report.total_files_deleted
                }
            );
        }
    }

    if verbose {
        let total_mb = grand_report.total_bytes_freed as f64 / 1024.0 / 1024.0;
        let total_gb = total_mb / 1024.0;

        if dry_run {
            println!(
                "{}",
                format!(
                    "📋 預覽完成：共找到 {} 個可清理檔案，預估可釋放 {:.2} GB ({:.1} MB)",
                    grand_report.total_files_scanned, total_gb, total_mb
                )
                .yellow()
                .bold()
            );
        } else {
            println!(
                "{}",
                format!(
                    "🎉 清理完成：成功刪除 {} 個檔案，釋放了 {:.2} GB ({:.1} MB) 磁碟空間！",
                    grand_report.total_files_deleted, total_gb, total_mb
                )
                .bright_green()
                .bold()
            );
        }
    }

    grand_report
}
