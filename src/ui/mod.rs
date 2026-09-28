//! Rendering. Each screen draws into the area above a one-line footer of key hints.

mod help;
mod menu;
mod session;
mod stats;

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, Screen};

pub const ACCENT: Color = Color::Cyan;
pub const GOOD: Color = Color::Green;
pub const BAD: Color = Color::Red;
pub const HILITE: Color = Color::Yellow;
pub const DIM: Color = Color::DarkGray;

pub fn bold(c: Color) -> Style {
    Style::new().fg(c).add_modifier(Modifier::BOLD)
}

pub fn dim() -> Style {
    Style::new().fg(DIM)
}

pub fn draw(f: &mut Frame, app: &App, now: Instant) {
    let [main, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(f.area());
    match app.screen {
        Screen::Menu => menu::draw_menu(f, main, app),
        Screen::Setup => menu::draw_setup(f, main, app),
        Screen::Session => session::draw_session(f, main, app, now),
        Screen::Summary => session::draw_summary(f, main, app),
        Screen::Help => help::draw(f, main, app),
        Screen::Soroban => stats::draw_soroban(f, main, app),
        Screen::Stats => stats::draw_stats(f, main, app),
    }
    draw_footer(f, footer, app);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let hints = match app.screen {
        Screen::Menu => "↑↓ move · Enter select · q quit",
        Screen::Setup => "↑↓ method · ←→ level · Tab unlimited/timed · t duration · Enter start · Esc back",
        Screen::Session => "Enter submit · Tab skip · ? rule · w walkthroughs · Esc end",
        Screen::Summary => "r restart · Enter menu",
        Screen::Help => "↑↓ page · j/k PgUp/PgDn scroll · ←→ level · n new examples · p practise · Esc back",
        Screen::Soroban => "type digits · ↑↓ ±1 · PgUp/PgDn ±10 · Backspace · x clear · r random · Esc back",
        Screen::Stats => "Esc back",
    };
    let mut spans = vec![Span::styled(hints, dim())];
    if let Some(s) = &app.status {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(s.clone(), bold(HILITE)));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// A `w`×`h` rectangle centred in `area`, clipped to fit.
pub fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

/// "● ● ○ ○ ○" style level indicator.
pub fn level_dots(level: u8) -> String {
    (1..=crate::methods::MAX_DIFFICULTY)
        .map(|i| if i <= level { "●" } else { "○" })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn fmt_secs(d: std::time::Duration) -> String {
    format!("{:.1}s", d.as_secs_f64())
}
