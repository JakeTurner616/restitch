use crate::config::ConfigManifest;
use chrono::Local;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tar::Archive;

/// Restores configs from a backup archive using a manifest, optionally as a dry-run
pub fn restore_configs(archive_path: &str, manifest_path: &str, dry_run: bool) {
    // Validate both files exist
    if !Path::new(archive_path).exists() || !Path::new(manifest_path).exists() {
        println!("❌ Archive or manifest not found.\n");
        println!("Restitch could not find the default archive or manifest file in:");
        println!("  • {}", archive_path);
        println!("  • {}\n", manifest_path);
        println!("💡 To restore configs, either:");
        println!("  1. First run `restitch` and export a config archive.");
        println!("  2. Or pass custom paths to an archive and manifest like so:");
        println!("     restitch --restore path/to/archive.tar.gz path/to/manifest.toml\n");
        std::process::exit(1);
    }

    println!("📦 Extracting archive...");
    let tar_gz = fs::File::open(archive_path).expect("❌ Could not open archive file");
    let decompressor = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = Archive::new(decompressor);

    fs::create_dir_all("restitch_tmp").expect("❌ Could not create temp extraction directory");
    archive
        .unpack("restitch_tmp")
        .expect("❌ Failed to extract archive");

    println!("📂 Extracted to: restitch_tmp/\n");

    let manifest_str = fs::read_to_string(manifest_path).expect("❌ Could not read manifest file");
    let manifest: ConfigManifest = toml::from_str(&manifest_str).expect("❌ Invalid manifest format");

    println!("🧭 Restore Plan{}:", if dry_run { " (dry-run)" } else { "" });
    println!("───────────────────────────────────────────────");

    let home = dirs::home_dir().expect("Could not get home directory");
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let backup_dir = PathBuf::from("backups").join(&timestamp);

    for item in &manifest.items {
        let rel_path = Path::new(&item.path)
            .strip_prefix(home.to_str().unwrap())
            .unwrap_or(Path::new(&item.path));
        let backup_path = backup_dir.join(rel_path);

        let is_dir = fs::metadata(&item.path)
            .map(|meta| meta.is_dir())
            .unwrap_or(false);

        println!(
            "🔁 REPLACE: {} → {}\n   ↪ Backup will be created at: {}{}",
            item.name,
            item.path,
            backup_path.display(),
            if is_dir {
                "\n   ⚠️  This is a directory and all its contents will be restored recursively"
            } else {
                ""
            }
        );

        if !dry_run {
            fs::create_dir_all(backup_path.parent().unwrap())
                .expect("❌ Could not create backup directory");

            if Path::new(&item.path).exists() {
                fs::rename(&item.path, &backup_path)
                    .expect("❌ Failed to back up existing file");
            }

            let extracted_path = Path::new("restitch_tmp").join(rel_path);
            fs::create_dir_all(Path::new(&item.path).parent().unwrap())
                .expect("❌ Could not create destination directory");

            if extracted_path.is_dir() {
                copy_dir_recursive(&extracted_path, Path::new(&item.path))
                    .expect("❌ Failed to copy directory");
            } else {
                fs::copy(&extracted_path, &item.path)
                    .expect("❌ Failed to copy file");
            }
        }
    }

    if dry_run {
        println!("\n🔎 Restore dry-run complete.");
        println!("👉 If the above plan looks correct, run with `--restore` (no --dry-run) to apply the changes.");
        println!("💡 All listed config files would be overwritten, and existing versions backed up.");
        println!("💡 Directory paths will restore all nested files and subdirectories from the archive.");
        println!("💡 You can undo applied changes at any time by running: `restitch --revert`");
        println!("───────────────────────────────────────────────");
    } else {
        println!("\n✅ Restore completed successfully.");
        println!("📁 Backups saved to: backups/{}/", timestamp);
        println!("💡 You can revert these changes at any time by running: `restitch --revert`.");
        println!("───────────────────────────────────────────────");
    }
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}