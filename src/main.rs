// main.rs

mod tui;
mod detectors;
mod packager;
mod config;
mod restore;
mod revert;

use clap::Parser;

/// Restitc CLI – Export, Restore, or Revert Linux Configs
#[derive(Parser, Debug)]
#[command(author, version, about = "🌀 Restitch – Export, Restore, or Revert Linux Configs")]
struct Args {
    /// Enable restore mode
    #[arg(long)]
    restore: bool,

    /// Enable revert mode
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
    let args = Args::parse();

    // 🚫 Invalid usage: dry-run without --restore
    if args.dry_run && !args.restore {
        eprintln!("❌ '--dry-run' can only be used with '--restore'");
        std::process::exit(1);
    }

    if args.revert {
        revert::revert();
    }
    else if args.restore {
        let archive = args.archive.unwrap_or_else(|| "outputs/restitch-archive.tar.gz".to_string());
        let manifest = args.manifest.unwrap_or_else(|| "outputs/restitch-archive.manifest.toml".to_string());
        restore::restore_configs(&archive, &manifest, args.dry_run);
    } else {
        match tui::run_ui_with_config(&args.config_path) {
            Ok(items) => {
                if items.is_empty() {
                    println!("⚠️ No config items selected. Nothing to export.");
                } else {
                    packager::create_archive(&items, "restitch-archive");
                }
            }
            Err(e) => {
                eprintln!("❌ UI error: {}", e);
                std::process::exit(1);
            }
        }
    }
}