use rand::seq::SliceRandom;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};
use std::time::{Duration, Instant};

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
    pub text: String,
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

impl BubbleManager {
    pub fn new() -> Self {
        Self {
            bubbles: Vec::new(),
            max_visible: 3,
        }
    }

    pub fn trigger_status_change(&mut self, agent_id: impl Into<String>, new_status: AgentStatus) {
        let pool: &[&str] = match new_status {
            AgentStatus::Working => &["on it!", "let's go", "diving in...", "focus time"],
            AgentStatus::Idle => &["break time", "brb", "coffee...", "stretching"],
            AgentStatus::Finished => &["shipped!", "done!", "nailed it", "ez"],
            AgentStatus::Error => &["ugh...", "hmm...", "not good", "help?"],
            AgentStatus::Spawning => &["reporting in", "hello!", "ready"],
        };
        self.add_bubble(agent_id.into(), pool);
    }

    pub fn trigger_tool_use(
        &mut self,
        agent_id: impl Into<String>,
        tool: &str,
        args_hint: Option<&str>,
    ) {
        let agent_id = agent_id.into();
        let text = match tool {
            "Read" => {
                let filename = args_hint
                    .map(short_filename)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("...");
                if args_hint.is_some() {
                    format!("reading {}", filename)
                } else {
                    "reading...".to_string()
                }
            }
            "Edit" | "Write" => {
                let filename = args_hint
                    .map(short_filename)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("...");
                if args_hint.is_some() {
                    if tool == "Edit" {
                        format!("editing {}", filename)
                    } else {
                        format!("writing {}", filename)
                    }
                } else if tool == "Edit" {
                    "editing...".to_string()
                } else {
                    "writing...".to_string()
                }
            }
            "Bash" => {
                let is_test = args_hint
                    .map(|a| a.contains("test"))
                    .unwrap_or(false);
                if is_test {
                    "fingers crossed".to_string()
                } else {
                    "running cmd...".to_string()
                }
            }
            "Grep" | "Glob" => "searching...".to_string(),
            "Agent" => "calling backup".to_string(),
            _ => return, // Unknown tools → no bubble
        };
        self.add_bubble_text(agent_id, text);
    }

    pub fn trigger_lounge_arrival(&mut self, agent_id: impl Into<String>) {
        const POOL: &[&str] = &["ping pong?", "nice", "chill", "ahh"];
        self.add_bubble(agent_id.into(), POOL);
    }

    pub fn trigger_long_task(&mut self, agent_id: impl Into<String>) {
        const POOL: &[&str] = &["still going...", "almost...", "deep in it", "hang on"];
        self.add_bubble(agent_id.into(), POOL);
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

        let text = &bubble.text;
        let inner_width = text.len() as u16;
        let box_width = inner_width + 4; // "│ " + text + " │"

        // Bubble is 3 rows tall: top border, text, bottom border + stem
        // Stem row is 1 more below = total 4 rows above agent_y
        // We place the bubble so the stem points at (agent_x, agent_y - 1)
        // Layout:
        //   row 0 (agent_y - 4): ┌───┐
        //   row 1 (agent_y - 3): │ text │
        //   row 2 (agent_y - 2): └──┬──┘
        //   row 3 (agent_y - 1): (space with ┆ below stem position)
        // Actually simpler: place bubble 3 rows above agent

        if agent_y < 3 {
            return; // Not enough room above
        }

        let bubble_top = agent_y - 3;

        // Calculate left edge, clamping to area bounds
        let stem_col = agent_x;
        // Default: center box around stem_col
        let mut box_left = stem_col.saturating_sub(box_width / 2);
        // Clamp right edge
        if box_left + box_width > area.x + area.width {
            box_left = (area.x + area.width).saturating_sub(box_width);
        }
        // Clamp left edge
        if box_left < area.x {
            box_left = area.x;
        }

        let row_top = area.y + bubble_top;
        let row_mid = area.y + bubble_top + 1;
        let row_bot = area.y + bubble_top + 2;

        // Guard rows within area
        if row_bot >= area.y + area.height {
            return;
        }

        let style = Style::default().fg(Color::White);

        // Top border: ┌───────────┐
        if let Some(cell) = buf.cell_mut((box_left, row_top)) {
            cell.set_symbol("┌");
            cell.set_style(style);
        }
        for i in 1..box_width - 1 {
            if let Some(cell) = buf.cell_mut((box_left + i, row_top)) {
                cell.set_symbol("─");
                cell.set_style(style);
            }
        }
        if let Some(cell) = buf.cell_mut((box_left + box_width - 1, row_top)) {
            cell.set_symbol("┐");
            cell.set_style(style);
        }

        // Middle row: │ text │
        if let Some(cell) = buf.cell_mut((box_left, row_mid)) {
            cell.set_symbol("│");
            cell.set_style(style);
        }
        if let Some(cell) = buf.cell_mut((box_left + 1, row_mid)) {
            cell.set_symbol(" ");
            cell.set_style(style);
        }
        for (i, ch) in text.chars().enumerate() {
            if let Some(cell) = buf.cell_mut((box_left + 2 + i as u16, row_mid)) {
                cell.set_symbol(&ch.to_string());
                cell.set_style(style);
            }
        }
        if let Some(cell) = buf.cell_mut((box_left + 2 + inner_width, row_mid)) {
            cell.set_symbol(" ");
            cell.set_style(style);
        }
        if let Some(cell) = buf.cell_mut((box_left + box_width - 1, row_mid)) {
            cell.set_symbol("│");
            cell.set_style(style);
        }

        // Bottom border with stem: └─────┬─────┘
        // Stem position within box
        let stem_offset = stem_col.saturating_sub(box_left);
        let clamped_stem = stem_offset.clamp(1, box_width - 2);

        if let Some(cell) = buf.cell_mut((box_left, row_bot)) {
            cell.set_symbol("└");
            cell.set_style(style);
        }
        for i in 1..box_width - 1 {
            let sym = if i == clamped_stem { "┬" } else { "─" };
            if let Some(cell) = buf.cell_mut((box_left + i, row_bot)) {
                cell.set_symbol(sym);
                cell.set_style(style);
            }
        }
        if let Some(cell) = buf.cell_mut((box_left + box_width - 1, row_bot)) {
            cell.set_symbol("┘");
            cell.set_style(style);
        }
    }

    // Internal helpers

    fn add_bubble(&mut self, agent_id: String, pool: &[&str]) {
        let mut rng = rand::thread_rng();
        if let Some(text) = pool.choose(&mut rng) {
            self.add_bubble_text(agent_id, text.to_string());
        }
    }

    fn add_bubble_text(&mut self, agent_id: String, text: String) {
        // Remove existing bubble for this agent
        self.bubbles.retain(|b| b.agent_id != agent_id);

        // Random lifetime 3000–5000ms
        let mut rng = rand::thread_rng();
        let ms: u64 = 3000 + rand::Rng::gen_range(&mut rng, 0u64..2001);
        let lifetime = Duration::from_millis(ms);

        self.bubbles.push(Bubble {
            agent_id,
            text,
            created_at: Instant::now(),
            lifetime,
        });

        // Enforce max_visible: remove oldest if over limit
        while self.bubbles.len() > self.max_visible {
            self.bubbles.remove(0);
        }
    }
}

/// Returns the last path component after '/'.
pub fn short_filename(path: &str) -> &str {
    path.rfind('/').map(|i| &path[i + 1..]).unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_bubble_expires() {
        let b = Bubble {
            agent_id: "a1".to_string(),
            text: "hello".to_string(),
            created_at: Instant::now() - Duration::from_secs(10),
            lifetime: Duration::from_secs(5),
        };
        assert!(b.is_expired());
    }

    #[test]
    fn test_bubble_not_expired() {
        let b = Bubble {
            agent_id: "a1".to_string(),
            text: "hello".to_string(),
            created_at: Instant::now(),
            lifetime: Duration::from_secs(5),
        };
        assert!(!b.is_expired());
    }

    #[test]
    fn test_max_visible_enforced() {
        let mut mgr = BubbleManager::new(); // max_visible = 3
        mgr.add_bubble_text("a1".to_string(), "one".to_string());
        mgr.add_bubble_text("a2".to_string(), "two".to_string());
        mgr.add_bubble_text("a3".to_string(), "three".to_string());
        mgr.add_bubble_text("a4".to_string(), "four".to_string());
        assert_eq!(mgr.bubbles.len(), 3);
    }

    #[test]
    fn test_one_bubble_per_agent() {
        let mut mgr = BubbleManager::new();
        mgr.add_bubble_text("a1".to_string(), "first".to_string());
        mgr.add_bubble_text("a1".to_string(), "second".to_string());
        let agent_bubbles: Vec<_> = mgr.bubbles.iter().filter(|b| b.agent_id == "a1").collect();
        assert_eq!(agent_bubbles.len(), 1);
        assert_eq!(agent_bubbles[0].text, "second");
    }

    #[test]
    fn test_tick_removes_expired() {
        let mut mgr = BubbleManager::new();
        // Add an already-expired bubble
        mgr.bubbles.push(Bubble {
            agent_id: "old".to_string(),
            text: "stale".to_string(),
            created_at: Instant::now() - Duration::from_secs(10),
            lifetime: Duration::from_secs(1),
        });
        // Add a fresh bubble
        mgr.add_bubble_text("new".to_string(), "fresh".to_string());
        assert_eq!(mgr.bubbles.len(), 2);
        mgr.tick();
        assert_eq!(mgr.bubbles.len(), 1);
        assert_eq!(mgr.bubbles[0].agent_id, "new");
    }

    #[test]
    fn test_short_filename() {
        assert_eq!(short_filename("/Users/me/project/src/main.rs"), "main.rs");
        assert_eq!(short_filename("main.rs"), "main.rs");
        assert_eq!(short_filename("/a/b/c"), "c");
    }
}
