use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, LineGauge, Paragraph, Wrap};

use super::{ACCENT, BAD, GOOD, HILITE, bold, centered, dim, fmt_secs};
use crate::app::App;
use crate::session::{FlashView, Mode, Session};
use crate::soroban;

/// 3×5 block font for flashed numbers; each pixel is drawn two cells wide.
const FONT: [[&str; 5]; 11] = [
    ["###", "# #", "# #", "# #", "###"],
    [" # ", "## ", " # ", " # ", "###"],
    ["###", "  #", "###", "#  ", "###"],
    ["###", "  #", "###", "  #", "###"],
    ["# #", "# #", "###", "  #", "  #"],
    ["###", "#  ", "###", "  #", "###"],
    ["###", "#  ", "###", "# #", "###"],
    ["###", "  #", "  #", "  #", "  #"],
    ["###", "# #", "###", "# #", "###"],
    ["###", "# #", "###", "  #", "###"],
    ["   ", "   ", "###", "   ", "   "],
];

pub fn big_text(s: &str) -> Vec<String> {
    let mut rows = vec![String::new(); 5];
    for c in s.chars() {
        let glyph = match c {
            '0'..='9' => &FONT[c as usize - '0' as usize],
            '-' | '−' => &FONT[10],
            _ => continue,
        };
        for (row, g) in rows.iter_mut().zip(glyph) {
            for px in g.chars() {
                row.push_str(if px == '#' { "██" } else { "  " });
            }
            row.push_str("  ");
        }
    }
    rows
}

pub fn draw_session(f: &mut Frame, area: Rect, app: &App, now: Instant) {
    let Some(s) = &app.session else { return };
    let fb_lines = feedback_lines(s, app.always_steps);
    let fb_height = (fb_lines.len() as u16 + 2).min(area.height / 2).max(3);
    let [header, problem, input, feedback] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(7),
        Constraint::Length(3),
        Constraint::Length(fb_height),
    ])
    .areas(area);

    draw_header(f, header, s, now);
    draw_problem(f, problem, s, app.flash_soroban, now);

    let ready = s.input_ready(now);
    let input_line = match (&s.error, ready) {
        (_, false) => Line::styled("watch the numbers…", dim()),
        (Some(e), _) => Line::from(vec![
            Span::styled(format!("> {}", s.input), bold(HILITE)),
            Span::raw("   "),
            Span::styled(e.clone(), bold(BAD)),
        ]),
        (None, true) => Line::from(vec![
            Span::styled(format!("> {}", s.input), bold(HILITE)),
            Span::styled("▏", Style::new().fg(HILITE)),
        ]),
    };
    f.render_widget(
        Paragraph::new(input_line).block(Block::bordered().title(" Answer ")),
        input,
    );

    let (title, style) = match &s.feedback {
        None => (" Last problem ".to_string(), dim()),
        Some(fb) if fb.correct => (format!(" ✓ Correct · {} ", fmt_secs(fb.time)), bold(GOOD)),
        Some(fb) => match &fb.given {
            Some(g) => (format!(" ✗ You said {g} "), bold(BAD)),
            None => (" Skipped ".to_string(), bold(BAD)),
        },
    };
    f.render_widget(
        Paragraph::new(fb_lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().title(Span::styled(title, style))),
        feedback,
    );

    if app.rule_overlay {
        draw_rule_overlay(f, area, s);
    }
}

fn draw_header(f: &mut Frame, area: Rect, s: &Session, now: Instant) {
    let block = Block::bordered().title(Span::styled(
        format!(
            " {} · {} · level {} ",
            match s.mode {
                Mode::Unlimited => "Unlimited",
                Mode::Timed(_) => "Timed",
            },
            s.choice.name(),
            s.difficulty
        ),
        bold(ACCENT),
    ));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let [text, gauge] = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(inner);

    let mut spans = vec![
        Span::styled(format!("✓ {}", s.correct()), bold(GOOD)),
        Span::raw(format!(" / {}", s.records.len())),
        Span::raw(format!("   accuracy {:.0}%", s.accuracy() * 100.0)),
        Span::raw(format!("   streak {} (best {})", s.streak, s.best_streak)),
    ];
    if let Some(avg) = s.avg_correct_time() {
        spans.push(Span::raw(format!("   avg {}", fmt_secs(avg))));
    }
    let clock = match s.remaining(now) {
        Some(r) => format!("⏱ {}:{:02} left", r.as_secs() / 60, r.as_secs() % 60),
        None => {
            let e = s.elapsed(now).as_secs();
            format!("⏱ {}:{:02}", e / 60, e % 60)
        }
    };
    spans.push(Span::styled(format!("   {clock}"), bold(HILITE)));
    f.render_widget(Paragraph::new(Line::from(spans)), text);

    if let (Mode::Timed(total), Some(rem)) = (s.mode, s.remaining(now)) {
        let ratio = rem.as_secs_f64() / total.as_secs_f64();
        let color = if ratio < 0.2 { BAD } else { ACCENT };
        f.render_widget(
            LineGauge::default()
                .ratio(ratio.clamp(0.0, 1.0))
                .filled_style(Style::new().fg(color))
                .unfilled_style(dim())
                .label(""),
            gauge,
        );
    }
}

fn draw_problem(f: &mut Frame, area: Rect, s: &Session, show_soroban: bool, now: Instant) {
    let p = &s.problem;
    let block = Block::bordered().title(Span::styled(
        format!(" {} · {} ", p.method.name(), p.technique_name()),
        dim(),
    ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if let (Some(view), Some(nums)) = (s.flash(now), &p.flash) {
        let status = format!(
            "{} numbers · {} ms each   [ slower  ] faster  r replay  s soroban {}",
            nums.len(),
            s.flash_interval.as_millis(),
            if show_soroban { "on" } else { "off" }
        );
        let mut lines: Vec<Line> = Vec::new();
        match view {
            FlashView::Show(i) => {
                let n = nums[i];
                let text = if n < 0 { format!("-{}", -n) } else { n.to_string() };
                lines.extend(big_text(&text).into_iter().map(|r| Line::styled(r, bold(HILITE))));
                if show_soroban {
                    lines.push(Line::raw(""));
                    lines.extend(
                        soroban::render(n.unsigned_abs(), 4)
                            .into_iter()
                            .map(|r| Line::styled(r, bold(ACCENT))),
                    );
                }
                lines.push(Line::raw(""));
                lines.push(Line::styled(format!("{} / {}", i + 1, nums.len()), dim()));
            }
            FlashView::Gap(_) => {}
            FlashView::Done => {
                lines.push(Line::styled(p.prompt.clone(), bold(ACCENT)));
            }
        }
        let [body, foot] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(inner);
        let h = lines.len() as u16;
        f.render_widget(Paragraph::new(lines).centered(), centered(body, body.width, h));
        f.render_widget(Paragraph::new(Line::styled(status, dim())).centered(), foot);
        return;
    }

    let mut lines = vec![Line::styled(format!("{} = ?", p.prompt), bold(ACCENT))];
    if let Some(note) = p.answer.tolerance_note() {
        lines.push(Line::styled(format!("({note})"), dim()));
    }
    let h = lines.len() as u16;
    f.render_widget(Paragraph::new(lines).centered(), centered(inner, inner.width, h));
}

fn feedback_lines(s: &Session, always_steps: bool) -> Vec<Line<'static>> {
    let Some(fb) = &s.feedback else {
        return vec![Line::styled(
            "Your result and the worked solution will appear here.",
            dim(),
        )];
    };
    let p = &fb.problem;
    let mut lines = vec![Line::from(vec![
        Span::raw(format!("{} = ", p.prompt.trim_end_matches(" as a percent"))),
        Span::styled(p.answer.shown.clone(), bold(GOOD)),
        Span::styled(
            p.answer
                .tolerance_note()
                .map(|n| format!("  ({n})"))
                .unwrap_or_default(),
            dim(),
        ),
    ])];
    if let Some(nums) = &p.flash {
        let seq: Vec<String> = nums.iter().map(|n| n.to_string()).collect();
        lines.push(Line::styled(format!("Flashed: {}", seq.join(", ")), dim()));
    }
    if !fb.correct || always_steps {
        lines.push(Line::styled(
            format!("How ({}):", p.technique_name()),
            bold(ACCENT),
        ));
        lines.extend(p.steps.iter().map(|st| Line::raw(format!("  {st}"))));
    }
    lines
}

fn draw_rule_overlay(f: &mut Frame, area: Rect, s: &Session) {
    let p = &s.problem;
    let tech = &p.method.techniques()[p.technique];
    let mut lines = vec![Line::raw("")];
    lines.extend(tech.rule.iter().map(|r| Line::raw(format!(" {r}"))));
    lines.push(Line::raw(""));
    lines.push(Line::styled(" Any key to close", dim()));
    let w = area.width.saturating_sub(8).min(90);
    let h = (lines.len() as u16 + 4).min(area.height);
    let r = centered(area, w, h);
    f.render_widget(Clear, r);
    f.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::bordered()
                .border_style(Style::new().fg(HILITE))
                .title(Span::styled(
                    format!(" {} · {} ", p.method.name(), tech.name),
                    bold(HILITE),
                )),
        ),
        r,
    );
}

pub fn draw_summary(f: &mut Frame, area: Rect, app: &App) {
    let Some(sum) = &app.summary else { return };
    let s = &sum.session;
    let mut lines = vec![
        Line::styled(format!("{} · level {}", s.choice.name(), s.difficulty), dim()),
        Line::raw(""),
        Line::from(vec![
            Span::raw("Correct:      "),
            Span::styled(format!("{} / {}", s.correct(), s.records.len()), bold(GOOD)),
            Span::raw(format!("  ({:.0}%)", s.accuracy() * 100.0)),
        ]),
        Line::raw(format!(
            "Avg time:     {}",
            s.avg_correct_time().map(fmt_secs).unwrap_or_else(|| "–".into())
        )),
        Line::raw(format!("Best streak:  {}", s.best_streak)),
        Line::raw(format!("Time:         {}", fmt_secs(s.elapsed(s.started)))),
    ];
    if s.timed_key().is_some() {
        lines.push(if sum.new_best {
            Line::styled(
                match sum.previous_best {
                    Some(b) => format!("New best! (previous {b})"),
                    None => "First score for this setup!".to_string(),
                },
                bold(HILITE),
            )
        } else {
            Line::raw(format!("Best:         {}", sum.previous_best.unwrap_or(0)))
        });
    }
    let missed: Vec<_> = s.records.iter().filter(|r| !r.correct).collect();
    if !missed.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::styled(format!("Missed ({}):", missed.len()), bold(BAD)));
        for r in missed.iter().take(12) {
            let you = r
                .given
                .as_deref()
                .map(|g| format!("you: {g}"))
                .unwrap_or("skipped".into());
            lines.push(Line::from(vec![
                Span::raw(format!("  {} = ", r.prompt)),
                Span::styled(r.expected.clone(), bold(GOOD)),
                Span::styled(format!("   ({you})"), dim()),
            ]));
        }
        if missed.len() > 12 {
            lines.push(Line::styled(format!("  …and {} more", missed.len() - 12), dim()));
        }
    }
    let h = lines.len() as u16 + 2;
    let title = match s.mode {
        Mode::Unlimited => " Session summary ",
        Mode::Timed(_) => " Time's up ",
    };
    f.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(Span::styled(title, bold(ACCENT)))),
        centered(area, 70, h),
    );
}
