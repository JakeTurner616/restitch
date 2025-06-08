// revert.rs

use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use chrono::NaiveDateTime;

/// Finds the latest backup directory in ./backups/ or returns an error.
fn find_latest_backup() -> Option<PathBuf> {
    let backups_dir = PathBuf::from("backups");
    let mut entries = fs::read_dir(&backups_dir).ok()?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.path());
    entries.pop().map(|e| e.path())
}

/// Recursively restore files from backup to their original location.
fn restore_backup_dir(backup_dir: &PathBuf) -> std::io::Result<()> {
    for entry in fs::read_dir(backup_dir)? {
        let entry = entry?;
        let entry_path = entry.path().to_path_buf();
        let rel_path = entry_path.strip_prefix(backup_dir).unwrap();
        let dest = dirs::home_dir().unwrap().join(rel_path);

        if entry.file_type()?.is_dir() {
            fs::create_dir_all(&dest)?;
            restore_backup_dir(&entry.path())?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &dest)?;
            println!("🔁 Restored: {}", dest.display());
        }
    }
    Ok(())
}

/// Main entry point for revert: prompts and restores the latest backup.
pub fn revert() {
    let backup = find_latest_backup();
    if backup.is_none() {
        println!("❌ No backup directories found in ./backups/");
        return;
    }
    let backup = backup.unwrap();

    let label = backup
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let ts_hint = NaiveDateTime::parse_from_str(label, "%Y-%m-%d_%H-%M-%S")
        .map(|ts| format!("{}", ts.format("%b %d, %Y at %H:%M")))
        .unwrap_or_else(|_| "unknown timestamp".to_string());

    println!("📁 Most recent backup: ./backups/{label}");
    println!("📅 Created: {}", ts_hint);
    println!();

    print!("⚠️  This will overwrite your current config files. Proceed? [y/N]: ");
    stdout().flush().unwrap();

    let mut resp = String::new();
    stdin().read_line(&mut resp).unwrap();
    if resp.trim().to_lowercase() != "y" {
        println!("\n❌ Revert cancelled.");
        return;
    }

    println!("\n🚧 Reverting system configs...");
    match restore_backup_dir(&backup) {
        Ok(_) => println!("\n✅ Revert complete."),
        Err(e) => println!("❌ Error during revert: {}", e),
    }
}
