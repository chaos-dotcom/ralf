use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io::{stdout, Stdout};

pub enum ConnectChoice {
    Ssh,
    Https,
    Abort,
}

pub fn choose_github_protocol() -> Result<ConnectChoice> {
    let items = ["SSH", "HTTPS", "Abort"];
    match list_select("Select GitHub protocol", &items)? {
        Some(0) => Ok(ConnectChoice::Ssh),
        Some(1) => Ok(ConnectChoice::Https),
        Some(2) | None => Ok(ConnectChoice::Abort),
        _ => Ok(ConnectChoice::Abort),
    }
}

pub fn confirm(prompt: &str) -> Result<bool> {
    let items = ["Yes", "No"];
    match list_select(prompt, &items)? {
        Some(0) => Ok(true),
        _ => Ok(false),
    }
}

fn list_select(title: &str, items: &[&str]) -> Result<Option<usize>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut idx: usize = 0;

    let res = loop {
        terminal.draw(|f| {
            let size = f.size();

            let block = Block::default().title(title).borders(Borders::ALL);

            let items_widgets: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let style = if i == idx {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from((*s).to_string())).style(style)
                })
                .collect();

            let list = List::new(items_widgets).block(block);
            f.render_widget(list, size);

            let hint = Paragraph::new("Use ↑/↓ to move, Enter to select, Esc to abort")
                .alignment(Alignment::Center);
            let hint_area = ratatui::layout::Rect {
                x: size.x,
                y: size.y.saturating_add(size.height.saturating_sub(2)),
                width: size.width,
                height: 1,
            };
            f.render_widget(hint, hint_area);
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Esc => break None,
                    KeyCode::Up => {
                        if idx > 0 {
                            idx -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if idx + 1 < items.len() {
                            idx += 1;
                        }
                    }
                    KeyCode::Enter => break Some(idx),
                    _ => {}
                }
            }
        }
    };

    cleanup(terminal)?;
    Ok(res)
}

fn cleanup(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    terminal.show_cursor()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
