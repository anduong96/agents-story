mod app;
mod demo;
mod game;
mod input;
mod stream;
mod ui;

use std::io;
use std::time::Instant;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame, Terminal,
};
use tokio::sync::mpsc;

use app::App;
use game::agent::{Agent, AgentStatus, Room, SessionInfo};
use game::pathfinding::compute_path;
use stream::protocol::StreamEvent;
use stream::reader::ReaderMessage;
use ui::agent_panel::AgentPanel;
use ui::bubbles::AgentStatus as BubbleAgentStatus;
use ui::floor_view::FloorView;
use ui::stats_bar::StatsBar;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DEFAULT_FLOOR_WIDTH: u16 = 80;
const DEFAULT_FLOOR_HEIGHT: u16 = 30;
const MAX_WIDTH: u16 = 120;
const MAX_HEIGHT: u16 = 50;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> io::Result<()> {
    let demo_mode = std::env::args().any(|a| a == "--demo");

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run(demo_mode, &mut terminal).await;

    // Terminal teardown
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Main run loop
// ---------------------------------------------------------------------------

async fn run(
    demo_mode: bool,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    // Get initial terminal size for floor dimensions
    let term_size = terminal.size()?;
    let floor_w = term_size.width.max(DEFAULT_FLOOR_WIDTH);
    let floor_h = (term_size.height as f32 * 0.65) as u16;
    let floor_h = floor_h.max(DEFAULT_FLOOR_HEIGHT);
    let mut app = App::new(floor_w, floor_h);

    let (tx, mut rx) = mpsc::channel::<ReaderMessage>(256);

    // Spawn demo producer if --demo flag was set.
    if demo_mode {
        tokio::spawn(demo::run_demo(tx));
    }

    let mut last_tick = Instant::now();

    while app.running {
        // --- Draw ---
        terminal.draw(|frame| render(frame, &mut app))?;

        // --- Input ---
        let timeout = app.frame_duration();
        if event::poll(timeout)? {
            let ev = event::read()?;
            input::handle_event(&mut app, ev);
        }

        // --- Drain stream events ---
        while let Ok(msg) = rx.try_recv() {
            handle_stream_message(&mut app, msg);
        }

        // --- Tick game state ---
        let now = Instant::now();
        let delta = now.duration_since(last_tick);
        last_tick = now;
        app.tick(delta.as_secs_f32());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Clamp to max dimensions and center
    let clamped_w = size.width.min(MAX_WIDTH);
    let clamped_h = size.height.min(MAX_HEIGHT);
    let offset_x = (size.width.saturating_sub(clamped_w)) / 2;
    let offset_y = (size.height.saturating_sub(clamped_h)) / 2;
    let centered = Rect::new(offset_x, offset_y, clamped_w, clamped_h);

    // Layout: floor (65%) | stats bar (1 line) | agent panel (rest)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(65),
            Constraint::Length(1),
            Constraint::Min(3),
        ])
        .split(centered);

    let floor_area = chunks[0];
    let stats_area = chunks[1];
    let panel_area = chunks[2];

    // Resize floor to fill available pane
    app.resize_floor(floor_area.width, floor_area.height);

    // 1. Floor view
    let floor_view = FloorView::new(&app.state)
        .with_highlight(app.highlighted_room)
        .with_tick(app.tick_count);
    frame.render_widget(floor_view, floor_area);

    // 2. Bubbles (rendered on top of floor)
    render_bubbles(frame, app, floor_area);

    // 3. Stats bar
    let stats_bar = StatsBar::new(&app.state.stats);
    frame.render_widget(stats_bar, stats_area);

    // 4. Agent panel (stateful widget)
    let agent_panel = AgentPanel::new(&app.state.agents);
    frame.render_stateful_widget(agent_panel, panel_area, &mut app.agent_panel);

    // 5. Help overlay (rendered on top of everything)
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
        let hx = size.width.saturating_sub(help_width) / 2;
        let hy = size.height.saturating_sub(help_height) / 2;

        for (i, line) in help_text.iter().enumerate() {
            let y = hy + i as u16;
            if y < size.bottom() {
                frame.buffer_mut().set_string(
                    hx, y, line,
                    ratatui::style::Style::default()
                        .fg(ratatui::style::Color::White)
                        .bg(ratatui::style::Color::Rgb(30, 30, 40)),
                );
            }
        }
    }
}

fn render_bubbles(frame: &mut Frame, app: &App, floor_area: Rect) {
    let buf = frame.buffer_mut();
    for agent in &app.state.agents {
        let ax = agent.position.0 as u16;
        let ay = agent.position.1 as u16;
        app.bubbles
            .render_bubble_at(&agent.id, ax, ay, floor_area, buf);
    }
}

// ---------------------------------------------------------------------------
// Stream message handling
// ---------------------------------------------------------------------------

fn handle_stream_message(app: &mut App, msg: ReaderMessage) {
    match msg {
        ReaderMessage::Event { session_id, event } => {
            handle_stream_event(app, &session_id, event);
        }
        ReaderMessage::SessionEnded { session_id } => {
            // Mark all agents from this session as Finished.
            let ids: Vec<String> = app
                .state
                .agents
                .iter()
                .filter(|a| a.session.session_id == session_id)
                .map(|a| a.id.clone())
                .collect();
            for id in ids {
                transition_agent(app, &id, AgentStatus::Finished);
            }
            app.state.ceo_status = game::state::CeoStatus::AllComplete;
        }
        ReaderMessage::ReaderError {
            session_id,
            error: _,
        } => {
            // Mark all agents from this session as Error.
            let ids: Vec<String> = app
                .state
                .agents
                .iter()
                .filter(|a| a.session.session_id == session_id)
                .map(|a| a.id.clone())
                .collect();
            for id in ids {
                transition_agent(app, &id, AgentStatus::Error);
            }
        }
    }
}

fn handle_stream_event(app: &mut App, session_id: &str, event: StreamEvent) {
    match event {
        StreamEvent::SessionInit { model, .. } => {
            app.state.stats.model = shorten_model(&model);
            app.state.ceo_status = game::state::CeoStatus::Waiting;
        }

        StreamEvent::AgentSpawn {
            agent_id,
            name,
            description,
        } => {
            let color_index = app.state.next_color_index;
            app.state.next_color_index += 1;

            let session = SessionInfo {
                session_id: session_id.to_string(),
                repo: "agents-story".to_string(),
                branch: "main".to_string(),
                worktree: None,
            };

            let mut agent = Agent::new(
                agent_id.clone(),
                name,
                app.state.stats.model.clone(),
                session,
                color_index,
            );
            agent.task = Some(description);

            // Assign a desk and compute the path from the nearest door to the desk.
            if let Some(desk_idx) = app.state.floor.assign_desk(&agent.name) {
                agent.assigned_desk = Some(desk_idx);
                app.state.floor.desks[desk_idx].agent_color = Some(agent.sprite_color);
                let desk = &app.state.floor.desks[desk_idx];
                let target_x = desk.chair_x;
                let target_y = desk.chair_y;

                // Spawn at the first workspace-lounge door.
                let spawn_door = app.state.floor.doors.first();
                let (start_x, start_y) = spawn_door
                    .map(|d| (d.x, d.y.saturating_sub(1)))
                    .unwrap_or((1, 1));

                agent.position = (start_x as f32, start_y as f32);
                agent.path = compute_path(
                    start_x,
                    start_y,
                    Room::Workspace,
                    Room::Workspace,
                    target_x,
                    target_y,
                    &app.state.floor,
                );
            }

            agent.status = AgentStatus::Working;
            app.state.stats.total_tasks += 1;

            // Trigger bubble
            app.bubbles
                .trigger_status_change(&agent_id, BubbleAgentStatus::Spawning);

            app.state.agents.push(agent);
        }

        StreamEvent::ToolUse { tool, args_hint } => {
            // Find the most recently added agent for this session that is Working.
            if let Some(agent) = app
                .state
                .agents
                .iter_mut()
                .rev()
                .find(|a| a.session.session_id == session_id && a.status == AgentStatus::Working)
            {
                agent.current_tool = Some(tool.clone());
                app.bubbles
                    .trigger_tool_use(&agent.id, &tool, args_hint.as_deref());
            }
        }

        StreamEvent::ToolResult { .. } => {
            // No-op for now; tool results don't change visual state.
        }

        StreamEvent::AgentResult { agent_id } => {
            transition_agent(app, &agent_id, AgentStatus::Finished);
            app.state.stats.completed_tasks += 1;

            // Check if all agents are now finished.
            let all_done = app
                .state
                .agents
                .iter()
                .all(|a| matches!(a.status, AgentStatus::Finished | AgentStatus::Error));
            if all_done && !app.state.agents.is_empty() {
                app.state.ceo_status = game::state::CeoStatus::AllComplete;
            }
        }

        StreamEvent::StatsUpdate {
            input_tokens,
            output_tokens,
            cost,
        } => {
            app.state.stats.total_tokens = input_tokens + output_tokens;
            app.state.stats.total_cost = cost;
        }

        StreamEvent::TextDelta { .. } => {
            // Text deltas are informational; no visual side-effect for now.
        }

        StreamEvent::SessionEnd => {
            // Handled by ReaderMessage::SessionEnded.
        }

        StreamEvent::Error { message: _ } => {
            // Find the most recently active agent in this session and mark it Error.
            let agent_id = app
                .state
                .agents
                .iter()
                .rev()
                .find(|a| a.session.session_id == session_id && a.status == AgentStatus::Working)
                .map(|a| a.id.clone());
            if let Some(id) = agent_id {
                transition_agent(app, &id, AgentStatus::Error);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Transition an agent to a new status, updating target room, path, desk, and bubbles.
fn transition_agent(app: &mut App, agent_id: &str, new_status: AgentStatus) {
    let floor = &app.state.floor;

    // Find agent index.
    let idx = match app.state.agents.iter().position(|a| a.id == agent_id) {
        Some(i) => i,
        None => return,
    };

    let new_room = Agent::target_room_for_status(&new_status);

    // Pre-compute target position and path before mutably borrowing the agent.
    let (target_x, target_y) = floor.room_center(new_room);
    let agent = &app.state.agents[idx];
    let from_x = agent.position.0 as u16;
    let from_y = agent.position.1 as u16;
    let from_room = agent.target_room;
    let path = compute_path(from_x, from_y, from_room, new_room, target_x, target_y, floor);
    let old_desk = agent.assigned_desk;

    // Now mutate the agent.
    let agent = &mut app.state.agents[idx];
    agent.status = new_status.clone();
    agent.target_room = new_room;
    agent.path = path;

    // Free desk if leaving workspace.
    if new_room != Room::Workspace {
        if let Some(desk_idx) = old_desk {
            app.state.floor.free_desk(desk_idx);
            app.state.agents[idx].assigned_desk = None;
        }
    }

    // Trigger bubble.
    let bubble_status = to_bubble_status(&new_status);
    app.bubbles
        .trigger_status_change(agent_id, bubble_status);
}

/// Convert game::agent::AgentStatus to ui::bubbles::AgentStatus.
fn to_bubble_status(status: &AgentStatus) -> BubbleAgentStatus {
    match status {
        AgentStatus::Working => BubbleAgentStatus::Working,
        AgentStatus::Idle => BubbleAgentStatus::Idle,
        AgentStatus::Spawning => BubbleAgentStatus::Spawning,
        AgentStatus::Finished => BubbleAgentStatus::Finished,
        AgentStatus::Error => BubbleAgentStatus::Error,
    }
}

/// Shorten model names: strip "claude-" prefix and trailing date suffixes like "-20260301".
fn shorten_model(model: &str) -> String {
    let s = model.strip_prefix("claude-").unwrap_or(model);
    // Strip trailing date suffix: -YYYYMMDD pattern (8 digits after dash at end).
    if s.len() > 9 {
        let suffix = &s[s.len() - 9..];
        if suffix.starts_with('-') && suffix[1..].chars().all(|c| c.is_ascii_digit()) {
            return s[..s.len() - 9].to_string();
        }
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shorten_model_strips_prefix_and_date() {
        assert_eq!(shorten_model("claude-opus-4-6-20260301"), "opus-4-6");
    }

    #[test]
    fn test_shorten_model_strips_prefix_only() {
        assert_eq!(shorten_model("claude-opus-4-6"), "opus-4-6");
    }

    #[test]
    fn test_shorten_model_no_prefix() {
        assert_eq!(shorten_model("gpt-4o"), "gpt-4o");
    }

    #[test]
    fn test_to_bubble_status_mapping() {
        assert_eq!(to_bubble_status(&AgentStatus::Working), BubbleAgentStatus::Working);
        assert_eq!(to_bubble_status(&AgentStatus::Finished), BubbleAgentStatus::Finished);
        assert_eq!(to_bubble_status(&AgentStatus::Error), BubbleAgentStatus::Error);
    }
}
