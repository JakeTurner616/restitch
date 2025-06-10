use crate::config::{ConfigItem, ConfigManifest};
use chrono::Local;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tar::Archive;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

pub fn restore_configs(archive_path: &str, manifest_path: &str, dry_run: bool) {
    if !Path::new(archive_path).exists() || !Path::new(manifest_path).exists() {
        println!("❌ Archive or manifest not found.\n");
        println!("Restitch could not find the default archive or manifest file in:");
        println!("  • {}", archive_path);
        println!("  • {}\n", manifest_path);
        std::process::exit(1);
    }

    println!("📦 Extracting archive...");
    let tar_gz = fs::File::open(archive_path).expect("❌ Could not open archive file");
    let decompressor = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = Archive::new(decompressor);

    fs::create_dir_all("restitch_tmp").expect("❌ Could not create temp extraction directory");
    archive.unpack("restitch_tmp").expect("❌ Failed to extract archive");

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
    }

    if dry_run {
        println!("\n🔎 Restore dry-run complete.");
        return;
    }

    // 🛑 Prompt confirmation before continuing
    println!("\n⚠️  This operation will overwrite the above config files.");
    print!("Proceed with restore? [y/N]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() != "y" {
        println!("\n❌ Restore cancelled.");
        return;
    }

    // 🛠️ Perform actual restore
    for item in &manifest.items {
        let rel_path = Path::new(&item.path)
            .strip_prefix(home.to_str().unwrap())
            .unwrap_or(Path::new(&item.path));
        let backup_path = backup_dir.join(rel_path);

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

    println!("\n✅ Restore completed successfully.");
    println!("📁 Backups saved to: backups/{}/", timestamp);
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
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

pub fn run_restore_ui(manifest_path: &str, archive_path: &str, dry_run: bool) -> io::Result<()> {
    let manifest_str = fs::read_to_string(manifest_path)?;
    let manifest: ConfigManifest = toml::from_str(&manifest_str)
        .expect("❌ Invalid manifest format");

    let mut items: Vec<ConfigItem> = manifest.items
        .into_iter()
        .map(|mut item| { item.selected = true; item })
        .collect();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = ui_loop(&mut terminal, &mut items, dry_run);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(true) = result {
        let selected_items: Vec<ConfigItem> = items
            .into_iter()
            .filter(|i| i.selected)
            .collect();

        if selected_items.is_empty() {
            println!("❌ No items selected.");
        } else {
            let temp_manifest = ConfigManifest { items: selected_items };
            let temp_manifest_str = toml::to_string(&temp_manifest).unwrap();
            fs::write("restitch_tmp_selected.manifest.toml", &temp_manifest_str)?;
            restore_configs(archive_path, "restitch_tmp_selected.manifest.toml", dry_run);
        }
    }

    Ok(())
}

fn ui_loop<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    items: &mut Vec<ConfigItem>,
    dry_run: bool,
) -> io::Result<bool> {
    // Handle config loading errors BEFORE enabling terminal features
    let mut state = ListState::default();
    if !items.is_empty() {
        state.select(Some(0));
    }

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(2)].as_ref())
                .split(size);

            let list_items: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let prefix = if item.selected { "[x]" } else { "[ ]" };
                    let line = format!("{} {}", prefix, item.name);
                    let style = if state.selected() == Some(i) {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Span::raw(line)).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(Block::default().title("🌀 Restitch: Restore Configs").borders(Borders::ALL))
                .highlight_symbol(">>");

            f.render_stateful_widget(list, chunks[0], &mut state);

            let help_text = if dry_run {
                "↑↓: Navigate  ␣: Toggle  enter: Run Dry-run  q: Quit"
            } else {
                "↑↓: Navigate  ␣: Toggle  enter: Restore  q: Quit"
            };

            let help = Block::default().title(help_text).borders(Borders::ALL);
            f.render_widget(help, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => return Ok(false),
                    KeyCode::Enter => return Ok(true),
                    KeyCode::Down => {
                        if let Some(i) = state.selected() {
                            let next = if i >= items.len() - 1 { 0 } else { i + 1 };
                            state.select(Some(next));
                        }
                    }
                    KeyCode::Up => {
                        if let Some(i) = state.selected() {
                            let prev = if i == 0 { items.len() - 1 } else { i - 1 };
                            state.select(Some(prev));
                        }
                    }
                    KeyCode::Char(' ') => {
                        if let Some(i) = state.selected() {
                            items[i].selected = !items[i].selected;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
