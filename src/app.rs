use std::time::{Duration, Instant};

use crate::game::agent::{AgentStatus, Direction, Room};
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
    pub running: bool,
    pub tick_count: u64,
    pub show_help: bool,
    pub panel_top: Option<u16>,
    pub floor_scroll_y: u16,
    pub ceo_pos: (f32, f32),
    pub ceo_path: Vec<(u16, u16)>,
    pub ceo_returning: bool,
    frame_count: u64,
    last_fps_update: Instant,
}

impl App {
    pub fn new(floor_width: u16, floor_height: u16) -> Self {
        let state = GameState::new(floor_width, floor_height);
        let ceo_pos = (
            state.floor.ceo_chair.0 as f32,
            state.floor.ceo_chair.1 as f32,
        );
        App {
            state,
            agent_panel: AgentPanelState::new(),
            bubbles: BubbleManager::new(),
            focus: Focus::Floor,
            running: true,
            tick_count: 0,
            show_help: false,
            panel_top: None,
            floor_scroll_y: 0,
            ceo_pos,
            ceo_path: Vec::new(),
            ceo_returning: false,
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

        // Advance CEO along path
        if !self.ceo_path.is_empty() {
            advance_along_path(&mut self.ceo_pos, &mut self.ceo_path, 6.0, delta_secs);
            // When CEO arrives at whiteboard and path is empty, start returning
            if self.ceo_path.is_empty() && !self.ceo_returning {
                self.ceo_returning = true;
                let chair = self.state.floor.ceo_chair;
                // Path back: workspace → CEO office
                self.ceo_path = crate::game::pathfinding::compute_path(
                    self.ceo_pos.0 as u16,
                    self.ceo_pos.1 as u16,
                    Room::Workspace,
                    Room::CeoOffice,
                    chair.0,
                    chair.1,
                    &self.state.floor,
                );
            } else if self.ceo_path.is_empty() && self.ceo_returning {
                self.ceo_returning = false;
            }
        }

        // Snapshot all agent positions for collision checks
        let positions: Vec<(f32, f32)> = self.state.agents.iter().map(|a| a.position).collect();

        // Check if a position collides with any other agent
        let check_collision = |pos: (f32, f32), skip: usize| -> bool {
            let cx = pos.0.round() as i32;
            let cy = pos.1.round() as i32;
            positions.iter().enumerate().any(|(j, &(ox, oy))| {
                if j == skip {
                    return false;
                }
                let ocx = ox.round() as i32;
                let ocy = oy.round() as i32;
                (cx - ocx).abs() < 2 && (cy - ocy).abs() < 2
            })
        };

        // Advance agents with collision avoidance
        for i in 0..self.state.agents.len() {
            let agent = &mut self.state.agents[i];
            if agent.is_animating() {
                let old_pos = agent.position;

                advance_along_path(&mut agent.position, &mut agent.path, 4.0, delta_secs);

                if check_collision(agent.position, i) {
                    // Try nudging perpendicular to path direction
                    let dx = agent.position.0 - old_pos.0;
                    let dy = agent.position.1 - old_pos.1;
                    let nudge = 2.0;

                    // Try perpendicular directions: (-dy, dx) and (dy, -dx)
                    let try1 = (
                        old_pos.0 - dy.signum() * nudge,
                        old_pos.1 + dx.signum() * nudge,
                    );
                    let try2 = (
                        old_pos.0 + dy.signum() * nudge,
                        old_pos.1 - dx.signum() * nudge,
                    );

                    if !check_collision(try1, i) {
                        agent.position = try1;
                    } else if !check_collision(try2, i) {
                        agent.position = try2;
                    } else {
                        // Both sides blocked — wait
                        agent.position = old_pos;
                    }
                }

                if agent.position.0 > old_pos.0 {
                    agent.facing = Direction::Right;
                } else if agent.position.0 < old_pos.0 {
                    agent.facing = Direction::Left;
                }
            } else if let Some(desk_idx) = agent.assigned_desk {
                if let Some(desk) = self.state.floor.desks.get(desk_idx) {
                    agent.position = (desk.chair_x as f32, desk.chair_y as f32);
                }
            }
        }

        // Remove finished temp agents
        self.state.agents.retain(|a| {
            !(matches!(a.status, AgentStatus::Finished | AgentStatus::Error)
                && !a.is_permanent
                && !a.is_animating())
        });

        // Remove desks that no agent references (only when no one is animating)
        let anyone_animating = self.state.agents.iter().any(|a| a.is_animating());
        if anyone_animating {
            // Skip cleanup — agents are still moving, desk indices could be stale
            self.bubbles.tick();
            self.state.update_stats();
            return;
        }
        let referenced_desks: std::collections::HashSet<usize> = self
            .state
            .agents
            .iter()
            .filter_map(|a| a.assigned_desk)
            .collect();
        let occupied_indices: Vec<usize> = self
            .state
            .floor
            .desks
            .iter()
            .enumerate()
            .filter(|(i, _)| referenced_desks.contains(i))
            .map(|(i, _)| i)
            .collect();

        if occupied_indices.len() < self.state.floor.desks.len()
            && occupied_indices.len() >= crate::game::floor::MIN_DESKS
        {
            // Build index map: old_index -> new_index
            let mut index_map = vec![None; self.state.floor.desks.len()];
            for (new_idx, &old_idx) in occupied_indices.iter().enumerate() {
                index_map[old_idx] = Some(new_idx);
            }

            // Keep only occupied desks
            let new_desks: Vec<_> = occupied_indices
                .iter()
                .map(|&i| self.state.floor.desks[i].clone())
                .collect();

            // Clear old desk cells from grid
            for desk in &self.state.floor.desks {
                let w = desk.variant.width();
                for r in 0..3u16 {
                    for c in 0..w {
                        let gy = (desk.desk_y + r) as usize;
                        let gx = (desk.desk_x + c) as usize;
                        if gy < self.state.floor.height as usize
                            && gx < self.state.floor.width as usize
                        {
                            self.state.floor.grid[gy][gx] = crate::game::floor::CellType::Empty;
                        }
                    }
                }
            }

            self.state.floor.desks = new_desks;

            // Re-mark desk cells
            for desk in &self.state.floor.desks {
                let w = desk.variant.width();
                for r in 0..3u16 {
                    for c in 0..w {
                        let gy = (desk.desk_y + r) as usize;
                        let gx = (desk.desk_x + c) as usize;
                        if gy < self.state.floor.height as usize
                            && gx < self.state.floor.width as usize
                        {
                            self.state.floor.grid[gy][gx] = crate::game::floor::CellType::Desk;
                        }
                    }
                }
            }

            // Remap agent desk indices
            for agent in &mut self.state.agents {
                if let Some(old_idx) = agent.assigned_desk {
                    agent.assigned_desk = if old_idx < index_map.len() {
                        index_map[old_idx]
                    } else {
                        None
                    };
                }
            }
        }

        // Idle agents in lounge wander near furniture
        let lounge = self.state.floor.lounge;
        let ping_pong = self.state.floor.ping_pong;
        for agent in &mut self.state.agents {
            if agent.status == AgentStatus::Idle
                && agent.target_room == Room::Lounge
                && agent.path.is_empty()
            {
                // Occasionally pick a new wander target near furniture
                if self.tick_count % 90 == (agent.sprite_color.0 as u64 * 13) % 90 {
                    let targets = [
                        // Near ping pong table
                        (ping_pong.0.saturating_sub(1), ping_pong.1 + ping_pong.3 + 1),
                        (ping_pong.0 + ping_pong.2 + 1, ping_pong.1),
                        // Lounge edges
                        (lounge.0 + 3, lounge.1 + 3),
                        (lounge.0 + lounge.2 - 5, lounge.1 + lounge.3 - 3),
                        // Center of lounge
                        (lounge.0 + lounge.2 / 2, lounge.1 + lounge.3 / 2),
                        // Arcade machines (bottom-left)
                        (5, lounge.1 + lounge.3 - 5),
                        (9, lounge.1 + lounge.3 - 5),
                    ];
                    let pick = (self.tick_count / 90 + agent.sprite_color.0 as u64) as usize
                        % targets.len();
                    let (tx, ty) = targets[pick];
                    agent.path = vec![(tx, ty)];
                }
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
        if let Ok(output) = Command::new("ps")
            .args(["-o", "rss=", "-p", &pid.to_string()])
            .output()
        {
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
