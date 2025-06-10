use std::fs;
use std::io;
use std::path::PathBuf;

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

pub fn restore_backup_dir(backup_dir: &PathBuf) -> std::io::Result<()> {
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

pub fn run_revert_ui() -> io::Result<()> {
    let backups_dir = PathBuf::from("backups");
    let mut entries = fs::read_dir(&backups_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.path());
    entries.reverse(); // newest first

    if entries.is_empty() {
        println!("❌ No backups available.");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let selected = ui_loop(&mut terminal, &entries)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Some(index) = selected {
        let backup_path = entries[index].path();
        println!("⚠️  This will overwrite your current configs with backup: {}", backup_path.display());
        println!("Proceeding...\n");

        match restore_backup_dir(&backup_path) {
            Ok(_) => println!("✅ Revert complete."),
            Err(e) => println!("❌ Error during revert: {}", e),
        }
    }

    Ok(())
}

fn ui_loop<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    entries: &[fs::DirEntry],
) -> io::Result<Option<usize>> {
    let mut state = ListState::default();
    if !entries.is_empty() {
        state.select(Some(0));
    }

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(2)].as_ref())
                .split(size);

            let list_items: Vec<ListItem> = entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let label = entry.file_name().to_string_lossy().to_string();
                    let style = if state.selected() == Some(i) {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Span::raw(label)).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(Block::default().title("🌀 Restitch: Select Backup to Revert").borders(Borders::ALL))
                .highlight_symbol(">>");

            f.render_stateful_widget(list, chunks[0], &mut state);

            let help = Block::default()
                .title("↑↓: Navigate  ↵: Revert Selected  q: Cancel")
                .borders(Borders::ALL);
            f.render_widget(help, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => return Ok(None),
                    KeyCode::Enter => return Ok(state.selected()),
                    KeyCode::Down => {
                        if let Some(i) = state.selected() {
                            let next = if i >= entries.len() - 1 { 0 } else { i + 1 };
                            state.select(Some(next));
                        }
                    }
                    KeyCode::Up => {
                        if let Some(i) = state.selected() {
                            let prev = if i == 0 { entries.len() - 1 } else { i - 1 };
                            state.select(Some(prev));
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
