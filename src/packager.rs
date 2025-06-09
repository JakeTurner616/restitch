// packager.rs

use crate::config::{ConfigItem, ConfigManifest};

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use tar::Builder;
use flate2::write::GzEncoder;
use flate2::Compression;

/// Add a file or directory into the tarball, using home-relative paths
fn add_path_to_tar<T: Write>(
    tar: &mut Builder<T>,
    source: &Path,
    base_dir: &Path,
) -> std::io::Result<()> {
    if source.is_file() {
        tar.append_path_with_name(source, source.strip_prefix(base_dir).unwrap())?;
    } else if source.is_dir() {
        tar.append_dir_all(source.strip_prefix(base_dir).unwrap(), source)?;
    }
    Ok(())
}

/// Create a .tar.gz archive and a manifest.toml for selected config items
pub fn create_archive(items: &[ConfigItem], archive_name: &str) {
    let home = dirs::home_dir().expect("Could not get home directory");

    let mut valid_paths = vec![];
    let mut invalid_paths = vec![];

    for item in items {
        let full_path = shellexpand::tilde(&item.path).into_owned();
        let path = PathBuf::from(full_path);

        if path.exists() {
            valid_paths.push((item, path));
        } else {
            invalid_paths.push(item);
        }
    }

    println!("\n🔍 Restitch: Validating selected config targets");
    println!("──────────────────────────────────────────────");
    println!("  ✅ Valid configs:   {}", valid_paths.len());
    println!("  ❌ Invalid configs: {}", invalid_paths.len());

    if !invalid_paths.is_empty() {
        println!("\n❌ Skipping packaging. The following entries are invalid:");
        for item in &invalid_paths {
            println!("   - {} ({})", item.name, item.path);
        }
        println!("\n💡 Fix or deselect these entries before proceeding.");
        return;
    }

    let output_dir = PathBuf::from("outputs");
    fs::create_dir_all(&output_dir).expect("❌ Failed to create output directory");

    let archive_path = output_dir.join(format!("{archive_name}.tar.gz"));
    let manifest_path = output_dir.join(format!("{archive_name}.manifest.toml"));

    let archive_file = BufWriter::new(
        File::create(&archive_path).expect("❌ Failed to create archive file"),
    );

    let encoder = GzEncoder::new(archive_file, Compression::default());
    let mut tar = Builder::new(encoder);

    println!("\n📦 Packaging:");
    for (idx, (_item, path)) in valid_paths.iter().enumerate() {
        let bullet = if idx == valid_paths.len() - 1 { "└─" } else { "├─" };
        println!("  {} 📁 {}", bullet, path.display());
        add_path_to_tar(&mut tar, path, &home).expect("❌ Failed to add to archive");
    }

    tar.finish().expect("❌ Failed to finalize archive");

    let manifest = ConfigManifest {
        items: items.to_vec(),
    };

    let toml_string = toml::to_string_pretty(&manifest).expect("Failed to serialize manifest");
    fs::write(&manifest_path, toml_string).expect("Failed to write manifest.toml");

    println!("\n📁 Output Summary:");
    println!("  📦 Archive:   {}", archive_path.display());
    println!("  📝 Manifest:  {}", manifest_path.display());
    println!("\n✅ Restitch archive complete. Ready to use `--restore --dry-run`");
}
