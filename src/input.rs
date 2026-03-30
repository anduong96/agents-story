use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};

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
            KeyCode::Char('j') | KeyCode::Down => {
                app.agent_panel.select_next(app.state.agents.len())
            }
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
    match mouse.kind {
        MouseEventKind::Down(_) => {
            let y = mouse.row;
            if let Some(panel_y) = app.panel_top {
                if y > panel_y {
                    let row = (y - panel_y - 1) as usize;
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
        MouseEventKind::ScrollUp => {
            if let Some(panel_y) = app.panel_top {
                if mouse.row >= panel_y {
                    app.agent_panel.scroll_up();
                } else {
                    app.floor_scroll_y = app.floor_scroll_y.saturating_sub(3);
                }
            } else {
                app.floor_scroll_y = app.floor_scroll_y.saturating_sub(3);
            }
        }
        MouseEventKind::ScrollDown => {
            if let Some(panel_y) = app.panel_top {
                if mouse.row >= panel_y {
                    app.agent_panel.scroll_down();
                } else {
                    app.floor_scroll_y += 3;
                }
            } else {
                app.floor_scroll_y += 3;
            }
        }
        _ => {}
    }
}
