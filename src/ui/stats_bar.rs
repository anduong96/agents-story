use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::game::state::Stats;

pub struct StatsBar<'a> {
    pub stats: &'a Stats,
}

impl<'a> StatsBar<'a> {
    pub fn new(stats: &'a Stats) -> Self {
        StatsBar { stats }
    }
}

pub fn format_tokens(tokens: u64) -> String {
    if tokens < 1_000 {
        format!("{}", tokens)
    } else if tokens < 1_000_000 {
        let k = tokens as f64 / 1_000.0;
        if k == k.floor() {
            format!("{}k", k as u64)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        let m = tokens as f64 / 1_000_000.0;
        if m == m.floor() {
            format!("{}M", m as u64)
        } else {
            format!("{:.1}M", m)
        }
    }
}

pub fn cost_color(cost: f64) -> Color {
    if cost < 1.0 {
        Color::Green
    } else if cost < 5.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

pub fn usage_color(usage: f32) -> Color {
    if usage < 50.0 {
        Color::Green
    } else if usage < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn agents_color(active: usize, total: usize) -> Color {
    // Green if all agents are healthy (no implied errors — just report green if active <= total)
    // Yellow if active > total (shouldn't happen) or if something looks off
    if active <= total {
        Color::Green
    } else {
        Color::Yellow
    }
}

impl<'a> Widget for StatsBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let s = self.stats;

        let sep = Span::styled(" │ ", Style::default().fg(Color::DarkGray));

        let model_span = Span::styled(
            format!(" Model: {}", s.model),
            Style::default().fg(Color::White),
        );

        let agents_color = agents_color(s.active_agents, s.total_agents);
        let agents_span = Span::styled(
            format!("Agents: {}/{}", s.active_agents, s.total_agents),
            Style::default().fg(agents_color),
        );

        let tasks_span = Span::styled(
            format!("Tasks: {}/{}", s.completed_tasks, s.total_tasks),
            Style::default().fg(Color::White),
        );

        let tokens_span = Span::styled(
            format!("Tokens: {}", format_tokens(s.total_tokens)),
            Style::default().fg(Color::Cyan),
        );

        let cost_span = Span::styled(
            format!("Cost: ${:.2}", s.total_cost),
            Style::default().fg(cost_color(s.total_cost)),
        );

        let usage_span = Span::styled(
            format!("Usage: {:.0}%", s.usage_percent),
            Style::default().fg(usage_color(s.usage_percent)),
        );

        let fps_span = Span::styled(
            format!("FPS: {}", s.fps),
            Style::default().fg(Color::DarkGray),
        );

        let ram_span = Span::styled(
            format!("RAM: {:.1}MB", s.ram_mb),
            Style::default().fg(Color::DarkGray),
        );

        let hotkeys = Span::styled(
            " q:Quit ?:Help Tab:Focus j/k:Nav",
            Style::default().fg(Color::Rgb(100, 100, 110)),
        );

        let line = Line::from(vec![
            model_span,
            sep.clone(),
            agents_span,
            sep.clone(),
            tasks_span,
            sep.clone(),
            tokens_span,
            sep.clone(),
            cost_span,
            sep.clone(),
            usage_span,
            sep.clone(),
            fps_span,
            sep.clone(),
            ram_span,
            sep.clone(),
            hotkeys,
        ]);

        line.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tokens_small() {
        assert_eq!(format_tokens(500), "500");
    }

    #[test]
    fn format_tokens_thousands() {
        assert_eq!(format_tokens(14_200), "14.2k");
    }

    #[test]
    fn format_tokens_millions() {
        assert_eq!(format_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn cost_color_green() {
        assert_eq!(cost_color(0.42), Color::Green);
    }

    #[test]
    fn cost_color_yellow() {
        assert_eq!(cost_color(2.50), Color::Yellow);
    }

    #[test]
    fn cost_color_red() {
        assert_eq!(cost_color(10.0), Color::Red);
    }

    #[test]
    fn cost_color_boundary_one() {
        // exactly $1.00 is yellow
        assert_eq!(cost_color(1.0), Color::Yellow);
    }

    #[test]
    fn usage_color_green() {
        assert_eq!(usage_color(30.0), Color::Green);
    }

    #[test]
    fn usage_color_yellow() {
        assert_eq!(usage_color(67.0), Color::Yellow);
    }

    #[test]
    fn usage_color_red() {
        assert_eq!(usage_color(90.0), Color::Red);
    }
}
