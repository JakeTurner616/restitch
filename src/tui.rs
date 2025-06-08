// tui.rs

use crate::config::ConfigItem;
use crate::detectors::scan_targets_from_file;

use std::io;
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

pub fn run_ui_with_config(config_path: &str) -> io::Result<Vec<ConfigItem>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = ui_loop(&mut terminal, config_path);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn ui_loop<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config_path: &str,
) -> io::Result<Vec<ConfigItem>> {
    let mut items = scan_targets_from_file(config_path);
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
                .block(Block::default().title("🌀 Restitch: Select Configs").borders(Borders::ALL))
                .highlight_symbol(">>");

            f.render_stateful_widget(list, chunks[0], &mut state);

            let help = Block::default()
                .title("↑↓: Navigate  ␣: Toggle  p: Package  q: Quit")
                .borders(Borders::ALL);
            f.render_widget(help, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('p') => {
                        let selected_items = items
                            .iter()
                            .filter(|i| i.selected)
                            .cloned()
                            .collect::<Vec<_>>();
                        return Ok(selected_items);
                    }
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

    Ok(vec![]) // fallback if exited with 'q'
}