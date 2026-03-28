# Agents Story Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Game Dev Story-inspired TUI visualizer that renders an office where AI agents move between rooms based on real-time Claude Code JSONL streaming data.

**Architecture:** Tick-based game loop (15 FPS active / 2 FPS idle) running on a tokio runtime. Background async tasks discover Claude Code sessions and read their JSONL streams, sending events over mpsc channels to the main thread which updates game state and renders via Ratatui.

**Tech Stack:** Rust, Ratatui, Crossterm, Tokio, Serde, Notify

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Dependencies and project metadata |
| `src/main.rs` | Entry point, tokio runtime, tick loop |
| `src/app.rs` | App struct: holds GameState, channels, focus, frame rate |
| `src/game/mod.rs` | Re-exports game types |
| `src/game/agent.rs` | Agent, AgentStatus, Direction, SpriteVariant, SessionInfo |
| `src/game/floor.rs` | Floor grid, Room enum, CellType, layout generation, desk/door positions |
| `src/game/pathfinding.rs` | Waypoint graph, compute_path between rooms |
| `src/game/state.rs` | GameState: agents vec, stats, bubble manager, tick update |
| `src/stream/mod.rs` | Re-exports stream types |
| `src/stream/protocol.rs` | StreamEvent enum, parse_line() |
| `src/stream/reader.rs` | Async JSONL line reader per session |
| `src/stream/discovery.rs` | Watch ~/.claude/ for active sessions |
| `src/ui/mod.rs` | Top-level render function, composes all UI sections |
| `src/ui/floor_view.rs` | Render rooms, walls, objects, agent sprites on the grid |
| `src/ui/sprites.rs` | Braille/half-block art constants for agents, desks, monitors, ping pong table |
| `src/ui/stats_bar.rs` | Render the stats line with color coding |
| `src/ui/agent_panel.rs` | Render scrollable agent list with collapsed/expanded views |
| `src/ui/bubbles.rs` | Bubble struct, BubbleManager, trigger logic, render |
| `src/input.rs` | Keyboard/mouse event handling, focus cycling |
| `src/demo.rs` | Demo mode: synthetic events for testing without Claude Code |

---

### Task 1: Project Scaffold & Hello World TUI

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "agents-story"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
notify = "7"
rand = "0.8"
```

- [ ] **Step 2: Create minimal main.rs**

```rust
use std::io;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let block = Block::default()
                .title(" Agents Story ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let paragraph = Paragraph::new("Press 'q' to quit")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
```

- [ ] **Step 3: Build and run**

Run: `cargo build`
Expected: Compiles with no errors.

Run: `cargo run`
Expected: A bordered box with "Agents Story" title appears. Press 'q' to quit cleanly.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: scaffold project with hello world TUI"
```

---

### Task 2: Core Game Types

**Files:**
- Create: `src/game/mod.rs`
- Create: `src/game/agent.rs`
- Create: `src/game/floor.rs`
- Create: `src/game/state.rs`
- Modify: `src/main.rs` (add `mod game;`)

- [ ] **Step 1: Create game/mod.rs**

```rust
pub mod agent;
pub mod floor;
pub mod state;
```

- [ ] **Step 2: Create game/agent.rs with core types**

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Working,
    Idle,
    Spawning,
    Finished,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Room {
    Workspace,
    Lounge,
    CeoOffice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteColor(pub u8);

impl SpriteColor {
    pub const GREEN: Self = Self(0);
    pub const CYAN: Self = Self(1);
    pub const MAGENTA: Self = Self(2);
    pub const YELLOW: Self = Self(3);
    pub const BLUE: Self = Self(4);
    pub const RED: Self = Self(5);
    pub const WHITE: Self = Self(6);
    pub const ORANGE: Self = Self(7);

    const PALETTE: [ratatui::style::Color; 8] = [
        ratatui::style::Color::Green,
        ratatui::style::Color::Cyan,
        ratatui::style::Color::Magenta,
        ratatui::style::Color::Yellow,
        ratatui::style::Color::Blue,
        ratatui::style::Color::Red,
        ratatui::style::Color::White,
        ratatui::style::Color::Rgb(255, 165, 0),
    ];

    pub fn from_index(index: usize) -> Self {
        Self((index % Self::PALETTE.len()) as u8)
    }

    pub fn to_color(self) -> ratatui::style::Color {
        Self::PALETTE[self.0 as usize % Self::PALETTE.len()]
    }
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub repo: String,
    pub branch: String,
    pub worktree: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub model: String,
    pub task: Option<String>,
    pub current_tool: Option<String>,
    pub session: SessionInfo,
    pub tokens: u64,
    pub cost: f64,
    pub started_at: std::time::Instant,

    // Visual state
    pub position: (f32, f32),
    pub target_room: Room,
    pub path: Vec<(u16, u16)>,
    pub assigned_desk: Option<usize>,
    pub sprite_color: SpriteColor,
    pub facing: Direction,
}

impl Agent {
    pub fn new(id: String, name: String, model: String, session: SessionInfo, color_index: usize) -> Self {
        Self {
            id,
            name,
            status: AgentStatus::Spawning,
            model,
            task: None,
            current_tool: None,
            session,
            tokens: 0,
            cost: 0.0,
            started_at: std::time::Instant::now(),
            position: (0.0, 0.0),
            target_room: Room::Workspace,
            path: Vec::new(),
            assigned_desk: None,
            sprite_color: SpriteColor::from_index(color_index),
            facing: Direction::Right,
        }
    }

    pub fn target_room_for_status(status: AgentStatus) -> Room {
        match status {
            AgentStatus::Working => Room::Workspace,
            AgentStatus::Idle => Room::Lounge,
            AgentStatus::Spawning => Room::Workspace,
            AgentStatus::Finished => Room::Lounge,
            AgentStatus::Error => Room::Workspace,
        }
    }

    pub fn is_animating(&self) -> bool {
        !self.path.is_empty()
    }
}
```

- [ ] **Step 3: Create game/floor.rs with layout types**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Empty,
    Wall,
    Door,
    Desk,
    Monitor,
    PingPongTable,
    CeoDesk,
    CeoMonitor,
}

#[derive(Debug, Clone, Copy)]
pub struct DeskSlot {
    pub desk_x: u16,
    pub desk_y: u16,
    pub chair_x: u16,
    pub chair_y: u16,
    pub occupied: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct DoorPos {
    pub x: u16,
    pub y: u16,
    pub connects: [super::agent::Room; 2],
}

#[derive(Debug, Clone)]
pub struct Floor {
    pub width: u16,
    pub height: u16,
    pub grid: Vec<Vec<CellType>>,

    // Room boundaries (x, y, w, h) in grid coords
    pub workspace: (u16, u16, u16, u16),
    pub lounge: (u16, u16, u16, u16),
    pub ceo_office: (u16, u16, u16, u16),

    pub desks: Vec<DeskSlot>,
    pub doors: Vec<DoorPos>,
    pub ceo_chair: (u16, u16),
    pub ping_pong: (u16, u16, u16, u16), // x, y, w, h
}

impl Floor {
    pub fn generate(width: u16, height: u16) -> Self {
        let workspace_h = (height as f32 * 0.65) as u16;
        let bottom_h = height - workspace_h;
        let lounge_w = (width as f32 * 0.75) as u16;
        let ceo_w = width - lounge_w;

        let mut grid = vec![vec![CellType::Empty; width as usize]; height as usize];

        // Draw workspace walls (top border)
        for x in 0..width {
            grid[0][x as usize] = CellType::Wall;
        }
        // Left and right borders for workspace
        for y in 0..workspace_h {
            grid[y as usize][0] = CellType::Wall;
            grid[y as usize][(width - 1) as usize] = CellType::Wall;
        }

        // Divider between workspace and bottom row
        let div_y = workspace_h;
        for x in 0..width {
            grid[div_y as usize][x as usize] = CellType::Wall;
        }

        // Vertical divider between lounge and CEO
        for y in div_y..height {
            grid[y as usize][lounge_w as usize] = CellType::Wall;
        }

        // Bottom and side walls for bottom row
        for x in 0..width {
            grid[(height - 1) as usize][x as usize] = CellType::Wall;
        }
        for y in div_y..height {
            grid[y as usize][0] = CellType::Wall;
            grid[y as usize][(width - 1) as usize] = CellType::Wall;
        }

        // Place doors
        let mut doors = Vec::new();

        // Lounge door-left: near left side of divider
        let door_left_x = 2;
        grid[div_y as usize][door_left_x as usize] = CellType::Door;
        grid[div_y as usize][(door_left_x + 1) as usize] = CellType::Door;
        doors.push(DoorPos {
            x: door_left_x,
            y: div_y,
            connects: [super::agent::Room::Workspace, super::agent::Room::Lounge],
        });

        // Lounge door-right: near right side of lounge
        let door_right_x = lounge_w - 3;
        grid[div_y as usize][door_right_x as usize] = CellType::Door;
        grid[div_y as usize][(door_right_x + 1) as usize] = CellType::Door;
        doors.push(DoorPos {
            x: door_right_x,
            y: div_y,
            connects: [super::agent::Room::Workspace, super::agent::Room::Lounge],
        });

        // CEO door
        let ceo_door_x = lounge_w + (ceo_w / 2);
        grid[div_y as usize][ceo_door_x as usize] = CellType::Door;
        grid[div_y as usize][(ceo_door_x + 1) as usize] = CellType::Door;
        doors.push(DoorPos {
            x: ceo_door_x,
            y: div_y,
            connects: [super::agent::Room::Workspace, super::agent::Room::CeoOffice],
        });

        // Place desks in workspace (rows of desks with spacing)
        let mut desks = Vec::new();
        let desk_start_x = 3u16;
        let desk_start_y = 2u16;
        let desk_spacing_x = 6u16;
        let desk_spacing_y = 3u16;

        let desks_per_row = ((width - 6) / desk_spacing_x) as usize;
        let desk_rows = ((workspace_h - 4) / desk_spacing_y) as usize;

        for row in 0..desk_rows {
            for col in 0..desks_per_row {
                let dx = desk_start_x + (col as u16) * desk_spacing_x;
                let dy = desk_start_y + (row as u16) * desk_spacing_y;
                if dx + 2 < width - 1 && dy + 1 < div_y {
                    grid[dy as usize][dx as usize] = CellType::Desk;
                    grid[dy as usize][(dx + 1) as usize] = CellType::Monitor;
                    desks.push(DeskSlot {
                        desk_x: dx,
                        desk_y: dy,
                        chair_x: dx,
                        chair_y: dy + 1,
                        occupied: false,
                    });
                }
            }
        }

        // Place ping pong table in lounge center
        let pp_w = 6u16;
        let pp_h = 2u16;
        let pp_x = (lounge_w / 2).saturating_sub(pp_w / 2).max(2);
        let pp_y = div_y + (bottom_h / 2).saturating_sub(pp_h / 2);
        for dy in 0..pp_h {
            for dx in 0..pp_w {
                let gy = (pp_y + dy) as usize;
                let gx = (pp_x + dx) as usize;
                if gy < height as usize - 1 && gx < lounge_w as usize {
                    grid[gy][gx] = CellType::PingPongTable;
                }
            }
        }

        // CEO desk
        let ceo_desk_x = lounge_w + (ceo_w / 2) - 1;
        let ceo_desk_y = div_y + (bottom_h / 2);
        if (ceo_desk_x as usize) < width as usize - 1 && (ceo_desk_y as usize) < height as usize - 1 {
            grid[ceo_desk_y as usize][ceo_desk_x as usize] = CellType::CeoDesk;
            grid[ceo_desk_y as usize][(ceo_desk_x + 1) as usize] = CellType::CeoMonitor;
        }
        let ceo_chair = (ceo_desk_x, ceo_desk_y + 1);

        Floor {
            width,
            height,
            grid,
            workspace: (0, 0, width, workspace_h),
            lounge: (0, div_y, lounge_w, bottom_h),
            ceo_office: (lounge_w, div_y, ceo_w, bottom_h),
            desks,
            doors,
            ceo_chair,
            ping_pong: (pp_x, pp_y, pp_w, pp_h),
        }
    }

    pub fn room_center(&self, room: super::agent::Room) -> (u16, u16) {
        let (rx, ry, rw, rh) = match room {
            super::agent::Room::Workspace => self.workspace,
            super::agent::Room::Lounge => self.lounge,
            super::agent::Room::CeoOffice => self.ceo_office,
        };
        (rx + rw / 2, ry + rh / 2)
    }

    pub fn assign_desk(&mut self) -> Option<usize> {
        for (i, desk) in self.desks.iter_mut().enumerate() {
            if !desk.occupied {
                desk.occupied = true;
                return Some(i);
            }
        }
        // All desks full — share the first desk
        if !self.desks.is_empty() {
            Some(0)
        } else {
            None
        }
    }

    pub fn free_desk(&mut self, index: usize) {
        if let Some(desk) = self.desks.get_mut(index) {
            desk.occupied = false;
        }
    }

    pub fn nearest_door(&self, x: u16, y: u16, from_room: super::agent::Room, to_room: super::agent::Room) -> Option<&DoorPos> {
        self.doors
            .iter()
            .filter(|d| d.connects.contains(&from_room) && d.connects.contains(&to_room))
            .min_by_key(|d| {
                let dx = (d.x as i32 - x as i32).unsigned_abs();
                let dy = (d.y as i32 - y as i32).unsigned_abs();
                dx + dy
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_floor_dimensions() {
        let floor = Floor::generate(80, 30);
        assert_eq!(floor.width, 80);
        assert_eq!(floor.height, 30);
        assert_eq!(floor.grid.len(), 30);
        assert_eq!(floor.grid[0].len(), 80);
    }

    #[test]
    fn test_floor_has_three_doors() {
        let floor = Floor::generate(80, 30);
        assert_eq!(floor.doors.len(), 3);
    }

    #[test]
    fn test_floor_has_desks() {
        let floor = Floor::generate(80, 30);
        assert!(!floor.desks.is_empty());
    }

    #[test]
    fn test_desk_assignment() {
        let mut floor = Floor::generate(80, 30);
        let first = floor.assign_desk();
        assert!(first.is_some());
        assert_eq!(first.unwrap(), 0);

        let second = floor.assign_desk();
        assert!(second.is_some());
        assert_eq!(second.unwrap(), 1);
    }

    #[test]
    fn test_desk_free_and_reassign() {
        let mut floor = Floor::generate(80, 30);
        let idx = floor.assign_desk().unwrap();
        floor.free_desk(idx);
        let reassigned = floor.assign_desk().unwrap();
        assert_eq!(reassigned, idx);
    }

    #[test]
    fn test_workspace_proportions() {
        let floor = Floor::generate(80, 30);
        let (_, _, _, wh) = floor.workspace;
        // 65% of 30 = 19
        assert_eq!(wh, 19);
    }

    #[test]
    fn test_lounge_width_proportion() {
        let floor = Floor::generate(80, 30);
        let (_, _, lw, _) = floor.lounge;
        // 75% of 80 = 60
        assert_eq!(lw, 60);
    }
}
```

- [ ] **Step 4: Create game/state.rs**

```rust
use super::agent::{Agent, AgentStatus, Room};
use super::floor::Floor;

#[derive(Debug)]
pub struct Stats {
    pub model: String,
    pub active_agents: usize,
    pub total_agents: usize,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub usage_percent: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            model: String::from("unknown"),
            active_agents: 0,
            total_agents: 0,
            completed_tasks: 0,
            total_tasks: 0,
            total_tokens: 0,
            total_cost: 0.0,
            usage_percent: 0.0,
        }
    }
}

pub struct GameState {
    pub floor: Floor,
    pub agents: Vec<Agent>,
    pub stats: Stats,
    pub ceo_status: CeoStatus,
    pub next_color_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeoStatus {
    Idle,
    PromptSent,
    Waiting,
    AllComplete,
    Error,
}

impl GameState {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            floor: Floor::generate(width, height),
            agents: Vec::new(),
            stats: Stats::default(),
            ceo_status: CeoStatus::Idle,
            next_color_index: 0,
        }
    }

    pub fn has_animations(&self) -> bool {
        self.agents.iter().any(|a| a.is_animating())
    }

    pub fn update_stats(&mut self) {
        self.stats.active_agents = self
            .agents
            .iter()
            .filter(|a| a.status == AgentStatus::Working)
            .count();
        self.stats.total_agents = self.agents.len();
        self.stats.total_tokens = self.agents.iter().map(|a| a.tokens).sum();
        self.stats.total_cost = self.agents.iter().map(|a| a.cost).sum();
    }
}
```

- [ ] **Step 5: Add mod game to main.rs**

Add `mod game;` at the top of `src/main.rs`.

- [ ] **Step 6: Build and run tests**

Run: `cargo test`
Expected: All floor tests pass.

Run: `cargo build`
Expected: Compiles clean.

- [ ] **Step 7: Commit**

```bash
git add src/game/
git commit -m "feat: add core game types — agent, floor layout, game state"
```

---

### Task 3: Pathfinding

**Files:**
- Create: `src/game/pathfinding.rs`
- Modify: `src/game/mod.rs` (add `pub mod pathfinding;`)

- [ ] **Step 1: Create pathfinding.rs with tests first**

```rust
use super::agent::Room;
use super::floor::Floor;

/// Compute a waypoint path from current position to a target position in a target room.
/// Returns a list of (x, y) waypoints the agent should walk through.
pub fn compute_path(
    from_x: u16,
    from_y: u16,
    from_room: Room,
    to_room: Room,
    to_x: u16,
    to_y: u16,
    floor: &Floor,
) -> Vec<(u16, u16)> {
    if from_room == to_room {
        // Same room — walk directly to target
        return vec![(to_x, to_y)];
    }

    let mut path = Vec::new();

    match (from_room, to_room) {
        (Room::Workspace, Room::Lounge) | (Room::Lounge, Room::Workspace) => {
            if let Some(door) = floor.nearest_door(from_x, from_y, from_room, to_room) {
                // Walk to door from current side
                let door_enter_y = if from_room == Room::Workspace {
                    door.y.saturating_sub(1)
                } else {
                    door.y + 1
                };
                path.push((door.x, door_enter_y));
                // Walk through door
                let door_exit_y = if from_room == Room::Workspace {
                    door.y + 1
                } else {
                    door.y.saturating_sub(1)
                };
                path.push((door.x, door_exit_y));
                // Walk to target
                path.push((to_x, to_y));
            }
        }
        (Room::Workspace, Room::CeoOffice) | (Room::CeoOffice, Room::Workspace) => {
            if let Some(door) = floor.nearest_door(from_x, from_y, from_room, to_room) {
                let door_enter_y = if from_room == Room::Workspace {
                    door.y.saturating_sub(1)
                } else {
                    door.y + 1
                };
                path.push((door.x, door_enter_y));
                let door_exit_y = if from_room == Room::Workspace {
                    door.y + 1
                } else {
                    door.y.saturating_sub(1)
                };
                path.push((door.x, door_exit_y));
                path.push((to_x, to_y));
            }
        }
        (Room::Lounge, Room::CeoOffice) | (Room::CeoOffice, Room::Lounge) => {
            // Must go through Workspace — no direct door
            let workspace_center = floor.room_center(Room::Workspace);

            // Step 1: exit current room into workspace
            let exit_room = from_room;
            let enter_workspace_room = Room::Workspace;
            if let Some(door_out) = floor.nearest_door(from_x, from_y, exit_room, enter_workspace_room) {
                let door_enter_y = if from_room == Room::Lounge || from_room == Room::CeoOffice {
                    door_out.y + 1
                } else {
                    door_out.y.saturating_sub(1)
                };
                path.push((door_out.x, door_enter_y));
                // Through door into workspace
                path.push((door_out.x, door_out.y.saturating_sub(1)));
            }

            // Step 2: walk through workspace to the other door
            path.push((workspace_center.0, workspace_center.1));

            // Step 3: exit workspace into target room
            if let Some(door_in) = floor.nearest_door(workspace_center.0, workspace_center.1, Room::Workspace, to_room) {
                path.push((door_in.x, door_in.y.saturating_sub(1)));
                path.push((door_in.x, door_in.y + 1));
            }

            path.push((to_x, to_y));
        }
        _ => {
            // Same room case already handled above
            path.push((to_x, to_y));
        }
    }

    path
}

/// Advance an agent's position toward the next waypoint.
/// Returns true if the agent moved (animation still active).
pub fn advance_along_path(
    position: &mut (f32, f32),
    path: &mut Vec<(u16, u16)>,
    speed: f32,
    delta_secs: f32,
) -> bool {
    if path.is_empty() {
        return false;
    }

    let target = path[0];
    let tx = target.0 as f32;
    let ty = target.1 as f32;

    let dx = tx - position.0;
    let dy = ty - position.1;
    let dist = (dx * dx + dy * dy).sqrt();

    let step = speed * delta_secs;

    if dist <= step {
        // Arrived at waypoint
        position.0 = tx;
        position.1 = ty;
        path.remove(0);
        // Recurse with remaining time if there's more path
        if !path.is_empty() {
            let remaining = (step - dist) / speed;
            if remaining > 0.001 {
                return advance_along_path(position, path, speed, remaining);
            }
        }
        !path.is_empty()
    } else {
        // Move toward waypoint
        let ratio = step / dist;
        position.0 += dx * ratio;
        position.1 += dy * ratio;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_floor() -> Floor {
        Floor::generate(80, 30)
    }

    #[test]
    fn test_same_room_direct_path() {
        let floor = test_floor();
        let path = compute_path(5, 5, Room::Workspace, Room::Workspace, 20, 10, &floor);
        assert_eq!(path.len(), 1);
        assert_eq!(path[0], (20, 10));
    }

    #[test]
    fn test_workspace_to_lounge_goes_through_door() {
        let floor = test_floor();
        let path = compute_path(10, 5, Room::Workspace, Room::Lounge, 10, 25, &floor);
        // Should have at least 3 waypoints: approach door, through door, destination
        assert!(path.len() >= 3);
    }

    #[test]
    fn test_lounge_to_ceo_goes_through_workspace() {
        let floor = test_floor();
        let path = compute_path(10, 25, Room::Lounge, Room::CeoOffice, 70, 25, &floor);
        // Should be longer — goes up through workspace then down
        assert!(path.len() >= 5);
    }

    #[test]
    fn test_advance_reaches_waypoint() {
        let mut pos = (0.0, 0.0);
        let mut path = vec![(10, 0)];
        let moved = advance_along_path(&mut pos, &mut path, 4.0, 3.0);
        // 4 tiles/sec * 3 sec = 12 tiles, target is 10 away — should arrive
        assert!(!moved);
        assert_eq!(pos.0, 10.0);
        assert!(path.is_empty());
    }

    #[test]
    fn test_advance_partial_movement() {
        let mut pos = (0.0, 0.0);
        let mut path = vec![(10, 0)];
        let moved = advance_along_path(&mut pos, &mut path, 4.0, 1.0);
        // 4 tiles/sec * 1 sec = 4 tiles
        assert!(moved);
        assert!((pos.0 - 4.0).abs() < 0.01);
        assert_eq!(path.len(), 1);
    }

    #[test]
    fn test_advance_empty_path() {
        let mut pos = (5.0, 5.0);
        let mut path: Vec<(u16, u16)> = vec![];
        let moved = advance_along_path(&mut pos, &mut path, 4.0, 1.0);
        assert!(!moved);
        assert_eq!(pos, (5.0, 5.0));
    }

    #[test]
    fn test_advance_chain_waypoints() {
        let mut pos = (0.0, 0.0);
        let mut path = vec![(2, 0), (2, 3)];
        // Speed 10, delta 1 = 10 tiles. First waypoint at dist 2, second at dist 3 more = 5 total
        let moved = advance_along_path(&mut pos, &mut path, 10.0, 1.0);
        assert!(!moved);
        assert!((pos.0 - 2.0).abs() < 0.01);
        assert!((pos.1 - 3.0).abs() < 0.01);
    }
}
```

- [ ] **Step 2: Add pathfinding to game/mod.rs**

```rust
pub mod agent;
pub mod floor;
pub mod pathfinding;
pub mod state;
```

- [ ] **Step 3: Run tests**

Run: `cargo test game::pathfinding`
Expected: All 6 pathfinding tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/game/pathfinding.rs src/game/mod.rs
git commit -m "feat: add waypoint pathfinding between rooms"
```

---

### Task 4: Sprite Definitions

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/sprites.rs`
- Modify: `src/main.rs` (add `mod ui;`)

- [ ] **Step 1: Create ui/mod.rs**

```rust
pub mod sprites;
```

- [ ] **Step 2: Create ui/sprites.rs**

```rust
use ratatui::style::Color;

/// Agent sprite — 2 cells wide × 2 cells tall using braille characters.
/// Top row: head, Bottom row: body.
pub struct Sprite {
    pub top: [&'static str; 2],    // [left_char, right_char]
    pub bottom: [&'static str; 2], // [left_char, right_char]
}

/// Agent facing right
pub const AGENT_RIGHT: Sprite = Sprite {
    top: ["⣿", "⣷"],
    bottom: ["⣿", "⣿"],
};

/// Agent facing left
pub const AGENT_LEFT: Sprite = Sprite {
    top: ["⣾", "⣿"],
    bottom: ["⣿", "⣿"],
};

/// CEO sprite — same dimensions, distinguished by color (gold)
pub const CEO_SPRITE: Sprite = Sprite {
    top: ["⣿", "⣷"],
    bottom: ["⣿", "⣿"],
};

pub const CEO_COLOR: Color = Color::Rgb(255, 215, 0); // Gold

/// Desk: 2 cells wide
pub const DESK: [&str; 2] = ["▄▄", "▄▄"];

/// Monitor on desk: 2 cells wide × 2 tall
pub const MONITOR: MonitorSprite = MonitorSprite {
    top: ["█▓", "▓█"],
    bottom: ["▀▀", "▀▀"],
};

pub struct MonitorSprite {
    pub top: [&'static str; 2],
    pub bottom: [&'static str; 2],
}

/// Monitor flicker variant (for working animation)
pub const MONITOR_FLICKER: MonitorSprite = MonitorSprite {
    top: ["█▒", "▒█"],
    bottom: ["▀▀", "▀▀"],
};

/// Ping pong table: 6 wide × 2 tall
pub const PING_PONG_TABLE: [[&str; 6]; 2] = [
    ["▄", "▄", "▄", "▄", "▄", "▄"],
    ["█", "▒", "▒", "▒", "▒", "█"],
];

pub const PING_PONG_COLOR: Color = Color::Rgb(0, 128, 0); // Dark green

/// Room wall colors
pub const WALL_COLOR: Color = Color::Rgb(100, 100, 120);
pub const DOOR_COLOR: Color = Color::Rgb(139, 90, 43); // Brown

/// Room background tints
pub const WORKSPACE_BG: Color = Color::Rgb(15, 15, 25);
pub const LOUNGE_BG: Color = Color::Rgb(20, 15, 15);
pub const CEO_BG: Color = Color::Rgb(25, 20, 10);

/// Monitor color
pub const MONITOR_COLOR: Color = Color::Rgb(60, 180, 255);
pub const MONITOR_FLICKER_COLOR: Color = Color::Rgb(40, 140, 200);

/// Desk color
pub const DESK_COLOR: Color = Color::Rgb(139, 90, 43);
```

- [ ] **Step 3: Add mod ui to main.rs**

Add `mod ui;` to `src/main.rs`.

- [ ] **Step 4: Build**

Run: `cargo build`
Expected: Compiles clean.

- [ ] **Step 5: Commit**

```bash
git add src/ui/
git commit -m "feat: add sprite and color definitions for agents, furniture, rooms"
```

---

### Task 5: Floor View Renderer

**Files:**
- Create: `src/ui/floor_view.rs`
- Modify: `src/ui/mod.rs` (add `pub mod floor_view;`)

- [ ] **Step 1: Create floor_view.rs**

```rust
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::game::agent::{Agent, AgentStatus, Direction, Room};
use crate::game::floor::{CellType, Floor};
use crate::game::state::{CeoStatus, GameState};
use super::sprites;

pub struct FloorView<'a> {
    pub state: &'a GameState,
    pub highlighted_room: Option<Room>,
    pub tick: u64,
}

impl<'a> Widget for FloorView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let floor = &self.state.floor;

        // Scale grid to fit area
        let scale_x = area.width.min(floor.width);
        let scale_y = area.height.min(floor.height);

        // Render grid cells
        for gy in 0..scale_y {
            for gx in 0..scale_x {
                let cell_type = floor.grid.get(gy as usize).and_then(|row| row.get(gx as usize)).copied().unwrap_or(CellType::Empty);
                let screen_x = area.x + gx;
                let screen_y = area.y + gy;

                if screen_x >= area.right() || screen_y >= area.bottom() {
                    continue;
                }

                let (ch, fg, bg) = match cell_type {
                    CellType::Wall => ('│', sprites::WALL_COLOR, Color::Reset),
                    CellType::Door => ('▒', sprites::DOOR_COLOR, Color::Reset),
                    CellType::Desk => ('▄', sprites::DESK_COLOR, Color::Reset),
                    CellType::Monitor => ('▓', sprites::MONITOR_COLOR, self.room_bg_at(floor, gx, gy)),
                    CellType::CeoDesk => ('▄', Color::Rgb(180, 120, 60), sprites::CEO_BG),
                    CellType::CeoMonitor => {
                        let color = if self.tick % 60 < 30 {
                            sprites::MONITOR_COLOR
                        } else {
                            sprites::MONITOR_FLICKER_COLOR
                        };
                        ('▓', color, sprites::CEO_BG)
                    }
                    CellType::PingPongTable => ('▒', sprites::PING_PONG_COLOR, self.room_bg_at(floor, gx, gy)),
                    CellType::Empty => (' ', Color::Reset, self.room_bg_at(floor, gx, gy)),
                };

                buf[(screen_x, screen_y)]
                    .set_char(ch)
                    .set_fg(fg)
                    .set_bg(bg);
            }
        }

        // Render top wall with room labels
        self.render_room_labels(area, buf, floor);

        // Render horizontal walls properly
        self.render_walls(area, buf, floor, scale_x, scale_y);

        // Render agents
        for agent in &self.state.agents {
            self.render_agent(area, buf, agent);
        }

        // Render CEO
        self.render_ceo(area, buf, floor);

        // Render room highlight if focused
        if let Some(room) = self.highlighted_room {
            self.render_room_highlight(area, buf, floor, room);
        }
    }
}

impl<'a> FloorView<'a> {
    fn room_bg_at(&self, floor: &Floor, x: u16, y: u16) -> Color {
        let (_, _, _, wh) = floor.workspace;
        let (lx, _, lw, _) = floor.lounge;

        if y < wh {
            sprites::WORKSPACE_BG
        } else if x >= lx && x < lx + lw {
            sprites::LOUNGE_BG
        } else {
            sprites::CEO_BG
        }
    }

    fn render_room_labels(&self, area: Rect, buf: &mut Buffer, floor: &Floor) {
        let (_, _, ww, _) = floor.workspace;
        let label_style = Style::default().fg(Color::White).bg(sprites::WALL_COLOR);

        // Workspace label
        let label = " Workspace ";
        let lx = area.x + (ww / 2).saturating_sub(label.len() as u16 / 2);
        buf.set_string(lx, area.y, label, label_style);

        // Lounge label
        let (_, ly, lw, _) = floor.lounge;
        let label = " Lounge ";
        let lx = area.x + (lw / 2).saturating_sub(label.len() as u16 / 2);
        buf.set_string(lx, area.y + ly, label, label_style);

        // CEO Office label
        let (cx, cy, cw, _) = floor.ceo_office;
        let label = " CEO ";
        let lx = area.x + cx + (cw / 2).saturating_sub(label.len() as u16 / 2);
        buf.set_string(lx, area.y + cy, label, label_style);
    }

    fn render_walls(&self, area: Rect, buf: &mut Buffer, floor: &Floor, width: u16, height: u16) {
        // Top border
        for x in 0..width {
            let sx = area.x + x;
            if sx < area.right() {
                buf[(sx, area.y)].set_char('─').set_fg(sprites::WALL_COLOR);
            }
        }
        // Bottom border
        let bot_y = area.y + height.saturating_sub(1);
        if bot_y < area.bottom() {
            for x in 0..width {
                let sx = area.x + x;
                if sx < area.right() {
                    buf[(sx, bot_y)].set_char('─').set_fg(sprites::WALL_COLOR);
                }
            }
        }
        // Corners
        if area.y < area.bottom() {
            buf[(area.x, area.y)].set_char('┌').set_fg(sprites::WALL_COLOR);
            if area.x + width - 1 < area.right() {
                buf[(area.x + width - 1, area.y)].set_char('┐').set_fg(sprites::WALL_COLOR);
            }
        }
        if bot_y < area.bottom() {
            buf[(area.x, bot_y)].set_char('└').set_fg(sprites::WALL_COLOR);
            if area.x + width - 1 < area.right() {
                buf[(area.x + width - 1, bot_y)].set_char('┘').set_fg(sprites::WALL_COLOR);
            }
        }
    }

    fn render_agent(&self, area: Rect, buf: &mut Buffer, agent: &Agent) {
        let ax = area.x + agent.position.0 as u16;
        let ay = area.y + agent.position.1 as u16;

        if ax + 1 >= area.right() || ay + 1 >= area.bottom() {
            return;
        }

        let color = if agent.status == AgentStatus::Error {
            Color::Red
        } else {
            agent.sprite_color.to_color()
        };

        let sprite = match agent.facing {
            Direction::Right => &sprites::AGENT_RIGHT,
            Direction::Left => &sprites::AGENT_LEFT,
        };

        let style = Style::default().fg(color);

        buf.set_string(ax, ay, sprite.top[0], style);
        buf.set_string(ax + 1, ay, sprite.top[1], style);
        buf.set_string(ax, ay + 1, sprite.bottom[0], style);
        buf.set_string(ax + 1, ay + 1, sprite.bottom[1], style);
    }

    fn render_ceo(&self, area: Rect, buf: &mut Buffer, floor: &Floor) {
        let (cx, cy) = floor.ceo_chair;
        let ax = area.x + cx;
        let ay = area.y + cy;

        if ax + 1 >= area.right() || ay + 1 >= area.bottom() {
            return;
        }

        let style = Style::default().fg(sprites::CEO_COLOR);
        buf.set_string(ax, ay, sprites::CEO_SPRITE.top[0], style);
        buf.set_string(ax + 1, ay, sprites::CEO_SPRITE.top[1], style);
        buf.set_string(ax, ay + 1, sprites::CEO_SPRITE.bottom[0], style);
        buf.set_string(ax + 1, ay + 1, sprites::CEO_SPRITE.bottom[1], style);
    }

    fn render_room_highlight(&self, area: Rect, buf: &mut Buffer, floor: &Floor, room: Room) {
        let (rx, ry, rw, rh) = match room {
            Room::Workspace => floor.workspace,
            Room::Lounge => floor.lounge,
            Room::CeoOffice => floor.ceo_office,
        };
        let highlight = Style::default().fg(Color::Rgb(255, 255, 100));

        // Top edge
        for x in rx..rx + rw {
            let sx = area.x + x;
            let sy = area.y + ry;
            if sx < area.right() && sy < area.bottom() {
                buf[(sx, sy)].set_style(highlight);
            }
        }
        // Bottom edge
        let by = ry + rh - 1;
        for x in rx..rx + rw {
            let sx = area.x + x;
            let sy = area.y + by;
            if sx < area.right() && sy < area.bottom() {
                buf[(sx, sy)].set_style(highlight);
            }
        }
    }
}
```

- [ ] **Step 2: Update ui/mod.rs**

```rust
pub mod floor_view;
pub mod sprites;
```

- [ ] **Step 3: Build**

Run: `cargo build`
Expected: Compiles clean.

- [ ] **Step 4: Commit**

```bash
git add src/ui/floor_view.rs src/ui/mod.rs
git commit -m "feat: add floor view renderer with rooms, agents, furniture"
```

---

### Task 6: Stats Bar & Agent Panel Renderers

**Files:**
- Create: `src/ui/stats_bar.rs`
- Create: `src/ui/agent_panel.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create stats_bar.rs**

```rust
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::game::state::Stats;

pub struct StatsBar<'a> {
    pub stats: &'a Stats,
}

impl<'a> Widget for StatsBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let sep = Span::styled(" │ ", Style::default().fg(Color::DarkGray));
        let label_style = Style::default().fg(Color::DarkGray);

        let spans = vec![
            Span::styled(" Model: ", label_style),
            Span::styled(&self.stats.model, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            sep.clone(),
            Span::styled("Agents: ", label_style),
            Span::styled(
                format!("{}/{}", self.stats.active_agents, self.stats.total_agents),
                Style::default().fg(if self.stats.active_agents == self.stats.total_agents {
                    Color::Green
                } else {
                    Color::Yellow
                }),
            ),
            sep.clone(),
            Span::styled("Tasks: ", label_style),
            Span::styled(
                format!("{}/{}", self.stats.completed_tasks, self.stats.total_tasks),
                Style::default().fg(Color::Cyan),
            ),
            sep.clone(),
            Span::styled("Tokens: ", label_style),
            Span::styled(format_tokens(self.stats.total_tokens), Style::default().fg(Color::White)),
            sep.clone(),
            Span::styled("Cost: ", label_style),
            Span::styled(
                format!("${:.2}", self.stats.total_cost),
                Style::default().fg(cost_color(self.stats.total_cost)),
            ),
            sep,
            Span::styled("Usage: ", label_style),
            Span::styled(
                format!("{:.0}%", self.stats.usage_percent),
                Style::default().fg(usage_color(self.stats.usage_percent)),
            ),
        ];

        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}k", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

fn cost_color(cost: f64) -> Color {
    if cost < 1.0 {
        Color::Green
    } else if cost < 5.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn usage_color(pct: f32) -> Color {
    if pct < 50.0 {
        Color::Green
    } else if pct < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tokens_small() {
        assert_eq!(format_tokens(500), "500");
    }

    #[test]
    fn test_format_tokens_thousands() {
        assert_eq!(format_tokens(14200), "14.2k");
    }

    #[test]
    fn test_format_tokens_millions() {
        assert_eq!(format_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn test_cost_color_green() {
        assert_eq!(cost_color(0.42), Color::Green);
    }

    #[test]
    fn test_cost_color_yellow() {
        assert_eq!(cost_color(3.50), Color::Yellow);
    }

    #[test]
    fn test_cost_color_red() {
        assert_eq!(cost_color(10.0), Color::Red);
    }

    #[test]
    fn test_usage_color_green() {
        assert_eq!(usage_color(30.0), Color::Green);
    }

    #[test]
    fn test_usage_color_yellow() {
        assert_eq!(usage_color(65.0), Color::Yellow);
    }

    #[test]
    fn test_usage_color_red() {
        assert_eq!(usage_color(90.0), Color::Red);
    }
}
```

- [ ] **Step 2: Create agent_panel.rs**

```rust
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, StatefulWidget, Widget},
};

use crate::game::agent::{Agent, AgentStatus};

pub struct AgentPanelState {
    pub selected: Option<usize>,
    pub expanded: Option<usize>,
    pub scroll_offset: usize,
}

impl AgentPanelState {
    pub fn new() -> Self {
        Self {
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
            Some(i) => (i + 1) % agent_count,
            None => 0,
        });
    }

    pub fn select_prev(&mut self, agent_count: usize) {
        if agent_count == 0 {
            self.selected = None;
            return;
        }
        self.selected = Some(match self.selected {
            Some(0) => agent_count - 1,
            Some(i) => i - 1,
            None => agent_count - 1,
        });
    }

    pub fn toggle_expand(&mut self) {
        if self.expanded == self.selected {
            self.expanded = None;
        } else {
            self.expanded = self.selected;
        }
    }
}

pub struct AgentPanel<'a> {
    pub agents: &'a [Agent],
}

impl<'a> StatefulWidget for AgentPanel<'a> {
    type State = AgentPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default()
            .title(" Agents ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.agents.is_empty() {
            let style = Style::default().fg(Color::DarkGray);
            buf.set_string(inner.x + 1, inner.y, "No agents connected", style);
            return;
        }

        let mut y = inner.y;
        for (i, agent) in self.agents.iter().enumerate().skip(state.scroll_offset) {
            if y >= inner.bottom() {
                break;
            }

            let is_selected = state.selected == Some(i);
            let is_expanded = state.expanded == Some(i);

            // Collapsed line
            let line = self.render_collapsed_line(agent, is_selected, is_expanded);
            buf.set_line(inner.x, y, &line, inner.width);
            y += 1;

            // Expanded details
            if is_expanded && y + 2 < inner.bottom() {
                let detail1 = Line::from(vec![
                    Span::raw("   Model: "),
                    Span::styled(&agent.model, Style::default().fg(Color::White)),
                    Span::raw("  Tokens: "),
                    Span::styled(format!("{}", agent.tokens), Style::default().fg(Color::White)),
                    Span::raw("  Cost: "),
                    Span::styled(format!("${:.2}", agent.cost), Style::default().fg(Color::Green)),
                ]);
                buf.set_line(inner.x, y, &detail1, inner.width);
                y += 1;

                if let Some(ref tool) = agent.current_tool {
                    let detail2 = Line::from(vec![
                        Span::raw("   Current tool: "),
                        Span::styled(tool, Style::default().fg(Color::Cyan)),
                    ]);
                    buf.set_line(inner.x, y, &detail2, inner.width);
                    y += 1;
                }

                let elapsed = agent.started_at.elapsed();
                let mins = elapsed.as_secs() / 60;
                let secs = elapsed.as_secs() % 60;
                let detail3 = Line::from(vec![
                    Span::raw("   Duration: "),
                    Span::styled(format!("{}m {}s", mins, secs), Style::default().fg(Color::DarkGray)),
                ]);
                buf.set_line(inner.x, y, &detail3, inner.width);
                y += 1;
            }
        }
    }
}

impl<'a> AgentPanel<'a> {
    fn render_collapsed_line(&self, agent: &Agent, selected: bool, expanded: bool) -> Line<'static> {
        let (indicator, indicator_color) = match agent.status {
            AgentStatus::Working => ("●", Color::Green),
            AgentStatus::Error => ("●", Color::Red),
            AgentStatus::Idle => ("○", Color::DarkGray),
            AgentStatus::Finished => ("◌", Color::DarkGray),
            AgentStatus::Spawning => ("◐", Color::Yellow),
        };

        let status_str = match agent.status {
            AgentStatus::Working => "WORKING",
            AgentStatus::Error => "ERROR",
            AgentStatus::Idle => "IDLE",
            AgentStatus::Finished => "DONE",
            AgentStatus::Spawning => "SPAWNING",
        };

        let arrow = if expanded { "▼" } else if selected { "▶" } else { " " };

        let name_style = if selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let task_str = agent.task.clone().unwrap_or_default();
        let session_str = format!(
            "{} @ {}{}",
            agent.session.branch,
            agent.session.repo,
            agent.session.worktree.as_ref().map(|w| format!(" ({})", w)).unwrap_or_default()
        );

        Line::from(vec![
            Span::styled(format!(" {} ", arrow), Style::default().fg(Color::Cyan)),
            Span::styled(format!("{} ", indicator), Style::default().fg(indicator_color)),
            Span::styled(format!("{:<12}", agent.name), name_style),
            Span::styled(format!("{:<10}", status_str), Style::default().fg(indicator_color)),
            Span::styled(
                if task_str.is_empty() { String::new() } else { format!("\"{}\"  ", task_str) },
                Style::default().fg(Color::White),
            ),
            Span::styled(session_str, Style::default().fg(Color::DarkGray)),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_next_wraps() {
        let mut state = AgentPanelState::new();
        state.selected = Some(2);
        state.select_next(3);
        assert_eq!(state.selected, Some(0));
    }

    #[test]
    fn test_select_prev_wraps() {
        let mut state = AgentPanelState::new();
        state.selected = Some(0);
        state.select_prev(3);
        assert_eq!(state.selected, Some(2));
    }

    #[test]
    fn test_select_next_from_none() {
        let mut state = AgentPanelState::new();
        state.select_next(3);
        assert_eq!(state.selected, Some(0));
    }

    #[test]
    fn test_toggle_expand() {
        let mut state = AgentPanelState::new();
        state.selected = Some(1);
        state.toggle_expand();
        assert_eq!(state.expanded, Some(1));
        state.toggle_expand();
        assert_eq!(state.expanded, None);
    }

    #[test]
    fn test_empty_agents() {
        let mut state = AgentPanelState::new();
        state.select_next(0);
        assert_eq!(state.selected, None);
    }
}
```

- [ ] **Step 3: Update ui/mod.rs**

```rust
pub mod agent_panel;
pub mod floor_view;
pub mod sprites;
pub mod stats_bar;
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: All tests pass (floor, pathfinding, stats_bar, agent_panel).

- [ ] **Step 5: Commit**

```bash
git add src/ui/stats_bar.rs src/ui/agent_panel.rs src/ui/mod.rs
git commit -m "feat: add stats bar and agent panel renderers"
```

---

### Task 7: Text Bubbles

**Files:**
- Create: `src/ui/bubbles.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create bubbles.rs**

```rust
use rand::seq::SliceRandom;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
};
use std::time::{Duration, Instant};

use crate::game::agent::AgentStatus;

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

    /// Trigger a bubble for an agent based on status change
    pub fn trigger_status_change(&mut self, agent_id: &str, new_status: AgentStatus) {
        let pool = match new_status {
            AgentStatus::Working => &["on it!", "let's go", "diving in...", "focus time"][..],
            AgentStatus::Idle => &["break time", "brb", "coffee...", "stretching"][..],
            AgentStatus::Finished => &["shipped!", "done!", "nailed it", "ez"][..],
            AgentStatus::Error => &["ugh...", "hmm...", "not good", "help?"][..],
            AgentStatus::Spawning => &["reporting in", "hello!", "ready"][..],
        };
        self.add_bubble(agent_id, pool);
    }

    /// Trigger a bubble for a tool use event
    pub fn trigger_tool_use(&mut self, agent_id: &str, tool: &str, args_hint: Option<&str>) {
        let text = match tool {
            "Read" => {
                if let Some(file) = args_hint {
                    format!("reading {}", short_filename(file))
                } else {
                    "reading...".to_string()
                }
            }
            "Edit" | "Write" => {
                if let Some(file) = args_hint {
                    format!("editing {}", short_filename(file))
                } else {
                    "writing...".to_string()
                }
            }
            "Bash" => {
                if let Some(cmd) = args_hint {
                    if cmd.contains("test") || cmd.contains("cargo test") {
                        "fingers crossed".to_string()
                    } else {
                        "running cmd...".to_string()
                    }
                } else {
                    "running cmd...".to_string()
                }
            }
            "Grep" | "Glob" => "searching...".to_string(),
            "Agent" => "calling backup".to_string(),
            _ => return, // Don't bubble for unknown tools
        };

        self.add_bubble_text(agent_id, &text);
    }

    /// Trigger a bubble when agent enters lounge
    pub fn trigger_lounge_arrival(&mut self, agent_id: &str) {
        let pool = &["ping pong?", "nice", "chill", "ahh"][..];
        self.add_bubble(agent_id, pool);
    }

    /// Trigger a bubble for long-running tasks
    pub fn trigger_long_task(&mut self, agent_id: &str) {
        let pool = &["still going...", "almost...", "deep in it", "hang on"][..];
        self.add_bubble(agent_id, pool);
    }

    fn add_bubble(&mut self, agent_id: &str, pool: &[&str]) {
        let mut rng = rand::thread_rng();
        if let Some(text) = pool.choose(&mut rng) {
            self.add_bubble_text(agent_id, text);
        }
    }

    fn add_bubble_text(&mut self, agent_id: &str, text: &str) {
        // Remove existing bubble for this agent
        self.bubbles.retain(|b| b.agent_id != agent_id);

        // Random lifetime between 3-5 seconds
        let lifetime_ms = 3000 + rand::random::<u64>() % 2000;

        self.bubbles.push(Bubble {
            agent_id: agent_id.to_string(),
            text: text.to_string(),
            created_at: Instant::now(),
            lifetime: Duration::from_millis(lifetime_ms),
        });

        // Enforce max visible — remove oldest if over limit
        while self.bubbles.len() > self.max_visible {
            self.bubbles.remove(0);
        }
    }

    /// Remove expired bubbles. Call each tick.
    pub fn tick(&mut self) {
        self.bubbles.retain(|b| !b.is_expired());
    }

    /// Render a bubble for a specific agent at a given screen position.
    pub fn render_bubble_at(&self, agent_id: &str, agent_x: u16, agent_y: u16, area: Rect, buf: &mut Buffer) {
        let bubble = match self.bubbles.iter().find(|b| b.agent_id == agent_id) {
            Some(b) => b,
            None => return,
        };

        let text_width = bubble.text.len() as u16;
        let box_width = text_width + 2; // padding
        let box_height = 3u16; // top border + text + bottom border with pointer

        // Position bubble above the agent
        let bx = agent_x.saturating_sub(box_width / 2);
        let by = agent_y.saturating_sub(box_height);

        // Clamp to area
        let bx = bx.max(area.x).min(area.right().saturating_sub(box_width));

        if by < area.y || bx + box_width > area.right() {
            return;
        }

        let style = Style::default().fg(Color::White);
        let border_style = Style::default().fg(Color::DarkGray);

        // Top border: ┌───┐
        let top = format!("┌{}┐", "─".repeat(text_width as usize));
        buf.set_string(bx, by, &top, border_style);

        // Text line: │text│
        buf.set_string(bx, by + 1, "│", border_style);
        buf.set_string(bx + 1, by + 1, &bubble.text, style);
        buf.set_string(bx + 1 + text_width, by + 1, "│", border_style);

        // Bottom border with pointer: └──┬──┘
        let pointer_pos = agent_x.saturating_sub(bx);
        let mut bottom = String::new();
        bottom.push('└');
        for i in 0..text_width {
            if i == pointer_pos {
                bottom.push('┬');
            } else {
                bottom.push('─');
            }
        }
        bottom.push('┘');
        buf.set_string(bx, by + 2, &bottom, border_style);
    }
}

fn short_filename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bubble_expires() {
        let bubble = Bubble {
            agent_id: "a1".to_string(),
            text: "test".to_string(),
            created_at: Instant::now() - Duration::from_secs(10),
            lifetime: Duration::from_secs(5),
        };
        assert!(bubble.is_expired());
    }

    #[test]
    fn test_bubble_not_expired() {
        let bubble = Bubble {
            agent_id: "a1".to_string(),
            text: "test".to_string(),
            created_at: Instant::now(),
            lifetime: Duration::from_secs(5),
        };
        assert!(!bubble.is_expired());
    }

    #[test]
    fn test_max_visible_enforced() {
        let mut mgr = BubbleManager::new();
        mgr.add_bubble_text("a1", "hi");
        mgr.add_bubble_text("a2", "hi");
        mgr.add_bubble_text("a3", "hi");
        mgr.add_bubble_text("a4", "hi");
        assert_eq!(mgr.bubbles.len(), 3);
    }

    #[test]
    fn test_one_bubble_per_agent() {
        let mut mgr = BubbleManager::new();
        mgr.add_bubble_text("a1", "first");
        mgr.add_bubble_text("a1", "second");
        assert_eq!(mgr.bubbles.len(), 1);
        assert_eq!(mgr.bubbles[0].text, "second");
    }

    #[test]
    fn test_tick_removes_expired() {
        let mut mgr = BubbleManager::new();
        mgr.bubbles.push(Bubble {
            agent_id: "a1".to_string(),
            text: "old".to_string(),
            created_at: Instant::now() - Duration::from_secs(10),
            lifetime: Duration::from_secs(3),
        });
        mgr.bubbles.push(Bubble {
            agent_id: "a2".to_string(),
            text: "new".to_string(),
            created_at: Instant::now(),
            lifetime: Duration::from_secs(5),
        });
        mgr.tick();
        assert_eq!(mgr.bubbles.len(), 1);
        assert_eq!(mgr.bubbles[0].agent_id, "a2");
    }

    #[test]
    fn test_short_filename() {
        assert_eq!(short_filename("/Users/me/project/src/main.rs"), "main.rs");
        assert_eq!(short_filename("main.rs"), "main.rs");
    }
}
```

- [ ] **Step 2: Update ui/mod.rs**

```rust
pub mod agent_panel;
pub mod bubbles;
pub mod floor_view;
pub mod sprites;
pub mod stats_bar;
```

- [ ] **Step 3: Run tests**

Run: `cargo test ui::bubbles`
Expected: All 6 bubble tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/ui/bubbles.rs src/ui/mod.rs
git commit -m "feat: add text bubble system with heuristic triggers"
```

---

### Task 8: App State & Input Handler

**Files:**
- Create: `src/app.rs`
- Create: `src/input.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create app.rs**

```rust
use crate::game::state::GameState;
use crate::ui::agent_panel::AgentPanelState;
use crate::ui::bubbles::BubbleManager;
use crate::game::agent::Room;

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
}

impl App {
    pub fn new(floor_width: u16, floor_height: u16) -> Self {
        Self {
            state: GameState::new(floor_width, floor_height),
            agent_panel: AgentPanelState::new(),
            bubbles: BubbleManager::new(),
            focus: Focus::Floor,
            highlighted_room: None,
            running: true,
            tick_count: 0,
            show_help: false,
        }
    }

    pub fn target_fps(&self) -> u64 {
        if self.state.has_animations() || self.bubbles.bubbles.iter().any(|b| !b.is_expired()) {
            15
        } else {
            2
        }
    }

    pub fn frame_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(1000 / self.target_fps())
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Floor => Focus::AgentPanel,
            Focus::AgentPanel => Focus::Floor,
        };
    }

    pub fn tick(&mut self, delta_secs: f32) {
        self.tick_count += 1;

        // Update agent movement
        for agent in &mut self.state.agents {
            if !agent.path.is_empty() {
                let old_x = agent.position.0;
                crate::game::pathfinding::advance_along_path(
                    &mut agent.position,
                    &mut agent.path,
                    4.0, // tiles per second
                    delta_secs,
                );
                // Update facing direction
                if agent.position.0 > old_x {
                    agent.facing = crate::game::agent::Direction::Right;
                } else if agent.position.0 < old_x {
                    agent.facing = crate::game::agent::Direction::Left;
                }
            }
        }

        // Update bubbles
        self.bubbles.tick();

        // Update stats
        self.state.update_stats();
    }
}
```

- [ ] **Step 2: Create input.rs**

```rust
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};

use crate::app::{App, Focus};
use crate::game::agent::Room;

pub fn handle_event(app: &mut App, event: Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(app, key),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        _ => {}
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    // Global keys
    match key.code {
        KeyCode::Char('q') => {
            app.running = false;
            return;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
            return;
        }
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
            return;
        }
        KeyCode::Tab => {
            app.cycle_focus();
            return;
        }
        KeyCode::Char('1') => {
            app.highlighted_room = Some(Room::Workspace);
            app.focus = Focus::Floor;
            return;
        }
        KeyCode::Char('2') => {
            app.highlighted_room = Some(Room::Lounge);
            app.focus = Focus::Floor;
            return;
        }
        KeyCode::Char('3') => {
            app.highlighted_room = Some(Room::CeoOffice);
            app.focus = Focus::Floor;
            return;
        }
        _ => {}
    }

    // Focus-specific keys
    match app.focus {
        Focus::AgentPanel => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.agent_panel.select_next(app.state.agents.len());
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.agent_panel.select_prev(app.state.agents.len());
            }
            KeyCode::Enter => {
                app.agent_panel.toggle_expand();
            }
            _ => {}
        },
        Focus::Floor => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.focus = Focus::AgentPanel;
                app.agent_panel.select_next(app.state.agents.len());
            }
            _ => {}
        },
    }
}

fn handle_mouse(_app: &mut App, _mouse: MouseEvent) {
    // Mouse support is optional — placeholder for future implementation
}
```

- [ ] **Step 3: Add mod declarations to main.rs and verify**

Add to the top of `src/main.rs`:
```rust
mod app;
mod game;
mod input;
mod ui;
```

- [ ] **Step 4: Build**

Run: `cargo build`
Expected: Compiles clean.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/input.rs src/main.rs
git commit -m "feat: add app state with adaptive FPS and input handler"
```

---

### Task 9: JSONL Protocol & Stream Reader

**Files:**
- Create: `src/stream/mod.rs`
- Create: `src/stream/protocol.rs`
- Create: `src/stream/reader.rs`
- Modify: `src/main.rs` (add `mod stream;`)

- [ ] **Step 1: Create stream/mod.rs**

```rust
pub mod protocol;
pub mod reader;
pub mod discovery;
```

- [ ] **Step 2: Create stream/protocol.rs**

```rust
use serde::Deserialize;
use serde_json::Value;

/// Raw JSONL message from Claude Code's --output-format stream-json.
/// Fields are optional because different message types have different shapes.
#[derive(Debug, Deserialize)]
pub struct RawMessage {
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    pub subtype: Option<String>,

    // Common fields
    pub session_id: Option<String>,
    pub model: Option<String>,

    // Text content
    pub content_block: Option<ContentBlock>,

    // Tool use
    pub tool_name: Option<String>,
    pub tool_input: Option<Value>,
    pub tool_result: Option<Value>,

    // Stats (from result messages)
    pub usage: Option<Usage>,
    pub cost_usd: Option<f64>,

    // Catch-all for unknown fields
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: Option<String>,
    pub text: Option<String>,
    pub name: Option<String>,
    pub input: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
}

/// Parsed, typed event for the game engine to consume.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    SessionInit {
        session_id: String,
        model: String,
    },
    ToolUse {
        tool: String,
        args_hint: Option<String>,
    },
    ToolResult {
        tool: String,
    },
    AgentSpawn {
        agent_id: String,
        name: String,
        description: String,
    },
    AgentResult {
        agent_id: String,
    },
    StatsUpdate {
        input_tokens: u64,
        output_tokens: u64,
        cost: f64,
    },
    TextDelta {
        text: String,
    },
    SessionEnd,
    Error {
        message: String,
    },
}

/// Parse a single JSONL line into a StreamEvent.
/// Returns None for unrecognized or irrelevant message types.
pub fn parse_line(line: &str) -> Option<StreamEvent> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let raw: RawMessage = serde_json::from_str(line).ok()?;
    let msg_type = raw.msg_type.as_deref()?;

    match msg_type {
        "system" => {
            // Init message
            if let (Some(session_id), Some(model)) = (raw.session_id, raw.model) {
                Some(StreamEvent::SessionInit { session_id, model })
            } else {
                None
            }
        }
        "assistant" => {
            // Check for tool use in content block
            if let Some(ref block) = raw.content_block {
                match block.block_type.as_deref() {
                    Some("tool_use") => {
                        let tool = block.name.clone().unwrap_or_default();
                        let args_hint = block
                            .input
                            .as_ref()
                            .and_then(|v| {
                                // Try to extract the first string arg as a hint
                                v.as_object()
                                    .and_then(|obj| {
                                        obj.values()
                                            .find_map(|v| v.as_str().map(|s| s.to_string()))
                                    })
                            });

                        // Check if this is an Agent tool call (subagent spawn)
                        if tool == "Agent" {
                            let name = block
                                .input
                                .as_ref()
                                .and_then(|v| v.get("description").and_then(|d| d.as_str()))
                                .unwrap_or("subagent")
                                .to_string();
                            Some(StreamEvent::AgentSpawn {
                                agent_id: format!("agent-{}", rand::random::<u16>()),
                                name,
                                description: args_hint.unwrap_or_default(),
                            })
                        } else {
                            Some(StreamEvent::ToolUse { tool, args_hint })
                        }
                    }
                    Some("text") => {
                        block.text.clone().map(|text| StreamEvent::TextDelta { text })
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        "result" => {
            let cost = raw.cost_usd.unwrap_or(0.0);
            let (input_tokens, output_tokens) = raw
                .usage
                .map(|u| {
                    (
                        u.input_tokens.unwrap_or(0) + u.cache_read_input_tokens.unwrap_or(0),
                        u.output_tokens.unwrap_or(0),
                    )
                })
                .unwrap_or((0, 0));

            Some(StreamEvent::StatsUpdate {
                input_tokens,
                output_tokens,
                cost,
            })
        }
        "error" => {
            let message = raw
                .extra
                .get("error")
                .and_then(|v| v.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
                .to_string();
            Some(StreamEvent::Error { message })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_system_init() {
        let line = r#"{"type":"system","session_id":"abc123","model":"claude-opus-4-6"}"#;
        let event = parse_line(line);
        assert!(matches!(event, Some(StreamEvent::SessionInit { .. })));
        if let Some(StreamEvent::SessionInit { session_id, model }) = event {
            assert_eq!(session_id, "abc123");
            assert_eq!(model, "claude-opus-4-6");
        }
    }

    #[test]
    fn test_parse_tool_use() {
        let line = r#"{"type":"assistant","content_block":{"type":"tool_use","name":"Read","input":{"file_path":"/src/main.rs"}}}"#;
        let event = parse_line(line);
        assert!(matches!(event, Some(StreamEvent::ToolUse { .. })));
        if let Some(StreamEvent::ToolUse { tool, args_hint }) = event {
            assert_eq!(tool, "Read");
            assert_eq!(args_hint, Some("/src/main.rs".to_string()));
        }
    }

    #[test]
    fn test_parse_agent_spawn() {
        let line = r#"{"type":"assistant","content_block":{"type":"tool_use","name":"Agent","input":{"description":"Fix auth bug","prompt":"..."}}}"#;
        let event = parse_line(line);
        assert!(matches!(event, Some(StreamEvent::AgentSpawn { .. })));
    }

    #[test]
    fn test_parse_result_with_stats() {
        let line = r#"{"type":"result","cost_usd":0.42,"usage":{"input_tokens":1000,"output_tokens":500,"cache_read_input_tokens":200}}"#;
        let event = parse_line(line);
        assert!(matches!(event, Some(StreamEvent::StatsUpdate { .. })));
        if let Some(StreamEvent::StatsUpdate { input_tokens, output_tokens, cost }) = event {
            assert_eq!(input_tokens, 1200);
            assert_eq!(output_tokens, 500);
            assert!((cost - 0.42).abs() < 0.001);
        }
    }

    #[test]
    fn test_parse_empty_line() {
        assert!(parse_line("").is_none());
        assert!(parse_line("   ").is_none());
    }

    #[test]
    fn test_parse_invalid_json() {
        assert!(parse_line("not json at all").is_none());
    }

    #[test]
    fn test_parse_unknown_type() {
        let line = r#"{"type":"some_future_type","data":"stuff"}"#;
        assert!(parse_line(line).is_none());
    }
}
```

- [ ] **Step 3: Create stream/reader.rs**

```rust
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

use super::protocol::{parse_line, StreamEvent};

pub struct SessionReader {
    pub session_id: String,
    pub path: PathBuf,
}

/// Message sent from reader tasks to the main thread.
#[derive(Debug)]
pub enum ReaderMessage {
    Event { session_id: String, event: StreamEvent },
    SessionEnded { session_id: String },
    ReaderError { session_id: String, error: String },
}

impl SessionReader {
    pub fn new(session_id: String, path: PathBuf) -> Self {
        Self { session_id, path }
    }

    /// Spawn a tokio task that reads JSONL lines from the file and sends parsed events.
    pub fn spawn(self, tx: mpsc::Sender<ReaderMessage>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            match self.run(&tx).await {
                Ok(()) => {
                    let _ = tx
                        .send(ReaderMessage::SessionEnded {
                            session_id: self.session_id.clone(),
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(ReaderMessage::ReaderError {
                            session_id: self.session_id.clone(),
                            error: e.to_string(),
                        })
                        .await;
                }
            }
        })
    }

    async fn run(&self, tx: &mpsc::Sender<ReaderMessage>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file = tokio::fs::File::open(&self.path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if let Some(event) = parse_line(&line) {
                tx.send(ReaderMessage::Event {
                    session_id: self.session_id.clone(),
                    event,
                })
                .await?;
            }
        }

        Ok(())
    }
}
```

- [ ] **Step 4: Create a stub stream/discovery.rs**

```rust
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum DiscoveryEvent {
    NewSession { session_id: String, path: PathBuf },
    SessionEnded { session_id: String },
}

/// Discover active Claude Code sessions.
/// This is a stub that will be refined after the JSONL spike.
/// For now, it watches a configurable directory for .jsonl files.
pub async fn discover_sessions(
    watch_dir: PathBuf,
    tx: mpsc::Sender<DiscoveryEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initial scan
    if watch_dir.exists() {
        let mut entries = tokio::fs::read_dir(&watch_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                let session_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let _ = tx
                    .send(DiscoveryEvent::NewSession {
                        session_id,
                        path,
                    })
                    .await;
            }
        }
    }

    // TODO: After JSONL spike, implement notify-based file watcher
    // For now, poll every 2 seconds
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        if !watch_dir.exists() {
            continue;
        }

        let mut entries = tokio::fs::read_dir(&watch_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                let session_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let _ = tx
                    .send(DiscoveryEvent::NewSession {
                        session_id,
                        path,
                    })
                    .await;
            }
        }
    }
}
```

- [ ] **Step 5: Add mod stream to main.rs**

Add `mod stream;` to `src/main.rs`.

- [ ] **Step 6: Run tests**

Run: `cargo test stream::protocol`
Expected: All 7 protocol tests pass.

Run: `cargo build`
Expected: Compiles clean.

- [ ] **Step 7: Commit**

```bash
git add src/stream/
git commit -m "feat: add JSONL protocol parser, stream reader, session discovery"
```

---

### Task 10: Demo Mode

**Files:**
- Create: `src/demo.rs`
- Modify: `src/main.rs` (add `mod demo;`)

- [ ] **Step 1: Create demo.rs**

```rust
use std::time::Duration;
use tokio::sync::mpsc;

use crate::stream::protocol::StreamEvent;
use crate::stream::reader::ReaderMessage;

/// Generate synthetic events that simulate a Claude Code session.
/// Used for testing the TUI without a live Claude Code instance.
pub async fn run_demo(tx: mpsc::Sender<ReaderMessage>) {
    let session_id = "demo-session".to_string();

    // Init
    send(&tx, &session_id, StreamEvent::SessionInit {
        session_id: session_id.clone(),
        model: "claude-opus-4-6".to_string(),
    }).await;

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Spawn agent 1
    send(&tx, &session_id, StreamEvent::AgentSpawn {
        agent_id: "agent-01".to_string(),
        name: "Fix auth bug".to_string(),
        description: "Investigating authentication failure".to_string(),
    }).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Agent 1 reads a file
    send(&tx, &session_id, StreamEvent::ToolUse {
        tool: "Read".to_string(),
        args_hint: Some("src/auth/middleware.rs".to_string()),
    }).await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Spawn agent 2
    send(&tx, &session_id, StreamEvent::AgentSpawn {
        agent_id: "agent-02".to_string(),
        name: "Add unit tests".to_string(),
        description: "Writing tests for user service".to_string(),
    }).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Agent 1 edits
    send(&tx, &session_id, StreamEvent::ToolUse {
        tool: "Edit".to_string(),
        args_hint: Some("src/auth/middleware.rs".to_string()),
    }).await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Agent 2 writes tests
    send(&tx, &session_id, StreamEvent::ToolUse {
        tool: "Bash".to_string(),
        args_hint: Some("cargo test auth::tests".to_string()),
    }).await;

    tokio::time::sleep(Duration::from_secs(3)).await;

    // Spawn agent 3
    send(&tx, &session_id, StreamEvent::AgentSpawn {
        agent_id: "agent-03".to_string(),
        name: "Code review".to_string(),
        description: "Reviewing changes".to_string(),
    }).await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Agent 1 finishes
    send(&tx, &session_id, StreamEvent::AgentResult {
        agent_id: "agent-01".to_string(),
    }).await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Stats update
    send(&tx, &session_id, StreamEvent::StatsUpdate {
        input_tokens: 14200,
        output_tokens: 3800,
        cost: 0.42,
    }).await;

    tokio::time::sleep(Duration::from_secs(3)).await;

    // Agent 2 finishes
    send(&tx, &session_id, StreamEvent::AgentResult {
        agent_id: "agent-02".to_string(),
    }).await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Agent 3 hits error
    send(&tx, &session_id, StreamEvent::Error {
        message: "Lint check failed".to_string(),
    }).await;

    tokio::time::sleep(Duration::from_secs(3)).await;

    // Agent 3 recovers and finishes
    send(&tx, &session_id, StreamEvent::ToolUse {
        tool: "Edit".to_string(),
        args_hint: Some("src/lib.rs".to_string()),
    }).await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    send(&tx, &session_id, StreamEvent::AgentResult {
        agent_id: "agent-03".to_string(),
    }).await;

    // Final stats
    send(&tx, &session_id, StreamEvent::StatsUpdate {
        input_tokens: 28400,
        output_tokens: 7200,
        cost: 0.87,
    }).await;

    // Loop the demo after a pause
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Restart — in a real app you might loop, but for now just signal end
    let _ = tx
        .send(ReaderMessage::SessionEnded {
            session_id,
        })
        .await;
}

async fn send(tx: &mpsc::Sender<ReaderMessage>, session_id: &str, event: StreamEvent) {
    let _ = tx
        .send(ReaderMessage::Event {
            session_id: session_id.to_string(),
            event,
        })
        .await;
}
```

- [ ] **Step 2: Add mod demo to main.rs**

Add `mod demo;` to `src/main.rs`.

- [ ] **Step 3: Build**

Run: `cargo build`
Expected: Compiles clean.

- [ ] **Step 4: Commit**

```bash
git add src/demo.rs src/main.rs
git commit -m "feat: add demo mode with synthetic Claude Code events"
```

---

### Task 11: Wire Everything — Main Tick Loop

**Files:**
- Modify: `src/main.rs` (full rewrite)

- [ ] **Step 1: Rewrite main.rs with the full tick loop**

```rust
mod app;
mod demo;
mod game;
mod input;
mod stream;
mod ui;

use std::io;
use std::time::Instant;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use tokio::sync::mpsc;

use app::App;
use game::agent::{Agent, AgentStatus, Room};
use game::pathfinding::compute_path;
use stream::protocol::StreamEvent;
use stream::reader::ReaderMessage;
use ui::agent_panel::AgentPanel;
use ui::floor_view::FloorView;
use ui::stats_bar::StatsBar;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let demo_mode = args.iter().any(|a| a == "--demo");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let size = terminal.size()?;
    let floor_height = size.height.saturating_sub(8); // Reserve space for stats + agent panel
    let mut app = App::new(size.width, floor_height);

    // Channel for stream events
    let (tx, mut rx) = mpsc::channel::<ReaderMessage>(256);

    // Start demo or real discovery
    if demo_mode {
        let demo_tx = tx.clone();
        tokio::spawn(async move {
            loop {
                demo::run_demo(demo_tx.clone()).await;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
    }
    // TODO: else start real session discovery

    let mut last_tick = Instant::now();

    while app.running {
        let frame_duration = app.frame_duration();

        // Render
        terminal.draw(|frame| render(frame, &mut app))?;

        // Poll for events within frame budget
        let timeout = frame_duration
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if event::poll(timeout)? {
            let ev = event::read()?;
            input::handle_event(&mut app, ev);
        }

        // Drain stream events (non-blocking)
        while let Ok(msg) = rx.try_recv() {
            handle_stream_message(&mut app, msg);
        }

        // Tick game state
        let delta = last_tick.elapsed().as_secs_f32();
        last_tick = Instant::now();
        app.tick(delta);
    }

    // Cleanup
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    io::stdout().execute(DisableMouseCapture)?;
    Ok(())
}

fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Layout: floor (variable) | stats (1 line) | agent panel (remaining)
    let floor_height = (area.height as f32 * 0.65) as u16;
    let stats_height = 1u16;
    let panel_height = area.height.saturating_sub(floor_height + stats_height);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(floor_height),
            Constraint::Length(stats_height),
            Constraint::Length(panel_height),
        ])
        .split(area);

    // Floor view
    let floor_view = FloorView {
        state: &app.state,
        highlighted_room: app.highlighted_room,
        tick: app.tick_count,
    };
    frame.render_widget(floor_view, chunks[0]);

    // Render bubbles on top of floor
    for agent in &app.state.agents {
        let ax = chunks[0].x + agent.position.0 as u16;
        let ay = chunks[0].y + agent.position.1 as u16;
        app.bubbles.render_bubble_at(&agent.id, ax, ay, chunks[0], frame.buffer_mut());
    }

    // Stats bar
    let stats_bar = StatsBar {
        stats: &app.state.stats,
    };
    frame.render_widget(stats_bar, chunks[1]);

    // Agent panel
    let agent_panel = AgentPanel {
        agents: &app.state.agents,
    };
    frame.render_stateful_widget(agent_panel, chunks[2], &mut app.agent_panel);
}

fn handle_stream_message(app: &mut App, msg: ReaderMessage) {
    match msg {
        ReaderMessage::Event { session_id, event } => {
            handle_stream_event(app, &session_id, event);
        }
        ReaderMessage::SessionEnded { session_id } => {
            // Mark all agents from this session as finished
            for agent in &mut app.state.agents {
                if agent.session.session_id == session_id {
                    transition_agent(app, agent.id.clone(), AgentStatus::Finished);
                }
            }
        }
        ReaderMessage::ReaderError { session_id, error } => {
            // Find an agent from this session and mark as error
            if let Some(agent) = app.state.agents.iter().find(|a| a.session.session_id == session_id) {
                let id = agent.id.clone();
                app.bubbles.trigger_status_change(&id, AgentStatus::Error);
            }
        }
    }
}

fn handle_stream_event(app: &mut App, session_id: &str, event: StreamEvent) {
    match event {
        StreamEvent::SessionInit { model, session_id: sid } => {
            app.state.stats.model = shorten_model(&model);
            app.state.ceo_status = game::state::CeoStatus::Idle;
        }
        StreamEvent::AgentSpawn { agent_id, name, description } => {
            let session = game::agent::SessionInfo {
                session_id: session_id.to_string(),
                repo: "repo".to_string(),
                branch: "main".to_string(),
                worktree: None,
            };
            let color_idx = app.state.next_color_index;
            app.state.next_color_index += 1;
            let mut agent = Agent::new(agent_id.clone(), name, "".to_string(), session, color_idx);
            agent.task = Some(description);

            // Assign desk and compute path
            if let Some(desk_idx) = app.state.floor.assign_desk() {
                agent.assigned_desk = Some(desk_idx);
                let desk = app.state.floor.desks[desk_idx];
                // Start at nearest door
                let door = app.state.floor.doors.first().unwrap();
                agent.position = (door.x as f32, (door.y + 1) as f32);
                agent.path = compute_path(
                    door.x, door.y + 1,
                    Room::Lounge, Room::Workspace,
                    desk.chair_x, desk.chair_y,
                    &app.state.floor,
                );
                agent.status = AgentStatus::Working;
                agent.target_room = Room::Workspace;
            }

            app.bubbles.trigger_status_change(&agent_id, AgentStatus::Spawning);
            app.state.agents.push(agent);
            app.state.stats.total_tasks += 1;
            app.state.ceo_status = game::state::CeoStatus::Waiting;
        }
        StreamEvent::ToolUse { tool, args_hint } => {
            // Attribute to the most recently active agent in this session
            if let Some(agent) = app.state.agents.iter_mut()
                .filter(|a| a.session.session_id == session_id && a.status == AgentStatus::Working)
                .last()
            {
                agent.current_tool = Some(format!("{}({})", tool, args_hint.as_deref().unwrap_or("")));
                app.bubbles.trigger_tool_use(&agent.id, &tool, args_hint.as_deref());
            }
        }
        StreamEvent::AgentResult { agent_id } => {
            transition_agent(app, agent_id, AgentStatus::Finished);
            app.state.stats.completed_tasks += 1;

            // Check if all agents are done
            let all_done = app.state.agents.iter().all(|a| {
                a.status == AgentStatus::Finished || a.status == AgentStatus::Idle
            });
            if all_done {
                app.state.ceo_status = game::state::CeoStatus::AllComplete;
            }
        }
        StreamEvent::StatsUpdate { input_tokens, output_tokens, cost } => {
            app.state.stats.total_tokens = input_tokens + output_tokens;
            app.state.stats.total_cost = cost;
        }
        StreamEvent::Error { message } => {
            // Mark the most recent working agent as error
            if let Some(agent) = app.state.agents.iter_mut()
                .filter(|a| a.session.session_id == session_id && a.status == AgentStatus::Working)
                .last()
            {
                let id = agent.id.clone();
                agent.status = AgentStatus::Error;
                app.bubbles.trigger_status_change(&id, AgentStatus::Error);
                app.state.ceo_status = game::state::CeoStatus::Error;
            }
        }
        StreamEvent::TextDelta { .. } | StreamEvent::ToolResult { .. } | StreamEvent::SessionEnd => {}
    }
}

fn transition_agent(app: &mut App, agent_id: String, new_status: AgentStatus) {
    if let Some(agent) = app.state.agents.iter_mut().find(|a| a.id == agent_id) {
        let old_status = agent.status;
        agent.status = new_status;
        agent.target_room = Agent::target_room_for_status(new_status);

        // Free desk if leaving workspace
        if agent.target_room != Room::Workspace {
            if let Some(desk_idx) = agent.assigned_desk.take() {
                app.state.floor.free_desk(desk_idx);
            }
        }

        // Compute new path
        let from_room = Agent::target_room_for_status(old_status);
        let (to_x, to_y) = match agent.target_room {
            Room::Lounge => {
                let (_, ly, lw, lh) = app.state.floor.lounge;
                (ly + lw / 2, ly + lh / 2)
            }
            Room::Workspace => {
                if let Some(desk_idx) = agent.assigned_desk {
                    let desk = app.state.floor.desks[desk_idx];
                    (desk.chair_x, desk.chair_y)
                } else {
                    app.state.floor.room_center(Room::Workspace)
                }
            }
            Room::CeoOffice => app.state.floor.ceo_chair,
        };

        agent.path = compute_path(
            agent.position.0 as u16,
            agent.position.1 as u16,
            from_room,
            agent.target_room,
            to_x, to_y,
            &app.state.floor,
        );

        app.bubbles.trigger_status_change(&agent_id, new_status);

        if new_status == AgentStatus::Idle || new_status == AgentStatus::Finished {
            app.bubbles.trigger_lounge_arrival(&agent_id);
        }
    }
}

fn shorten_model(model: &str) -> String {
    model
        .replace("claude-", "")
        .replace("-20251001", "")
        .replace("-", " ")
}
```

- [ ] **Step 2: Build**

Run: `cargo build`
Expected: Compiles clean. Fix any type mismatches between modules.

- [ ] **Step 3: Run in demo mode**

Run: `cargo run -- --demo`
Expected: The TUI launches showing the office floor. Over ~20 seconds, agents appear at doors, walk to desks, show bubbles, and eventually walk to the lounge. Stats bar updates. Press `q` to quit.

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire tick loop, event handling, and demo mode together"
```

---

### Task 12: Help Overlay & Polish

**Files:**
- Modify: `src/main.rs` (add help overlay to render function)

- [ ] **Step 1: Add help overlay rendering**

Add the following to the `render` function in `src/main.rs`, after the agent panel render:

```rust
    // Help overlay
    if app.show_help {
        let help_text = vec![
            "╔═══════════════════════════════════════╗",
            "║          Agents Story — Help          ║",
            "╠═══════════════════════════════════════╣",
            "║  q / Ctrl+C    Quit                   ║",
            "║  j / ↓         Select next agent      ║",
            "║  k / ↑         Select previous agent  ║",
            "║  Enter         Expand/collapse agent   ║",
            "║  Tab           Cycle focus             ║",
            "║  1 / 2 / 3     Focus room              ║",
            "║  ?             Toggle this help         ║",
            "╚═══════════════════════════════════════╝",
        ];
        let help_width = 43u16;
        let help_height = help_text.len() as u16;
        let hx = area.width.saturating_sub(help_width) / 2;
        let hy = area.height.saturating_sub(help_height) / 2;

        for (i, line) in help_text.iter().enumerate() {
            let y = hy + i as u16;
            if y < area.bottom() {
                frame.buffer_mut().set_string(
                    hx, y, line,
                    Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 40)),
                );
            }
        }
    }
```

- [ ] **Step 2: Build and test**

Run: `cargo run -- --demo`
Expected: Press `?` to see help overlay. Press `?` again to dismiss.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add help overlay with keybinding reference"
```

---

### Task 13: Integration Testing & Cleanup

**Files:**
- Modify: various files for compilation fixes
- No new files

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass. If any fail, fix the issues.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -- -W clippy::all`
Expected: No warnings. Fix any clippy suggestions.

- [ ] **Step 3: Test demo mode end-to-end**

Run: `cargo run -- --demo`
Verify:
- [ ] Office renders with 3 rooms (Workspace top, Lounge bottom-left, CEO bottom-right)
- [ ] Agents spawn and walk from door to desks
- [ ] Bubbles appear above agents on status change and tool use
- [ ] Stats bar shows Model, Agents, Tasks, Tokens, Cost
- [ ] Agent panel shows agent list, j/k selects, Enter expands
- [ ] Tab cycles focus, 1/2/3 highlights rooms
- [ ] `?` shows help overlay
- [ ] `q` quits cleanly (terminal restored)
- [ ] CEO sprite visible in CEO Office

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: fix clippy warnings and verify end-to-end demo"
```

---

## Summary

| Task | Description | Key Files |
|------|-------------|-----------|
| 1 | Project scaffold & hello world TUI | `Cargo.toml`, `src/main.rs` |
| 2 | Core game types (agent, floor, state) | `src/game/*.rs` |
| 3 | Waypoint pathfinding | `src/game/pathfinding.rs` |
| 4 | Sprite & color definitions | `src/ui/sprites.rs` |
| 5 | Floor view renderer | `src/ui/floor_view.rs` |
| 6 | Stats bar & agent panel | `src/ui/stats_bar.rs`, `src/ui/agent_panel.rs` |
| 7 | Text bubbles | `src/ui/bubbles.rs` |
| 8 | App state & input handler | `src/app.rs`, `src/input.rs` |
| 9 | JSONL protocol & stream reader | `src/stream/*.rs` |
| 10 | Demo mode | `src/demo.rs` |
| 11 | Wire everything — main tick loop | `src/main.rs` (full rewrite) |
| 12 | Help overlay & polish | `src/main.rs` |
| 13 | Integration testing & cleanup | Various |

**Post-plan spike (not in this plan):** Run `claude --output-format stream-json` to capture actual JSONL schema. Update `src/stream/protocol.rs` and `src/stream/discovery.rs` based on findings.
