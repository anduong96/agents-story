mod app;
mod demo;
mod game;
mod input;
mod stream;
mod ui;

use std::io::{self, IsTerminal};
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
const MAX_HEIGHT: u16 = 200;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> io::Result<()> {
    let demo_mode = std::env::args().any(|a| a == "--demo");
    let speed = if std::env::args().any(|a| a == "--extreme") {
        20.0
    } else if std::env::args().any(|a| a == "--fast") {
        10.0
    } else {
        5.0
    };

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run(demo_mode, speed, &mut terminal).await;

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
    speed: f32,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    // Get initial terminal size for floor dimensions
    let term_size = terminal.size()?;
    let floor_w = term_size.width.clamp(DEFAULT_FLOOR_WIDTH, MAX_WIDTH);
    let floor_h = ((term_size.height as f32 * 0.65) as u16).clamp(DEFAULT_FLOOR_HEIGHT, MAX_HEIGHT);
    let mut app = App::new(floor_w, floor_h);

    // Create 6 permanent staff agents, idle in the lounge
    create_staff_agents(&mut app);

    let (tx, mut rx) = mpsc::channel::<ReaderMessage>(256);

    // Spawn event producer: demo, stdin pipe, or auto-discover.
    if demo_mode {
        tokio::spawn(demo::run_demo(tx));
    } else if !io::stdin().is_terminal() {
        tokio::spawn(stream::stdin_reader::read_stdin(tx));
    } else {
        tokio::spawn(stream::watcher::watch_sessions(tx));
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
        app.tick(delta.as_secs_f32() * speed);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Clamp width and center horizontally; use full height
    let clamped_w = size.width.min(MAX_WIDTH);
    let offset_x = (size.width.saturating_sub(clamped_w)) / 2;
    let centered = Rect::new(offset_x, 0, clamped_w, size.height);

    // Layout: floor (actual height) | stats bar (1 line) | agent panel (rest)
    let floor_h = app.state.floor.height.max(DEFAULT_FLOOR_HEIGHT);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(floor_h),
            Constraint::Length(1),
            Constraint::Min(3),
        ])
        .split(centered);

    let floor_area = chunks[0];
    let stats_area = chunks[1];
    let panel_area = chunks[2];

    // Track panel position for mouse click handling
    app.panel_top = Some(panel_area.y);

    // Resize floor to fill available pane (clamped by centered layout)
    app.resize_floor(
        floor_area.width.min(MAX_WIDTH),
        floor_area.height.min(MAX_HEIGHT),
    );

    // Auto-scroll: if floor is taller than view, scroll to keep workspace desks visible
    let floor_h = app.state.floor.height;
    if floor_h > floor_area.height {
        // Keep the workspace area in view by default
        // If there are working agents, scroll to show them
        let max_scroll = floor_h.saturating_sub(floor_area.height);
        app.floor_scroll_y = app.floor_scroll_y.min(max_scroll);
    } else {
        app.floor_scroll_y = 0;
    }

    // 1. Floor view
    let floor_view = FloorView::new(&app.state)
        .with_tick(app.tick_count)
        .with_scroll(app.floor_scroll_y)
        .with_ceo_pos(app.ceo_pos);
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
                    hx,
                    y,
                    line,
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
    // CEO bubble
    let cx = app.ceo_pos.0 as u16;
    let cy = app.ceo_pos.1 as u16;
    app.bubbles.render_bubble_at("ceo", cx, cy, floor_area, buf);
}

// ---------------------------------------------------------------------------
// Stream message handling
// ---------------------------------------------------------------------------

fn handle_stream_message(app: &mut App, msg: ReaderMessage) {
    match msg {
        ReaderMessage::Event {
            session_id,
            project,
            event,
        } => {
            handle_stream_event(app, &session_id, &project, event);
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

fn handle_stream_event(app: &mut App, session_id: &str, project: &str, event: StreamEvent) {
    match event {
        StreamEvent::SessionInit { model, .. } => {
            app.state.stats.model = shorten_model(&model);
            app.state.ceo_status = game::state::CeoStatus::Waiting;
            ensure_main_agent(app, session_id, project);
        }
        StreamEvent::AgentSpawn {
            agent_id,
            name,
            description,
        } => handle_agent_spawn(app, session_id, project, agent_id, name, description),
        StreamEvent::ToolUse { tool, args_hint } => {
            ensure_main_agent(app, session_id, project);
            handle_tool_use(app, session_id, tool, args_hint);
        }
        StreamEvent::ToolResult { .. } => {}
        StreamEvent::AgentResult { agent_id } => handle_agent_result(app, agent_id),
        StreamEvent::StatsUpdate {
            input_tokens,
            output_tokens,
            cost,
        } => {
            app.state.stats.total_tokens = input_tokens + output_tokens;
            app.state.stats.total_cost = cost;
        }
        StreamEvent::TextDelta { .. } | StreamEvent::SessionEnd => {}
        StreamEvent::Error { message: _ } => handle_agent_error(app, session_id),
    }
}

/// Ensure a "main" agent exists for this session. The main agent represents
/// the primary Claude Code instance (not a sub-agent). It gets assigned a
/// desk and shows tool activity just like spawned agents.
fn ensure_main_agent(app: &mut App, session_id: &str, project: &str) {
    // Check if an agent already exists for this session.
    let exists = app
        .state
        .agents
        .iter()
        .any(|a| a.session.session_id == session_id && a.status == AgentStatus::Working);
    if exists {
        return;
    }

    let main_id = format!("main-{}", session_id);
    handle_agent_spawn(
        app,
        session_id,
        project,
        main_id,
        "Claude".to_string(),
        "Main session".to_string(),
    );
}

fn handle_agent_spawn(
    app: &mut App,
    session_id: &str,
    project: &str,
    agent_id: String,
    name: String,
    description: String,
) {
    app.state.stats.total_tasks += 1;

    // CEO runs to whiteboard and yells the task
    if app.ceo_path.is_empty() {
        let wb = app.state.floor.whiteboard_pos;
        app.ceo_returning = false;
        app.ceo_path = compute_path(
            app.ceo_pos.0 as u16,
            app.ceo_pos.1 as u16,
            Room::CeoOffice,
            Room::Workspace,
            wb.0,
            wb.1,
            &app.state.floor,
        );
        let yell = format!("NEW TASK: {}!", name.to_uppercase());
        app.bubbles.trigger_ceo_yell(yell);
    }

    // Try to assign an idle staff agent first
    let idle_idx = app
        .state
        .agents
        .iter()
        .position(|a| a.status == AgentStatus::Idle);

    if let Some(idx) = idle_idx {
        assign_staff_agent(app, idx, &agent_id, session_id, description);
    } else {
        hire_temp_agent(app, &agent_id, session_id, project, name, description);
    }
}

fn assign_staff_agent(
    app: &mut App,
    idx: usize,
    agent_id: &str,
    session_id: &str,
    description: String,
) {
    let agent = &mut app.state.agents[idx];
    agent.id = agent_id.to_string();
    agent.task = Some(description);
    agent.session.session_id = session_id.to_string();
    agent.status = AgentStatus::Working;
    agent.current_tool = None;
    agent.tokens = 0;
    agent.cost = 0.0;
    agent.started_at = std::time::Instant::now();

    let agent_name = agent.name.clone();
    let sprite_color = agent.sprite_color;
    let from_x = agent.position.0 as u16;
    let from_y = agent.position.1 as u16;

    if let Some(desk_idx) = app.state.floor.assign_desk(&agent_name) {
        app.state.agents[idx].assigned_desk = Some(desk_idx);
        app.state.floor.desks[desk_idx].agent_color = Some(sprite_color);
        let desk = &app.state.floor.desks[desk_idx];
        let path = compute_path(
            from_x,
            from_y,
            Room::Lounge,
            Room::Workspace,
            desk.chair_x,
            desk.chair_y,
            &app.state.floor,
        );
        app.state.agents[idx].path = path;
        app.state.agents[idx].target_room = Room::Workspace;
    }

    sync_agent_positions(app);
    app.bubbles
        .trigger_status_change(agent_id, BubbleAgentStatus::Working);
}

fn hire_temp_agent(
    app: &mut App,
    agent_id: &str,
    session_id: &str,
    project: &str,
    name: String,
    description: String,
) {
    let color_index = app.state.next_color_index;
    app.state.next_color_index += 1;

    let session = SessionInfo {
        session_id: session_id.to_string(),
        repo: project.to_string(),
    };

    let mut agent = Agent::new(
        agent_id.to_string(),
        name,
        app.state.stats.model.clone(),
        session,
        color_index,
    );
    agent.task = Some(description);
    agent.is_permanent = false;

    if let Some(desk_idx) = app.state.floor.assign_desk(&agent.name) {
        agent.assigned_desk = Some(desk_idx);
        app.state.floor.desks[desk_idx].agent_color = Some(agent.sprite_color);
        let desk = &app.state.floor.desks[desk_idx];

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
            desk.chair_x,
            desk.chair_y,
            &app.state.floor,
        );
    }

    agent.status = AgentStatus::Working;
    app.bubbles
        .trigger_status_change(agent_id, BubbleAgentStatus::Spawning);
    app.state.agents.push(agent);
    sync_agent_positions(app);
}

fn handle_tool_use(app: &mut App, session_id: &str, tool: String, args_hint: Option<String>) {
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

fn handle_agent_result(app: &mut App, agent_id: String) {
    app.state.stats.completed_tasks += 1;

    let is_permanent = app
        .state
        .agents
        .iter()
        .find(|a| a.id == agent_id)
        .map(|a| a.is_permanent)
        .unwrap_or(false);

    if is_permanent {
        transition_agent(app, &agent_id, AgentStatus::Idle);
    } else {
        // Temp leaves via top exit door
        let exit = app.state.floor.exit_pos;
        if let Some(i) = app.state.agents.iter().position(|a| a.id == agent_id) {
            let agent = &mut app.state.agents[i];
            agent.status = AgentStatus::Finished;
            agent.target_room = Room::Workspace;
            agent.path = vec![(exit.0, exit.1)];
            if let Some(desk_idx) = agent.assigned_desk.take() {
                app.state.floor.free_desk(desk_idx);
            }
            app.bubbles
                .trigger_status_change(&agent_id, BubbleAgentStatus::Finished);
        }
    }

    // Check if all working agents are done
    let all_done = app
        .state
        .agents
        .iter()
        .filter(|a| !a.is_permanent)
        .all(|a| {
            matches!(
                a.status,
                AgentStatus::Finished | AgentStatus::Error | AgentStatus::Idle
            )
        });
    let any_working = app
        .state
        .agents
        .iter()
        .any(|a| matches!(a.status, AgentStatus::Working | AgentStatus::Spawning));
    if all_done && !any_working && app.state.stats.total_tasks > 0 {
        app.state.ceo_status = game::state::CeoStatus::AllComplete;
    }
}

fn handle_agent_error(app: &mut App, session_id: &str) {
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
    let path = compute_path(
        from_x, from_y, from_room, new_room, target_x, target_y, floor,
    );
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
    app.bubbles.trigger_status_change(agent_id, bubble_status);
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

const STAFF_NAMES: [&str; 6] = ["Alice", "Bob", "Carol", "Dave", "Eve", "Frank"];

/// After relayout, snap all seated agents to their (possibly moved) desk positions.
fn sync_agent_positions(app: &mut App) {
    for agent in &mut app.state.agents {
        if let Some(desk_idx) = agent.assigned_desk {
            if !agent.is_animating() {
                if let Some(desk) = app.state.floor.desks.get(desk_idx) {
                    agent.position = (desk.chair_x as f32, desk.chair_y as f32);
                }
            }
        }
    }
}

/// Create 6 permanent staff agents, idle in the lounge.
fn create_staff_agents(app: &mut App) {
    let floor = &app.state.floor;
    let lounge_center = floor.room_center(Room::Lounge);

    for (i, name) in STAFF_NAMES.iter().enumerate() {
        let session = SessionInfo {
            session_id: "staff".to_string(),
            repo: String::new(),
        };

        let mut agent = Agent::new(
            format!("staff-{}", i),
            name.to_string(),
            String::new(),
            session,
            i,
        );
        agent.is_permanent = true;
        agent.status = AgentStatus::Idle;
        agent.target_room = Room::Lounge;

        // Scatter around lounge center
        let offset_x = (i as i16 % 3 - 1) * 4;
        let offset_y = (i as i16 / 3) * 3;
        let px = (lounge_center.0 as i16 + offset_x).max(2) as f32;
        let py = (lounge_center.1 as i16 + offset_y).max(2) as f32;
        agent.position = (px, py);

        app.state.agents.push(agent);
    }
    app.state.next_color_index = STAFF_NAMES.len();
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
        assert_eq!(
            to_bubble_status(&AgentStatus::Working),
            BubbleAgentStatus::Working
        );
        assert_eq!(
            to_bubble_status(&AgentStatus::Finished),
            BubbleAgentStatus::Finished
        );
        assert_eq!(
            to_bubble_status(&AgentStatus::Error),
            BubbleAgentStatus::Error
        );
    }
}
