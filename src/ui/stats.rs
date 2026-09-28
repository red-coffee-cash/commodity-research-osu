use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Wrap};

use super::{ACCENT, HILITE, bold, centered, dim};
use crate::app::App;
use crate::methods::Method;
use crate::soroban;

pub fn draw_stats(f: &mut Frame, area: Rect, app: &App) {
    let st = &app.stats;
    let area = centered(area, 80, area.height);
    let best_rows = st.timed_best.len().max(1) as u16;
    let [top, methods, bests] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(Method::ALL.len() as u16 + 4),
        Constraint::Length(best_rows + 4),
    ])
    .areas(area);

    let path = app
        .stats_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or("not saved".into());
    f.render_widget(
        Paragraph::new(vec![Line::from(vec![
            Span::styled(format!("{} sessions", st.sessions), bold(ACCENT)),
            Span::styled(format!("   saved at {path}"), dim()),
        ])])
        .wrap(Wrap { trim: false }),
        top,
    );

    let header = Row::new(["Method", "Attempts", "Accuracy", "Avg time (correct)"]).style(bold(HILITE));
    let rows = Method::ALL.iter().map(|m| {
        let s = st.methods.get(m.name()).copied().unwrap_or_default();
        Row::new([
            Cell::from(m.name()),
            Cell::from(s.attempts.to_string()),
            Cell::from(
                s.accuracy()
                    .map(|a| format!("{:.0}%", a * 100.0))
                    .unwrap_or("–".into()),
            ),
            Cell::from(s.avg_secs().map(|t| format!("{t:.1}s")).unwrap_or("–".into())),
        ])
    });
    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(24),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Fill(1),
            ],
        )
        .header(header)
        .block(Block::bordered().title(Span::styled(" By method ", bold(ACCENT)))),
        methods,
    );

    let header = Row::new(["Timed setup", "Best (correct)"]).style(bold(HILITE));
    let rows: Vec<Row> = if st.timed_best.is_empty() {
        vec![Row::new(["No timed runs yet", ""]).style(dim())]
    } else {
        st.timed_best
            .iter()
            .map(|(k, v)| Row::new([k.clone(), v.to_string()]))
            .collect()
    };
    f.render_widget(
        Table::new(rows, [Constraint::Fill(1), Constraint::Length(16)])
            .header(header)
            .block(Block::bordered().title(Span::styled(" Timed bests ", bold(ACCENT)))),
        bests,
    );
}

pub fn draw_soroban(f: &mut Frame, area: Rect, app: &App) {
    let v = app.soroban.value;
    let rods = v.to_string().len().max(7);
    let mut lines = vec![
        Line::from(vec![
            Span::raw("Value: "),
            Span::styled(v.to_string(), bold(HILITE)),
        ]),
        Line::raw(""),
    ];
    lines.extend(
        soroban::render(v, rods)
            .into_iter()
            .map(|r| Line::styled(r, bold(ACCENT))),
    );
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "Heaven bead (above the beam) = 5 when down; earth beads = 1 each when up.",
        dim(),
    ));
    lines.push(Line::raw(""));
    for s in soroban::explain(v) {
        lines.push(Line::raw(format!("  {s}")));
    }
    let h = lines.len() as u16 + 2;
    f.render_widget(
        Paragraph::new(lines)
            .centered()
            .block(Block::bordered().title(Span::styled(" Soroban visualizer ", bold(ACCENT)))),
        centered(area, 84, h),
    );
}
