use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io::{stdout, Stdout};

const TRANS_BLUE: Color = Color::Rgb(0x5B, 0xCF, 0xFA);   // #5BCFFA
const TRANS_PINK: Color = Color::Rgb(0xF5, 0xA9, 0xB8);   // #F5A9B8
const TRANS_WHITE: Color = Color::Rgb(0xFF, 0xFF, 0xFF);  // #FFFFFF

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

pub fn select(title: &str, items: &[&str]) -> Result<Option<usize>> {
    list_select(title, items)
}

pub fn input(prompt: &str) -> Result<Option<String>> {
    use crossterm::event::KeyModifiers;
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut buf = String::new();
    let res = loop {
        terminal.draw(|f| {
            let area = f.area();
            // Background stripes
            let stripes = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ])
                .split(area);
            let stripe_colors = [TRANS_BLUE, TRANS_PINK, TRANS_WHITE, TRANS_PINK, TRANS_BLUE];
            for (chunk, color) in stripes.iter().zip(stripe_colors.iter()) {
                f.render_widget(Block::default().style(Style::default().bg(*color)), *chunk);
            }
            // Input block
            let block = Block::default()
                .title(Line::from(prompt).style(Style::default().fg(TRANS_PINK).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TRANS_BLUE));
            let inner = block.inner(area);
            f.render_widget(block, area);
            let text = Paragraph::new(buf.clone())
                .style(Style::default().fg(Color::Black));
            f.render_widget(text, inner);
            let hint = Paragraph::new("Type text, Enter to submit, Esc to cancel, Ctrl-U to clear")
                .alignment(Alignment::Center)
                .style(Style::default().fg(TRANS_PINK).bg(Color::Reset));
            let hint_area = ratatui::layout::Rect {
                x: area.x,
                y: area.y.saturating_add(area.height.saturating_sub(2)),
                width: area.width,
                height: 1,
            };
            f.render_widget(hint, hint_area);
        })?;
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Esc => break None,
                    KeyCode::Enter => break Some(buf.clone()),
                    KeyCode::Backspace => { buf.pop(); }
                    KeyCode::Char(c) => {
                        if k.modifiers.contains(KeyModifiers::CONTROL) {
                            // simple Ctrl-U clear
                            if c == 'u' || c == 'U' { buf.clear(); }
                        } else {
                            buf.push(c);
                        }
                    }
                    _ => {}
                }
            }
        }
    };
    cleanup(terminal)?;
    Ok(res)
}

pub fn view_text(title: &str, body: &str) -> Result<()> {
    use std::cmp::{max, min};
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let lines: Vec<String> = body.lines().map(|s| s.to_string()).collect();
    let mut scroll: usize = 0;

    let res = loop {
        terminal.draw(|f| {
            let area = f.area();
            // Background stripes
            let stripes = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ])
                .split(area);
            let stripe_colors = [TRANS_BLUE, TRANS_PINK, TRANS_WHITE, TRANS_PINK, TRANS_BLUE];
            for (chunk, color) in stripes.iter().zip(stripe_colors.iter()) {
                f.render_widget(Block::default().style(Style::default().bg(*color)), *chunk);
            }

            let block = Block::default()
                .title(Line::from(title).style(Style::default().fg(TRANS_PINK).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TRANS_BLUE));
            let inner = block.inner(area);
            f.render_widget(block, area);

            let height = inner.height as usize;
            let total = lines.len();
            let start = min(scroll, total.saturating_sub(height));
            let end = min(start + height, total);
            let visible = if start < end { &lines[start..end] } else { &[] };
            let text = Paragraph::new(visible.join("\n")).style(Style::default().fg(Color::Black));
            f.render_widget(text, inner);

            let hint = Paragraph::new("↑/↓ PgUp/PgDn Home/End to scroll, q/Esc/Enter to return")
                .alignment(Alignment::Center)
                .style(Style::default().fg(TRANS_PINK).bg(Color::Reset));
            let hint_area = ratatui::layout::Rect {
                x: area.x,
                y: area.y.saturating_add(area.height.saturating_sub(2)),
                width: area.width,
                height: 1,
            };
            f.render_widget(hint, hint_area);
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => break,
                    KeyCode::Up => scroll = scroll.saturating_sub(1),
                    KeyCode::Down => scroll = scroll.saturating_add(1),
                    KeyCode::PageUp => scroll = scroll.saturating_sub(10),
                    KeyCode::PageDown => scroll = scroll.saturating_add(10),
                    KeyCode::Home => scroll = 0,
                    KeyCode::End => scroll = usize::MAX / 2, // will be clamped on draw
                    _ => {}
                }
            }
        }
    };

    cleanup(terminal)?;
    Ok(res)
}

pub fn notify(title: &str, message: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let _ = loop {
        terminal.draw(|f| {
            let area = f.area();
            // Background stripes
            let stripes = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ])
                .split(area);
            let stripe_colors = [TRANS_BLUE, TRANS_PINK, TRANS_WHITE, TRANS_PINK, TRANS_BLUE];
            for (chunk, color) in stripes.iter().zip(stripe_colors.iter()) {
                f.render_widget(Block::default().style(Style::default().bg(*color)), *chunk);
            }

            let block = Block::default()
                .title(Line::from(title).style(Style::default().fg(TRANS_PINK).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TRANS_BLUE));
            f.render_widget(block.clone(), area);

            let inner = block.inner(area);
            let text = Paragraph::new(message.to_string()).style(Style::default().fg(Color::Black));
            f.render_widget(text, inner);

            let hint = Paragraph::new("Press any key to return")
                .alignment(Alignment::Center)
                .style(Style::default().fg(TRANS_PINK).bg(Color::Reset));
            let hint_area = ratatui::layout::Rect {
                x: area.x,
                y: area.y.saturating_add(area.height.saturating_sub(2)),
                width: area.width,
                height: 1,
            };
            f.render_widget(hint, hint_area);
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(_) = event::read()? {
                break;
            }
        }
    };

    // Close
    // Reuse cleanup to restore terminal
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    cleanup(terminal)?;
    Ok(())
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
            let area = f.area();

            let stripes = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ])
                .split(area);
            // Blue, Pink, White, Pink, Blue
            let stripe_colors = [TRANS_BLUE, TRANS_PINK, TRANS_WHITE, TRANS_PINK, TRANS_BLUE];
            for (chunk, color) in stripes.iter().zip(stripe_colors.iter()) {
                f.render_widget(Block::default().style(Style::default().bg(*color)), *chunk);
            }

            let block = Block::default()
                .title(Line::from(title).style(Style::default().fg(TRANS_PINK).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TRANS_BLUE));

            let items_widgets: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let style = if i == idx {
                        Style::default()
                            .fg(Color::Black)
                            .bg(TRANS_WHITE)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Black)
                    };
                    ListItem::new(Line::from((*s).to_string())).style(style)
                })
                .collect();

            let list = List::new(items_widgets).block(block);
            f.render_widget(list, area);

            let hint = Paragraph::new("Use ↑/↓ to move, Enter to select, Esc to abort")
                .alignment(Alignment::Center)
                .style(Style::default().fg(TRANS_PINK).bg(Color::Reset));
            let hint_area = ratatui::layout::Rect {
                x: area.x,
                y: area.y.saturating_add(area.height.saturating_sub(2)),
                width: area.width,
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
