use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, StatefulWidget, Widget},
};

use crate::game::agent::{Agent, AgentStatus};

/// State for the agent panel (selection, expansion, scroll).
#[derive(Debug, Default)]
pub struct AgentPanelState {
    pub selected: Option<usize>,
    pub expanded: Option<usize>,
    pub scroll_offset: usize,
    /// Total rendered lines from last frame (used for scroll bounds).
    pub total_lines: usize,
    /// Visible height from last frame.
    pub visible_height: usize,
}

impl AgentPanelState {
    pub fn new() -> Self {
        AgentPanelState {
            selected: None,
            expanded: None,
            scroll_offset: 0,
            total_lines: 0,
            visible_height: 0,
        }
    }

    pub fn select_next(&mut self, agent_count: usize) {
        if agent_count == 0 {
            self.selected = None;
            return;
        }
        self.selected = Some(match self.selected {
            None => 0,
            Some(i) => (i + 1) % agent_count,
        });
    }

    pub fn select_prev(&mut self, agent_count: usize) {
        if agent_count == 0 {
            self.selected = None;
            return;
        }
        self.selected = Some(match self.selected {
            None => agent_count - 1,
            Some(0) => agent_count - 1,
            Some(i) => i - 1,
        });
    }

    pub fn toggle_expand(&mut self) {
        match self.selected {
            None => {}
            Some(i) => {
                if self.expanded == Some(i) {
                    self.expanded = None;
                } else {
                    self.expanded = Some(i);
                }
            }
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        let max = self.total_lines.saturating_sub(self.visible_height);
        self.scroll_offset = (self.scroll_offset + 1).min(max);
    }

    /// Ensure the selected agent's row is visible.
    fn ensure_visible(&mut self, selected_row: usize) {
        if selected_row < self.scroll_offset {
            self.scroll_offset = selected_row;
        } else if selected_row >= self.scroll_offset + self.visible_height {
            self.scroll_offset = selected_row.saturating_sub(self.visible_height - 1);
        }
    }
}

/// Returns (indicator_char, use_agent_color). If use_agent_color is false, use the returned color.
fn status_indicator(status: &AgentStatus) -> (&'static str, bool) {
    match status {
        AgentStatus::Working => ("●", true),  // filled, agent color
        AgentStatus::Spawning => ("●", true), // filled, agent color
        AgentStatus::Error => ("●", false),   // filled, red
        AgentStatus::Idle => ("○", true),     // outline, agent color
        AgentStatus::Finished => ("○", true), // outline, agent color
    }
}

fn status_label(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Working => "WORKING",
        AgentStatus::Idle => "IDLE",
        AgentStatus::Spawning => "SPAWNING",
        AgentStatus::Finished => "FINISHED",
        AgentStatus::Error => "ERROR",
    }
}

fn status_label_color(status: &AgentStatus) -> Color {
    match status {
        AgentStatus::Working => Color::Green,
        AgentStatus::Idle => Color::Gray,
        AgentStatus::Spawning => Color::Yellow,
        AgentStatus::Finished => Color::DarkGray,
        AgentStatus::Error => Color::Red,
    }
}

fn format_duration(elapsed: std::time::Duration) -> String {
    let secs = elapsed.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// The agent panel widget.
pub struct AgentPanel<'a> {
    pub agents: &'a [Agent],
}

impl<'a> AgentPanel<'a> {
    pub fn new(agents: &'a [Agent]) -> Self {
        AgentPanel { agents }
    }
}

/// Collect unique project names in the order they first appear.
fn project_order(agents: &[Agent]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut order = Vec::new();
    for agent in agents {
        let proj = if agent.session.repo.is_empty() {
            "Staff".to_string()
        } else {
            agent.session.repo.clone()
        };
        if seen.insert(proj.clone()) {
            order.push(proj);
        }
    }
    order
}

/// Build all lines for the panel, returning (lines, selected_row).
/// `selected_row` is the line index of the selected agent's collapsed row.
fn build_lines<'b>(agents: &'b [Agent], state: &AgentPanelState) -> (Vec<Line<'b>>, Option<usize>) {
    let mut lines: Vec<Line<'b>> = Vec::new();
    let mut selected_row = None;
    let projects = project_order(agents);

    for project in &projects {
        lines.push(Line::from(vec![Span::styled(
            format!("── {} ──", project),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));

        for (idx, agent) in agents.iter().enumerate() {
            let agent_project = if agent.session.repo.is_empty() {
                "Staff"
            } else {
                &agent.session.repo
            };
            if agent_project != project {
                continue;
            }

            let is_selected = state.selected == Some(idx);
            let is_expanded = state.expanded == Some(idx);

            if is_selected {
                selected_row = Some(lines.len());
            }

            let arrow = if is_expanded {
                "  ▼ "
            } else if is_selected {
                "  ▶ "
            } else {
                "    "
            };

            let (indicator, use_agent_color) = status_indicator(&agent.status);
            let indicator_color = if use_agent_color {
                agent.sprite_color.to_color()
            } else {
                Color::Red
            };
            let label = status_label(&agent.status);
            let label_color = status_label_color(&agent.status);

            let task_text = agent
                .task
                .as_deref()
                .map(|t| format!(" \"{}\"", t))
                .unwrap_or_default();

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::raw(arrow),
                Span::styled(indicator, Style::default().fg(indicator_color)),
                Span::raw(" "),
                Span::styled(agent.name.clone(), name_style),
                Span::raw(" "),
                Span::styled(label, Style::default().fg(label_color)),
                Span::raw(task_text),
            ]));

            if is_expanded {
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled("Model: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(agent.model.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw("  "),
                    Span::styled("Tokens: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        crate::ui::stats_bar::format_tokens(agent.tokens),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw("  "),
                    Span::styled("Cost: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("${:.4}", agent.cost),
                        Style::default().fg(crate::ui::stats_bar::cost_color(agent.cost)),
                    ),
                ]));

                let tool_text = agent.current_tool.as_deref().unwrap_or("(none)");
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled("Tool: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(tool_text, Style::default().fg(Color::Magenta)),
                ]));

                let elapsed = agent.started_at.elapsed();
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled("Duration: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format_duration(elapsed), Style::default().fg(Color::White)),
                ]));
            }
        }
    }
    (lines, selected_row)
}

impl<'a> StatefulWidget for AgentPanel<'a> {
    type State = AgentPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AgentPanelState) {
        let block = Block::default().title(" Agents ").borders(Borders::ALL);
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let visible_h = inner.height as usize;
        let (lines, selected_row) = build_lines(self.agents, state);

        // Update scroll state.
        state.total_lines = lines.len();
        state.visible_height = visible_h;
        if let Some(row) = selected_row {
            state.ensure_visible(row);
        }
        let max_scroll = lines.len().saturating_sub(visible_h);
        state.scroll_offset = state.scroll_offset.min(max_scroll);

        // Render visible lines.
        let visible = lines.into_iter().skip(state.scroll_offset).take(visible_h);

        for (i, line) in visible.enumerate() {
            let y = inner.top() + i as u16;
            let row_area = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };
            line.render(row_area, buf);
        }

        // Scroll indicators.
        if state.scroll_offset > 0 {
            buf.set_string(
                inner.right().saturating_sub(1),
                inner.top(),
                "▲",
                Style::default().fg(Color::DarkGray),
            );
        }
        if state.scroll_offset < max_scroll {
            buf.set_string(
                inner.right().saturating_sub(1),
                inner.bottom().saturating_sub(1),
                "▼",
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_next_wraps() {
        let mut state = AgentPanelState::new();
        state.selected = Some(4);
        state.select_next(5);
        assert_eq!(state.selected, Some(0));
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = AgentPanelState::new();
        state.selected = Some(0);
        state.select_prev(5);
        assert_eq!(state.selected, Some(4));
    }

    #[test]
    fn select_next_from_none() {
        let mut state = AgentPanelState::new();
        state.select_next(3);
        assert_eq!(state.selected, Some(0));
    }

    #[test]
    fn toggle_expand_sets_and_clears() {
        let mut state = AgentPanelState::new();
        state.selected = Some(2);
        state.toggle_expand();
        assert_eq!(state.expanded, Some(2));
        state.toggle_expand();
        assert_eq!(state.expanded, None);
    }

    #[test]
    fn empty_agents_select_next() {
        let mut state = AgentPanelState::new();
        state.select_next(0);
        assert_eq!(state.selected, None);
    }
}
