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
    #[allow(dead_code)]
    pub scroll_offset: usize,
}

impl AgentPanelState {
    pub fn new() -> Self {
        AgentPanelState {
            selected: None,
            expanded: None,
            scroll_offset: 0,
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

impl<'a> StatefulWidget for AgentPanel<'a> {
    type State = AgentPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AgentPanelState) {
        let block = Block::default().title(" Agents ").borders(Borders::ALL);

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let mut y = inner.top();

        for (idx, agent) in self.agents.iter().enumerate() {
            if y >= inner.bottom() {
                break;
            }

            let is_selected = state.selected == Some(idx);
            let is_expanded = state.expanded == Some(idx);

            let arrow = if is_expanded {
                "▼ "
            } else if is_selected {
                "▶ "
            } else {
                "  "
            };

            let (indicator, use_agent_color) = status_indicator(&agent.status);
            let indicator_color = if use_agent_color {
                agent.sprite_color.to_color()
            } else {
                Color::Red
            };
            let label = status_label(&agent.status);
            let label_color = status_label_color(&agent.status);

            let location = {
                let base = format!("{} @ {}", agent.session.branch, agent.session.repo);
                if let Some(ref wt) = agent.session.worktree {
                    format!("{} ({})", base, wt)
                } else {
                    base
                }
            };

            let task_text = agent
                .task
                .as_deref()
                .map(|t| format!(" \"{}\"", t))
                .unwrap_or_default();

            // Build the collapsed line
            let name_style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let collapsed_line = Line::from(vec![
                Span::raw(arrow),
                Span::styled(indicator, Style::default().fg(indicator_color)),
                Span::raw(" "),
                Span::styled(agent.name.clone(), name_style),
                Span::raw(" "),
                Span::styled(label, Style::default().fg(label_color)),
                Span::raw(task_text),
                Span::raw(" "),
                Span::styled(location, Style::default().fg(Color::DarkGray)),
            ]);

            let row_area = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };
            collapsed_line.render(row_area, buf);
            y += 1;

            // Expanded detail lines
            if is_expanded {
                // Line 1: Model / Tokens / Cost
                if y < inner.bottom() {
                    let detail1 = Line::from(vec![
                        Span::raw("    "),
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
                    ]);
                    let detail1_area = Rect {
                        x: inner.x,
                        y,
                        width: inner.width,
                        height: 1,
                    };
                    detail1.render(detail1_area, buf);
                    y += 1;
                }

                // Line 2: Current tool
                if y < inner.bottom() {
                    let tool_text = agent.current_tool.as_deref().unwrap_or("(none)");
                    let detail2 = Line::from(vec![
                        Span::raw("    "),
                        Span::styled("Tool: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(tool_text, Style::default().fg(Color::Magenta)),
                    ]);
                    let detail2_area = Rect {
                        x: inner.x,
                        y,
                        width: inner.width,
                        height: 1,
                    };
                    detail2.render(detail2_area, buf);
                    y += 1;
                }

                // Line 3: Duration
                if y < inner.bottom() {
                    let elapsed = agent.started_at.elapsed();
                    let detail3 = Line::from(vec![
                        Span::raw("    "),
                        Span::styled("Duration: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format_duration(elapsed), Style::default().fg(Color::White)),
                    ]);
                    let detail3_area = Rect {
                        x: inner.x,
                        y,
                        width: inner.width,
                        height: 1,
                    };
                    detail3.render(detail3_area, buf);
                    y += 1;
                }
            }
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
