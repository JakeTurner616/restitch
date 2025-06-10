mod tui;
mod detectors;
mod packager;
mod config;
mod restore;
mod revert;

use clap::Parser;
use std::fs;
use std::io;
use std::process;

/// Restitch CLI – Export, Restore, or Revert Linux Configs
#[derive(Parser, Debug)]
#[command(author, version, about = "🌀 Restitch – Export, Restore, or Revert Linux Configs")]
struct Args {
    /// Enable restore mode (TUI if no archive/manifest given)
    #[arg(long)]
    restore: bool,

    /// Enable revert mode (TUI interface)
    #[arg(long)]
    revert: bool,

    /// Simulate restore without writing files
    #[arg(long)]
    dry_run: bool,

    /// Optional path to archive (.tar.gz)
    #[arg()]
    archive: Option<String>,

    /// Optional path to manifest (.toml)
    #[arg()]
    manifest: Option<String>,

    /// Optional path to config_targets.toml
    #[arg(long, default_value = "config_targets.toml")]
    config_path: String,
}

fn main() {
    // Ensure unexpected panics do not corrupt terminal or print gibberish
    std::panic::set_hook(Box::new(|info| {
        eprintln!("❌ Unexpected fatal error: {}", info);
        process::exit(1);
    }));

    let args = Args::parse();

    // 🚫 Invalid usage: dry-run without --restore
    if args.dry_run && !args.restore {
        eprintln!("❌ '--dry-run' can only be used with '--restore'");
        process::exit(1);
    }

    // 🔁 Revert (always uses TUI selector)
    if args.revert {
        if let Err(e) = revert::run_revert_ui() {
            eprintln!("❌ Revert UI error: {}", e);
            process::exit(1);
        }
    }
    // 🔄 Restore (TUI if no archive/manifest provided)
    else if args.restore {
        let archive = args.archive.clone().unwrap_or_default();
        let manifest = args.manifest.clone().unwrap_or_default();

        if archive.is_empty() || manifest.is_empty() {
            // No explicit paths → launch TUI restore interface
            if let Err(e) = restore::run_restore_ui(
                "outputs/restitch-archive.manifest.toml",
                "outputs/restitch-archive.tar.gz",
                args.dry_run,
            ) {
                eprintln!("❌ Restore UI error: {}", e);
                process::exit(1);
            }
        } else {
            // Check that manifest exists and is readable
            match fs::read_to_string(&manifest) {
                Ok(_) => {
                    restore::restore_configs(&archive, &manifest, args.dry_run);
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    eprintln!("❌ Manifest file not found: '{}'", manifest);
                    process::exit(1);
                }
                Err(err) => {
                    eprintln!("❌ Failed to read manifest '{}': {}", manifest, err);
                    process::exit(1);
                }
            }
        }
    }
    // 📦 Default mode: Package (TUI for selecting configs)
    else {
        match tui::run_ui_with_cleanup(&args.config_path) {
            Ok(items) => {
                if items.is_empty() {
                    println!("⚠️ No config items selected. Nothing to export.");
                } else {
                    packager::create_archive(&items, "restitch-archive");
                }
            }
            Err(e) => {
                eprintln!("❌ UI error: {}", e);
                process::exit(1);
            }
        }
    }
}
