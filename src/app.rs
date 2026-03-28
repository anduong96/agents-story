use std::time::{Duration, Instant};

use crate::game::agent::{Direction, Room};
use crate::game::pathfinding::advance_along_path;
use crate::game::state::GameState;
use crate::ui::agent_panel::AgentPanelState;
use crate::ui::bubbles::BubbleManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Floor,
    AgentPanel,
}

pub struct App {
    pub state: GameState,
    pub agent_panel: AgentPanelState,
    pub bubbles: BubbleManager,
    pub focus: Focus,
    pub highlighted_room: Option<Room>,
    pub running: bool,
    pub tick_count: u64,
    pub show_help: bool,
    frame_count: u64,
    last_fps_update: Instant,
}

impl App {
    pub fn new(floor_width: u16, floor_height: u16) -> Self {
        App {
            state: GameState::new(floor_width, floor_height),
            agent_panel: AgentPanelState::new(),
            bubbles: BubbleManager::new(),
            focus: Focus::Floor,
            highlighted_room: None,
            running: true,
            tick_count: 0,
            show_help: false,
            frame_count: 0,
            last_fps_update: Instant::now(),
        }
    }

    /// Resize the floor to fill the given pane dimensions.
    pub fn resize_floor(&mut self, width: u16, height: u16) {
        if width == self.state.floor.width && height == self.state.floor.height {
            return;
        }
        // Only resize if no agents are active (avoid disrupting pathfinding)
        if self.state.agents.is_empty() {
            self.state = GameState::new(width, height);
        }
    }

    /// Target frames per second: fast when animating, slow when idle.
    pub fn target_fps(&self) -> u64 {
        if self.state.has_animations() || !self.bubbles.bubbles.is_empty() {
            15
        } else {
            2
        }
    }

    /// Duration of a single frame based on the current target FPS.
    pub fn frame_duration(&self) -> Duration {
        Duration::from_millis(1000 / self.target_fps())
    }

    /// Toggle focus between Floor and AgentPanel.
    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Floor => Focus::AgentPanel,
            Focus::AgentPanel => Focus::Floor,
        };
    }

    /// Advance simulation by one tick.
    pub fn tick(&mut self, delta_secs: f32) {
        self.tick_count += 1;
        self.frame_count += 1;

        // Update FPS every second
        let elapsed = self.last_fps_update.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.state.stats.fps = self.frame_count * 1000 / elapsed.as_millis().max(1) as u64;
            self.frame_count = 0;
            self.last_fps_update = Instant::now();

            // Update RAM usage (process RSS)
            self.state.stats.ram_mb = get_rss_mb();
        }

        // Advance each agent along its path and update facing direction.
        for agent in &mut self.state.agents {
            let prev_x = agent.position.0;
            advance_along_path(&mut agent.position, &mut agent.path, 4.0, delta_secs);
            let new_x = agent.position.0;

            // Update facing based on horizontal movement.
            if new_x > prev_x {
                agent.facing = Direction::Right;
            } else if new_x < prev_x {
                agent.facing = Direction::Left;
            }
        }

        self.bubbles.tick();
        self.state.update_stats();
    }
}

/// Get current process RSS in MB (macOS/Linux).
fn get_rss_mb() -> f64 {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let pid = std::process::id();
        if let Ok(output) = Command::new("ps").args(["-o", "rss=", "-p", &pid.to_string()]).output() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Ok(kb) = s.trim().parse::<f64>() {
                    return kb / 1024.0;
                }
            }
        }
        0.0
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(kb_str) = parts.get(1) {
                        if let Ok(kb) = kb_str.parse::<f64>() {
                            return kb / 1024.0;
                        }
                    }
                }
            }
        }
        0.0
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        0.0
    }
}
