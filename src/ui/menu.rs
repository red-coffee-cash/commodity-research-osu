use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};

use super::{ACCENT, HILITE, bold, centered, dim, level_dots};
use crate::app::{App, DURATIONS, MENU};
use crate::methods::Choice;
use crate::session::timed_key;

const BANNER: &[&str] = &[
    "┌┬┐┌─┐┌┐┌┌┬┐┌─┐┬    ┌┬┐┌─┐┌┬┐┬ ┬",
    "│││├┤ │││ │ ├─┤│    │││├─┤ │ ├─┤",
    "┴ ┴└─┘┘└┘ ┴ ┴ ┴┴─┘  ┴ ┴┴ ┴ ┴ ┴ ┴",
];

pub fn draw_menu(f: &mut Frame, area: Rect, app: &App) {
    let box_area = centered(area, 44, 16);
    let [banner, sub, list, info] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(2),
        Constraint::Length(MENU.len() as u16 + 2),
        Constraint::Fill(1),
    ])
    .areas(box_area);
    let lines: Vec<Line> = BANNER.iter().map(|l| Line::styled(*l, bold(ACCENT))).collect();
    f.render_widget(Paragraph::new(lines).centered(), banner);
    f.render_widget(
        Paragraph::new(Line::styled("mental math for quant interviews", dim())).centered(),
        sub,
    );
    let items: Vec<ListItem> = MENU.iter().map(|m| ListItem::new(format!("  {m}"))).collect();
    let mut state = ListState::default().with_selected(Some(app.menu));
    f.render_stateful_widget(
        List::new(items)
            .block(Block::bordered())
            .highlight_style(bold(HILITE))
            .highlight_symbol("▶"),
        list,
        &mut state,
    );
    let total: u64 = app.stats.methods.values().map(|m| m.attempts).sum();
    let correct: u64 = app.stats.methods.values().map(|m| m.correct).sum();
    if total > 0 {
        let line = format!(
            "{} sessions · {total} problems · {:.0}% correct",
            app.stats.sessions,
            100.0 * correct as f64 / total as f64
        );
        f.render_widget(Paragraph::new(Line::styled(line, dim())).centered(), info);
    }
}

pub fn draw_setup(f: &mut Frame, area: Rect, app: &App) {
    let s = &app.setup;
    let box_area = centered(area, 72, 22);
    let title = if s.timed {
        " Timed practice "
    } else {
        " Unlimited practice "
    };
    let block = Block::bordered().title(Span::styled(title, bold(ACCENT)));
    let inner = block.inner(box_area);
    f.render_widget(block, box_area);

    let [left, right] = Layout::horizontal([Constraint::Length(26), Constraint::Fill(1)])
        .spacing(2)
        .areas(inner);

    let items: Vec<ListItem> = Choice::ALL.iter().map(|c| ListItem::new(c.name())).collect();
    let mut state = ListState::default().with_selected(Some(s.choice));
    f.render_stateful_widget(
        List::new(items)
            .block(Block::bordered().title(" Method "))
            .highlight_style(bold(HILITE))
            .highlight_symbol("▶ "),
        left,
        &mut state,
    );

    let mut lines = vec![
        Line::from(vec![
            Span::raw("Level:    "),
            Span::styled(level_dots(s.difficulty), bold(HILITE)),
        ]),
        Line::from(vec![
            Span::raw("Mode:     "),
            Span::styled(if s.timed { "Timed" } else { "Unlimited" }, bold(HILITE)),
        ]),
    ];
    if s.timed {
        lines.push(Line::from(vec![
            Span::raw("Duration: "),
            Span::styled(format!("{}s", DURATIONS[s.duration]), bold(HILITE)),
        ]));
        let choice = Choice::ALL[s.choice];
        let key = timed_key(choice, s.difficulty, DURATIONS[s.duration]);
        if let Some(best) = app.stats.timed_best.get(&key) {
            lines.push(Line::styled(format!("Best:     {best} correct"), dim()));
        }
    }
    lines.push(Line::raw(""));
    match Choice::ALL[s.choice] {
        Choice::Mixed => {
            lines.push(Line::styled("Problems from every method, at random.", dim()));
        }
        Choice::Only(m) => {
            lines.push(Line::styled("Techniques drilled:", dim()));
            for t in m.techniques() {
                lines.push(Line::raw(format!("  • {}", t.name)));
            }
        }
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled("Press Enter to start.", bold(ACCENT)));
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), right);
}
