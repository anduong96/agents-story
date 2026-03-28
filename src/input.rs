use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};

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

    // Focus-specific keys.
    match app.focus {
        Focus::AgentPanel => match key.code {
            KeyCode::Char('j') | KeyCode::Down => app.agent_panel.select_next(),
            KeyCode::Char('k') | KeyCode::Up => app.agent_panel.select_prev(),
            KeyCode::Enter => app.agent_panel.toggle_expand(),
            _ => {}
        },
        Focus::Floor => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.focus = Focus::AgentPanel;
                app.agent_panel.select_next();
            }
            _ => {}
        },
    }
}

fn handle_mouse(_app: &mut App, _mouse: MouseEvent) {
    // Mouse handling — stub for future implementation.
}
