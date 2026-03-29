use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};
use std::time::{Duration, Instant};

use crate::ui::sprites;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Working,
    Idle,
    Spawning,
    Finished,
    Error,
}

#[derive(Debug, Clone)]
pub struct Bubble {
    pub agent_id: String,
    pub symbol: char,
    pub color: Color,
    pub created_at: Instant,
    pub lifetime: Duration,
}

impl Bubble {
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.lifetime
    }
}

pub struct BubbleManager {
    pub bubbles: Vec<Bubble>,
    pub max_visible: usize,
}

impl Default for BubbleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BubbleManager {
    pub fn new() -> Self {
        Self {
            bubbles: Vec::new(),
            max_visible: 3,
        }
    }

    pub fn trigger_status_change(&mut self, agent_id: impl Into<String>, new_status: AgentStatus) {
        let (symbol, color) = match new_status {
            AgentStatus::Working => (sprites::STATUS_WORKING, sprites::STATUS_WORKING_COLOR),
            AgentStatus::Idle => (sprites::STATUS_IDLE, sprites::STATUS_IDLE_COLOR),
            AgentStatus::Finished => (sprites::STATUS_FINISHED, sprites::STATUS_FINISHED_COLOR),
            AgentStatus::Error => (sprites::STATUS_ERROR, sprites::STATUS_ERROR_COLOR),
            AgentStatus::Spawning => (sprites::STATUS_SPAWNING, sprites::STATUS_SPAWNING_COLOR),
        };
        self.add_indicator(agent_id.into(), symbol, color);
    }

    pub fn trigger_tool_use(
        &mut self,
        agent_id: impl Into<String>,
        tool: &str,
        _args_hint: Option<&str>,
    ) {
        let symbol = match tool {
            "Read" => sprites::STATUS_TOOL_READ,
            "Edit" | "Write" => sprites::STATUS_TOOL_EDIT,
            "Bash" => sprites::STATUS_TOOL_BASH,
            "Grep" | "Glob" => sprites::STATUS_TOOL_SEARCH,
            "Agent" => sprites::STATUS_SPAWNING,
            _ => return,
        };
        self.add_indicator(agent_id.into(), symbol, sprites::STATUS_TOOL_COLOR);
    }

    pub fn tick(&mut self) {
        self.bubbles.retain(|b| !b.is_expired());
    }

    pub fn render_bubble_at(
        &self,
        agent_id: &str,
        agent_x: u16,
        agent_y: u16,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let bubble = match self.bubbles.iter().find(|b| b.agent_id == agent_id) {
            Some(b) => b,
            None => return,
        };

        // Show symbol 2 rows below agent (below the 2-tall sprite)
        let sx = area.x + agent_x;
        let sy = area.y + agent_y + 2;

        if sx < area.x + area.width && sy >= area.y && sy < area.y + area.height {
            if let Some(cell) = buf.cell_mut((sx, sy)) {
                cell.set_symbol(&bubble.symbol.to_string());
                cell.set_style(Style::default().fg(bubble.color));
            }
        }
    }

    // Internal helpers

    fn add_indicator(&mut self, agent_id: String, symbol: char, color: Color) {
        self.bubbles.retain(|b| b.agent_id != agent_id);

        let mut rng = rand::thread_rng();
        let ms: u64 = 2000 + rand::Rng::gen_range(&mut rng, 0u64..1500);
        let lifetime = Duration::from_millis(ms);

        self.bubbles.push(Bubble {
            agent_id,
            symbol,
            color,
            created_at: Instant::now(),
            lifetime,
        });

        while self.bubbles.len() > self.max_visible {
            self.bubbles.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_bubble_expires() {
        let b = Bubble {
            agent_id: "a1".to_string(),
            symbol: '⚙',
            color: Color::White,
            created_at: Instant::now() - Duration::from_secs(10),
            lifetime: Duration::from_secs(5),
        };
        assert!(b.is_expired());
    }

    #[test]
    fn test_bubble_not_expired() {
        let b = Bubble {
            agent_id: "a1".to_string(),
            symbol: '⚙',
            color: Color::White,
            created_at: Instant::now(),
            lifetime: Duration::from_secs(5),
        };
        assert!(!b.is_expired());
    }

    #[test]
    fn test_max_visible_enforced() {
        let mut mgr = BubbleManager::new(); // max_visible = 3
        mgr.add_indicator("a1".to_string(), '⚙', Color::White);
        mgr.add_indicator("a2".to_string(), '✓', Color::Green);
        mgr.add_indicator("a3".to_string(), '✗', Color::Red);
        mgr.add_indicator("a4".to_string(), '★', Color::Yellow);
        assert_eq!(mgr.bubbles.len(), 3);
    }

    #[test]
    fn test_one_indicator_per_agent() {
        let mut mgr = BubbleManager::new();
        mgr.add_indicator("a1".to_string(), '⚙', Color::White);
        mgr.add_indicator("a1".to_string(), '✓', Color::Green);
        let agent_bubbles: Vec<_> = mgr.bubbles.iter().filter(|b| b.agent_id == "a1").collect();
        assert_eq!(agent_bubbles.len(), 1);
        assert_eq!(agent_bubbles[0].symbol, '✓');
    }

    #[test]
    fn test_tick_removes_expired() {
        let mut mgr = BubbleManager::new();
        // Add an already-expired bubble
        mgr.bubbles.push(Bubble {
            agent_id: "old".to_string(),
            symbol: '◦',
            color: Color::Gray,
            created_at: Instant::now() - Duration::from_secs(10),
            lifetime: Duration::from_secs(1),
        });
        // Add a fresh bubble
        mgr.add_indicator("new".to_string(), '⚙', Color::White);
        assert_eq!(mgr.bubbles.len(), 2);
        mgr.tick();
        assert_eq!(mgr.bubbles.len(), 1);
        assert_eq!(mgr.bubbles[0].agent_id, "new");
    }
}
