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
use ratatui::widgets::canvas::{Canvas, Points};
use std::io::{stdout, Stdout};
use std::fs;
use std::path::PathBuf;
use dirs::home_dir;

const TRANS_BLUE: Color = Color::Rgb(0x5B, 0xCF, 0xFA);   // #5BCFFA
const TRANS_PINK: Color = Color::Rgb(0xF5, 0xA9, 0xB8);   // #F5A9B8
const TRANS_WHITE: Color = Color::Rgb(0xFF, 0xFF, 0xFF);  // #FFFFFF

enum ThemeName { Trans, Lesbian, Bisexual, NonBinary, Intersex, Progress }

fn theme_file() -> PathBuf {
    home_dir().unwrap_or_default().join(".ralf_theme")
}

fn parse_theme(s: &str) -> ThemeName {
    match s.trim().to_lowercase().as_str() {
        "trans" => ThemeName::Trans,
        "lesbian" => ThemeName::Lesbian,
        "bisexual" | "bi" => ThemeName::Bisexual,
        "non-binary" | "nonbinary" | "enby" => ThemeName::NonBinary,
        "intersex" => ThemeName::Intersex,
        "progress" => ThemeName::Progress,
        _ => ThemeName::Trans,
    }
}

pub fn current_theme() -> ThemeName {
    if let Ok(s) = std::env::var("RALF_THEME") { return parse_theme(&s); }
    if let Ok(s) = fs::read_to_string(theme_file()) { return parse_theme(&s); }
    ThemeName::Trans
}

pub fn set_theme_by_name(name: &str) -> Result<()> {
    fs::write(theme_file(), name.trim().to_lowercase())?;
    Ok(())
}

pub fn theme_options() -> &'static [&'static str] {
    &["Trans", "Lesbian", "Bisexual", "Non-binary", "Intersex", "Progress"]
}

fn theme_palette(t: ThemeName) -> (Vec<Color>, Color, Color) {
    match t {
        ThemeName::Trans => {
            let stripes = vec![TRANS_BLUE, TRANS_PINK, TRANS_WHITE, TRANS_PINK, TRANS_BLUE];
            (stripes, TRANS_PINK, TRANS_BLUE)
        }
        ThemeName::Lesbian => {
            let stripes = vec![
                Color::Rgb(0xD5,0x2D,0x00),
                Color::Rgb(0xEF,0x76,0x27),
                Color::Rgb(0xFF,0x9A,0x56),
                Color::Rgb(0xFF,0xFF,0xFF),
                Color::Rgb(0xD1,0x62,0xA4),
                Color::Rgb(0xB5,0x56,0x90),
                Color::Rgb(0xA3,0x02,0x62),
            ];
            (stripes, Color::Rgb(0xA3,0x02,0x62), Color::Rgb(0xD5,0x2D,0x00))
        }
        ThemeName::Bisexual => {
            let stripes = vec![
                Color::Rgb(0xD6,0x02,0x70),
                Color::Rgb(0x9B,0x4F,0x96),
                Color::Rgb(0x00,0x38,0xA8),
            ];
            (stripes, Color::Rgb(0xD6,0x02,0x70), Color::Rgb(0x00,0x38,0xA8))
        }
        ThemeName::NonBinary => {
            let stripes = vec![
                Color::Rgb(0xFF,0xF4,0x30),
                Color::Rgb(0xFF,0xFF,0xFF),
                Color::Rgb(0x9C,0x59,0xD1),
                Color::Rgb(0x2C,0x2C,0x2C),
            ];
            (stripes, Color::Rgb(0x9C,0x59,0xD1), Color::Rgb(0x9C,0x59,0xD1))
        }
        ThemeName::Intersex => {
            let stripes = vec![
                Color::Rgb(0xFF,0xD8,0x00),
                Color::Rgb(0xFF,0xD8,0x00),
                Color::Rgb(0xFF,0xD8,0x00),
                Color::Rgb(0xFF,0xD8,0x00),
                Color::Rgb(0xFF,0xD8,0x00),
            ];
            (stripes, Color::Rgb(0x79,0x02,0xAA), Color::Rgb(0x79,0x02,0xAA))
        }
        ThemeName::Progress => {
            // Simplified horizontal stripes approximation
            let stripes = vec![
                Color::Black,
                Color::Rgb(0x78,0x4F,0x17), // brown
                Color::Rgb(0x5B,0xCF,0xFA), // trans blue
                Color::Rgb(0xF5,0xA9,0xB8), // trans pink
                Color::Rgb(0xFF,0xFF,0xFF), // white
                Color::Rgb(0xE4,0x03,0x03), // red
                Color::Rgb(0xFF,0x8C,0x00), // orange
                Color::Rgb(0xFF,0xED,0x00), // yellow
                Color::Rgb(0x00,0x80,0x26), // green
                Color::Rgb(0x00,0x4D,0xFF), // blue
                Color::Rgb(0x75,0x07,0x87), // violet
            ];
            (stripes, Color::Rgb(0x75,0x07,0x87), Color::Rgb(0x00,0x4D,0xFF))
        }
    }
}

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
            let (stripe_colors, title_fg, border_fg) = theme_palette(current_theme());
            let n = stripe_colors.len() as u16;
            let base = 100 / n.max(1);
            let mut constraints: Vec<Constraint> = vec![Constraint::Percentage(base); n as usize];
            // add any remainder to the last stripe
            let used = base * n;
            if used < 100 {
                if let Some(last) = constraints.last_mut() {
                    *last = Constraint::Percentage(base + (100 - used));
                }
            }
            let stripes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);
            for (chunk, color) in stripes.iter().zip(stripe_colors.iter()) {
                f.render_widget(Block::default().style(Style::default().bg(*color)), *chunk);
            }
            // Input block
            let block = Block::default()
                .title(Line::from(prompt).style(Style::default().fg(title_fg).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_fg));
            let inner = block.inner(area);
            f.render_widget(block, area);
            // After: f.render_widget(block, area);
            if matches!(current_theme(), ThemeName::Intersex) {
                // Solid purple O (thick ring) centered within the inner area
                let cx = inner.width as f64 / 2.0;
                let cy = inner.height as f64 / 2.0;
                let min_dim = inner.width.min(inner.height) as f64;
                let r_outer = (min_dim * 0.35).max(6.0);
                let thickness = (min_dim * 0.12).max(3.0);
                let steps = thickness as i32;
                let mut pts: Vec<(f64, f64)> = Vec::with_capacity(360 * steps.max(1) as usize);
                for dr in 0..steps {
                    let r = r_outer - dr as f64;
                    for d in 0..360 {
                        let a = (d as f64).to_radians();
                        pts.push((cx + r * a.cos(), cy + r * a.sin()));
                    }
                }
                let canvas = Canvas::default()
                    .x_bounds([0.0, inner.width as f64])
                    .y_bounds([0.0, inner.height as f64])
                    .paint(|ctx| {
                        ctx.draw(&Points {
                            coords: &pts,
                            color: Color::Rgb(0x79, 0x02, 0xAA), // intersex purple
                        });
                    });
                f.render_widget(canvas, inner);
            }
            let text = Paragraph::new(buf.clone())
                .style(Style::default().fg(Color::Black));
            f.render_widget(text, inner);
            let hint = Paragraph::new("Type text, Enter to submit, Esc to cancel, Ctrl-U to clear")
                .alignment(Alignment::Center)
                .style(Style::default().fg(title_fg).bg(Color::Reset));
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
    use std::cmp::min;
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
            let (stripe_colors, title_fg, border_fg) = theme_palette(current_theme());
            let n = stripe_colors.len() as u16;
            let base = 100 / n.max(1);
            let mut constraints: Vec<Constraint> = vec![Constraint::Percentage(base); n as usize];
            // add any remainder to the last stripe
            let used = base * n;
            if used < 100 {
                if let Some(last) = constraints.last_mut() {
                    *last = Constraint::Percentage(base + (100 - used));
                }
            }
            let stripes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);
            for (chunk, color) in stripes.iter().zip(stripe_colors.iter()) {
                f.render_widget(Block::default().style(Style::default().bg(*color)), *chunk);
            }


            let block = Block::default()
                .title(Line::from(title).style(Style::default().fg(title_fg).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_fg));
            let inner = block.inner(area);
            f.render_widget(block, area);
            // After: f.render_widget(block, area);
            if matches!(current_theme(), ThemeName::Intersex) {
                let cx = inner.width as f64 / 2.0;
                let cy = inner.height as f64 / 2.0;
                let min_dim = inner.width.min(inner.height) as f64;
                let r_outer = (min_dim * 0.35).max(6.0);
                let thickness = (min_dim * 0.12).max(3.0);
                let steps = thickness as i32;
                let mut pts: Vec<(f64, f64)> = Vec::with_capacity(360 * steps.max(1) as usize);
                for dr in 0..steps {
                    let r = r_outer - dr as f64;
                    for d in 0..360 {
                        let a = (d as f64).to_radians();
                        pts.push((cx + r * a.cos(), cy + r * a.sin()));
                    }
                }
                let canvas = Canvas::default()
                    .x_bounds([0.0, inner.width as f64])
                    .y_bounds([0.0, inner.height as f64])
                    .paint(|ctx| {
                        ctx.draw(&Points {
                            coords: &pts,
                            color: Color::Rgb(0x79, 0x02, 0xAA),
                        });
                    });
                f.render_widget(canvas, inner);
            }

            let height = inner.height as usize;
            let total = lines.len();
            let start = min(scroll, total.saturating_sub(height));
            let end = min(start + height, total);
            let visible = if start < end { &lines[start..end] } else { &[] };
            let text = Paragraph::new(visible.join("\n")).style(Style::default().fg(Color::Black));
            f.render_widget(text, inner);

            let hint = Paragraph::new("↑/↓ PgUp/PgDn Home/End to scroll, q/Esc/Enter to return")
                .alignment(Alignment::Center)
                .style(Style::default().fg(title_fg).bg(Color::Reset));
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
            let (stripe_colors, title_fg, border_fg) = theme_palette(current_theme());
            let n = stripe_colors.len() as u16;
            let base = 100 / n.max(1);
            let mut constraints: Vec<Constraint> = vec![Constraint::Percentage(base); n as usize];
            // add any remainder to the last stripe
            let used = base * n;
            if used < 100 {
                if let Some(last) = constraints.last_mut() {
                    *last = Constraint::Percentage(base + (100 - used));
                }
            }
            let stripes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);
            for (chunk, color) in stripes.iter().zip(stripe_colors.iter()) {
                f.render_widget(Block::default().style(Style::default().bg(*color)), *chunk);
            }

            if matches!(current_theme(), ThemeName::Intersex) {
                let cx = area.width as f64 / 2.0;
                let cy = area.height as f64 / 2.0;
                let min_dim = area.width.min(area.height) as f64;
                let r_outer = (min_dim * 0.35).max(6.0);
                let thickness = (min_dim * 0.12).max(3.0);
                let steps = thickness as i32;
                let mut pts: Vec<(f64, f64)> = Vec::with_capacity(360 * steps.max(1) as usize);
                for dr in 0..steps {
                    let r = r_outer - dr as f64;
                    for d in 0..360 {
                        let a = (d as f64).to_radians();
                        pts.push((cx + r * a.cos(), cy + r * a.sin()));
                    }
                }
                let canvas = Canvas::default()
                    .x_bounds([0.0, area.width as f64])
                    .y_bounds([0.0, area.height as f64])
                    .paint(|ctx| {
                        ctx.draw(&Points {
                            coords: &pts,
                            color: Color::Rgb(0x79, 0x02, 0xAA),
                        });
                    });
                f.render_widget(canvas, area);
            }

            let block = Block::default()
                .title(Line::from(title).style(Style::default().fg(title_fg).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_fg));
            f.render_widget(block.clone(), area);
            // After: f.render_widget(block.clone(), area);
            if matches!(current_theme(), ThemeName::Intersex) {
                let inner = block.inner(area);
                let cx = inner.width as f64 / 2.0;
                let cy = inner.height as f64 / 2.0;
                let min_dim = inner.width.min(inner.height) as f64;
                let r_outer = (min_dim * 0.35).max(6.0);
                let thickness = (min_dim * 0.12).max(3.0);
                let steps = thickness as i32;
                let mut pts: Vec<(f64, f64)> = Vec::with_capacity(360 * steps.max(1) as usize);
                for dr in 0..steps {
                    let r = r_outer - dr as f64;
                    for d in 0..360 {
                        let a = (d as f64).to_radians();
                        pts.push((cx + r * a.cos(), cy + r * a.sin()));
                    }
                }
                let canvas = Canvas::default()
                    .x_bounds([0.0, inner.width as f64])
                    .y_bounds([0.0, inner.height as f64])
                    .paint(|ctx| {
                        ctx.draw(&Points {
                            coords: &pts,
                            color: Color::Rgb(0x79, 0x02, 0xAA),
                        });
                    });
                f.render_widget(canvas, inner);
            }

            let inner = block.inner(area);
            let text = Paragraph::new(message.to_string()).style(Style::default().fg(Color::Black));
            f.render_widget(text, inner);

            let hint = Paragraph::new("Press any key to return")
                .alignment(Alignment::Center)
                .style(Style::default().fg(title_fg).bg(Color::Reset));
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

            let (stripe_colors, title_fg, border_fg) = theme_palette(current_theme());
            let n = stripe_colors.len() as u16;
            let base = 100 / n.max(1);
            let mut constraints: Vec<Constraint> = vec![Constraint::Percentage(base); n as usize];
            // add any remainder to the last stripe
            let used = base * n;
            if used < 100 {
                if let Some(last) = constraints.last_mut() {
                    *last = Constraint::Percentage(base + (100 - used));
                }
            }
            let stripes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);
            for (chunk, color) in stripes.iter().zip(stripe_colors.iter()) {
                f.render_widget(Block::default().style(Style::default().bg(*color)), *chunk);
            }

            let block = Block::default()
                .title(Line::from(title).style(Style::default().fg(title_fg).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_fg));

            let items_widgets: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let style = if i == idx {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::White)
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
                .style(Style::default().fg(title_fg).bg(Color::Reset));
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
