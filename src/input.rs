use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};

use crate::app::{App, Focus};

pub fn handle_event(app: &mut App, event: Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(app, key),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        _ => {}
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    // Global keys — work regardless of focus.
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
        _ => {}
    }

    // Focus-specific keys.
    match app.focus {
        Focus::AgentPanel => match key.code {
            KeyCode::Char('j') | KeyCode::Down => app.agent_panel.select_next(app.state.agents.len()),
            KeyCode::Char('k') | KeyCode::Up => app.agent_panel.select_prev(app.state.agents.len()),
            KeyCode::Enter => app.agent_panel.toggle_expand(),
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

fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    if let MouseEventKind::Down(_) = mouse.kind {
        let y = mouse.row;
        // Check if click is in the agent panel area
        // The panel starts after the floor (65%) and stats bar (1 line)
        // Each agent row is 1 line, with a border at top
        if let Some(panel_y) = app.panel_top {
            if y > panel_y {
                let row = (y - panel_y - 1) as usize; // -1 for border
                let agent_count = app.state.agents.len();
                if row < agent_count {
                    app.focus = Focus::AgentPanel;
                    if app.agent_panel.selected == Some(row) {
                        app.agent_panel.toggle_expand();
                    } else {
                        app.agent_panel.selected = Some(row);
                        app.agent_panel.expanded = None;
                    }
                }
            }
        }
    }
}
