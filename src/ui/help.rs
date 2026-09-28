use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};

use super::{ACCENT, GOOD, HILITE, bold, dim, level_dots};
use crate::app::App;
use crate::help_content;
use crate::methods::Method;
use crate::soroban;

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let [left, right] = Layout::horizontal([Constraint::Length(26), Constraint::Fill(1)]).areas(area);

    let mut items = vec![ListItem::new("Overview & keys")];
    items.extend(Method::ALL.iter().map(|m| ListItem::new(m.name())));
    let mut state = ListState::default().with_selected(Some(app.help.page));
    f.render_stateful_widget(
        List::new(items)
            .block(Block::bordered().title(Span::styled(" Methods ", bold(ACCENT))))
            .highlight_style(bold(HILITE))
            .highlight_symbol("▶ "),
        left,
        &mut state,
    );

    let (title, lines) = match app.help_method() {
        None => ("Overview".to_string(), flow(help_content::OVERVIEW)),
        Some(m) => (m.name().to_string(), method_page(m, app)),
    };
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((app.help.scroll, 0))
            .block(Block::bordered().title(Span::styled(format!(" {title} "), bold(ACCENT)))),
        right,
    );
}

fn method_page(m: Method, app: &App) -> Vec<Line<'static>> {
    let mut lines = flow(help_content::intro(m));
    if m == Method::Anzan {
        lines.push(Line::raw(""));
        lines.push(Line::raw(
            "The number 2,735 on a soroban (heaven beads above the beam, earth below):",
        ));
        lines.extend(
            soroban::render(2735, 4)
                .into_iter()
                .map(|r| Line::styled(format!("  {r}"), bold(ACCENT))),
        );
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("TECHNIQUES", bold(ACCENT)),
        Span::styled(
            format!(
                "   (live examples at level {}  {}; n for new ones)",
                app.help.difficulty,
                level_dots(app.help.difficulty)
            ),
            dim(),
        ),
    ]));
    for (tech, ex) in m.techniques().iter().zip(&app.help.examples) {
        lines.push(Line::raw(""));
        lines.push(Line::styled(format!("── {} ──", tech.name), bold(HILITE)));
        lines.extend(tech.rule.iter().map(|r| Line::raw(format!("  {r}"))));
        let prompt = match &ex.flash {
            Some(nums) => format!(
                "{}: {}",
                ex.prompt,
                nums.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
            ),
            None => ex.prompt.clone(),
        };
        lines.push(Line::from(vec![
            Span::styled("  Example: ", dim()),
            Span::styled(prompt, bold(ACCENT)),
        ]));
        lines.extend(ex.steps.iter().map(|s| Line::raw(format!("    {s}"))));
        let mut answer = vec![
            Span::styled("  Answer: ", dim()),
            Span::styled(ex.answer.shown.clone(), bold(GOOD)),
        ];
        if let Some(note) = ex.answer.tolerance_note() {
            answer.push(Span::styled(format!(" ({note})"), dim()));
        }
        lines.push(Line::from(answer));
    }
    lines
}

/// Join consecutive prose lines into paragraphs so they wrap to the pane
/// width. Blank lines, indented lines and ALL-CAPS headings stay as they are.
fn flow(text: &[&str]) -> Vec<Line<'static>> {
    let is_prose = |l: &str| !l.is_empty() && !l.starts_with(' ') && l.chars().any(|c| c.is_lowercase());
    let mut out: Vec<String> = Vec::new();
    let mut prev_prose = false;
    for l in text {
        let prose = is_prose(l);
        match out.last_mut() {
            Some(last) if prose && prev_prose => {
                last.push(' ');
                last.push_str(l);
            }
            _ => out.push(l.to_string()),
        }
        prev_prose = prose;
    }
    out.into_iter().map(Line::raw).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_joins_prose_only() {
        let lines = flow(&["One two", "three.", "", "HEADING", "  indented", "  kept", "Next"]);
        let text: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        assert_eq!(
            text,
            ["One two three.", "", "HEADING", "  indented", "  kept", "Next"]
        );
    }
}
