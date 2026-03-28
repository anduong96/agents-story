use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::game::agent::{AgentStatus, Direction, Room};
use crate::game::floor::{CellType, Floor};
use crate::game::state::GameState;
use crate::ui::sprites;

pub struct FloorView<'a> {
    pub state: &'a GameState,
    pub highlighted_room: Option<Room>,
    pub tick: u64,
}

impl<'a> FloorView<'a> {
    pub fn new(state: &'a GameState) -> Self {
        FloorView {
            state,
            highlighted_room: None,
            tick: 0,
        }
    }

    pub fn with_highlight(mut self, room: Option<Room>) -> Self {
        self.highlighted_room = room;
        self
    }

    pub fn with_tick(mut self, tick: u64) -> Self {
        self.tick = tick;
        self
    }
}

/// Returns the background color for the given grid cell based on which room it's in.
fn room_bg_at(floor: &Floor, x: usize, y: usize) -> Color {
    let workspace_h = floor.workspace.3 as usize; // height of workspace
    let lounge_w = floor.lounge.2 as usize;       // width of lounge

    if y < workspace_h {
        sprites::WORKSPACE_BG
    } else if x < lounge_w {
        sprites::LOUNGE_BG
    } else {
        sprites::CEO_BG
    }
}

/// Returns whether a cell is on the border of the given room rect (x, y, w, h).
fn is_room_border(rx: u16, ry: u16, rw: u16, rh: u16, cx: usize, cy: usize) -> bool {
    let x = cx as u16;
    let y = cy as u16;
    x >= rx && x < rx + rw && y >= ry && y < ry + rh
        && (x == rx || x == rx + rw - 1 || y == ry || y == ry + rh - 1)
}

impl<'a> Widget for FloorView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let floor = &self.state.floor;
        let grid_w = floor.width as usize;
        let grid_h = floor.height as usize;

        // Render each grid cell
        for gy in 0..grid_h {
            for gx in 0..grid_w {
                let screen_x = area.x + gx as u16;
                let screen_y = area.y + gy as u16;

                // Bounds check
                if screen_x >= area.x + area.width || screen_y >= area.y + area.height {
                    continue;
                }

                let cell_type = floor.grid[gy][gx];
                let bg = room_bg_at(floor, gx, gy);

                // Determine highlight override for border cells
                let highlight_bg = if let Some(ref room) = self.highlighted_room {
                    let (rx, ry, rw, rh) = match room {
                        Room::Workspace => floor.workspace,
                        Room::Lounge => floor.lounge,
                        Room::CeoOffice => floor.ceo_office,
                    };
                    if is_room_border(rx, ry, rw, rh, gx, gy) {
                        Some(Color::Rgb(80, 80, 40))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let (ch, fg, actual_bg) = match cell_type {
                    CellType::Wall => ('│', sprites::WALL_COLOR, Color::Reset),
                    CellType::Door => ('▒', sprites::DOOR_COLOR, Color::Reset),
                    CellType::Desk => ('▄', sprites::DESK_COLOR, Color::Reset),
                    CellType::Monitor => ('▓', sprites::MONITOR_COLOR, bg),
                    CellType::CeoDesk => ('▄', Color::Rgb(180, 120, 60), sprites::CEO_BG),
                    CellType::CeoMonitor => {
                        let monitor_fg = if (self.tick % 60) < 30 {
                            sprites::MONITOR_COLOR
                        } else {
                            sprites::MONITOR_FLICKER_COLOR
                        };
                        ('▓', monitor_fg, sprites::CEO_BG)
                    }
                    CellType::PingPongTable => ('▒', sprites::PING_PONG_COLOR, bg),
                    CellType::Empty => (' ', Color::Reset, bg),
                };

                let final_bg = highlight_bg.unwrap_or(actual_bg);
                let cell = buf.cell_mut((screen_x, screen_y));
                if let Some(cell) = cell {
                    cell.set_char(ch);
                    cell.set_style(Style::default().fg(fg).bg(final_bg));
                }
            }
        }

        // Render room labels
        self.render_room_labels(floor, area, buf);

        // Render horizontal wall segments (top/bottom borders use box-drawing chars)
        self.render_horizontal_walls(floor, area, buf);

        // Render agents
        for agent in &self.state.agents {
            self.render_agent(agent, area, buf);
        }

        // Render CEO sprite at ceo_chair position
        self.render_ceo(floor, area, buf);
    }
}

impl<'a> FloorView<'a> {
    fn render_room_labels(&self, floor: &Floor, area: Rect, buf: &mut Buffer) {
        let divider_y = floor.workspace.3 as u16;
        let lounge_w = floor.lounge.2 as u16;

        // "Workspace" label — centered horizontally in workspace, one row above divider
        if divider_y > 1 {
            let label = "[ Workspace ]";
            let lx = area.x + floor.workspace.2 / 4;
            let ly = area.y + divider_y - 1;
            self.render_label(label, lx, ly, Color::Rgb(150, 150, 180), area, buf);
        }

        // "Lounge" label — in lounge section
        {
            let label = "[ Lounge ]";
            let lx = area.x + lounge_w / 4;
            let ly = area.y + divider_y + 1;
            self.render_label(label, lx, ly, Color::Rgb(180, 140, 120), area, buf);
        }

        // "CEO" label — in CEO section
        {
            let label = "[ CEO ]";
            let ceo_x = lounge_w + (floor.ceo_office.2 / 4);
            let lx = area.x + ceo_x;
            let ly = area.y + divider_y + 1;
            self.render_label(label, lx, ly, Color::Rgb(200, 180, 80), area, buf);
        }
    }

    fn render_label(
        &self,
        label: &str,
        lx: u16,
        ly: u16,
        fg: Color,
        area: Rect,
        buf: &mut Buffer,
    ) {
        if ly < area.y || ly >= area.y + area.height {
            return;
        }
        for (i, ch) in label.chars().enumerate() {
            let sx = lx + i as u16;
            if sx >= area.x + area.width {
                break;
            }
            if let Some(cell) = buf.cell_mut((sx, ly)) {
                cell.set_char(ch);
                cell.set_style(Style::default().fg(fg));
            }
        }
    }

    fn render_horizontal_walls(&self, floor: &Floor, area: Rect, buf: &mut Buffer) {
        // Top border: ┌─────┐
        let top_y = area.y;
        let bottom_y = area.y + floor.height as u16 - 1;
        let left_x = area.x;
        let right_x = area.x + floor.width as u16 - 1;

        let wall_style = Style::default().fg(sprites::WALL_COLOR).bg(Color::Reset);

        // Corners
        if let Some(c) = buf.cell_mut((left_x, top_y)) {
            c.set_char('┌');
            c.set_style(wall_style);
        }
        if let Some(c) = buf.cell_mut((right_x, top_y)) {
            c.set_char('┐');
            c.set_style(wall_style);
        }
        if let Some(c) = buf.cell_mut((left_x, bottom_y)) {
            c.set_char('└');
            c.set_style(wall_style);
        }
        if let Some(c) = buf.cell_mut((right_x, bottom_y)) {
            c.set_char('┘');
            c.set_style(wall_style);
        }

        // Top and bottom horizontal walls
        for gx in 1..floor.width as u16 - 1 {
            let sx = area.x + gx;
            if let Some(c) = buf.cell_mut((sx, top_y)) {
                c.set_char('─');
                c.set_style(wall_style);
            }
            if let Some(c) = buf.cell_mut((sx, bottom_y)) {
                c.set_char('─');
                c.set_style(wall_style);
            }
        }

        // Horizontal divider row between workspace and lower rooms
        let div_y = area.y + floor.workspace.3;
        let lounge_w = floor.lounge.2 as u16;

        // Left wall junction
        if let Some(c) = buf.cell_mut((area.x, div_y)) {
            c.set_char('├');
            c.set_style(wall_style);
        }
        // Right wall junction
        if let Some(c) = buf.cell_mut((right_x, div_y)) {
            c.set_char('┤');
            c.set_style(wall_style);
        }
        // Lounge/CEO divider junction on horizontal wall
        let div_jx = area.x + lounge_w;
        if div_jx < right_x {
            if let Some(c) = buf.cell_mut((div_jx, div_y)) {
                c.set_char('┬');
                c.set_style(wall_style);
            }
        }

        // Horizontal divider cells (non-wall/door cells become ─)
        for gx in 1..floor.width as u16 - 1 {
            let gy = floor.workspace.3 as usize;
            let sx = area.x + gx;
            let sy = div_y;
            if sy < area.y + area.height {
                let cell_type = floor.grid[gy][gx as usize];
                if cell_type == CellType::Wall {
                    if let Some(c) = buf.cell_mut((sx, sy)) {
                        c.set_char('─');
                        c.set_style(wall_style);
                    }
                }
            }
        }
    }

    fn render_agent(&self, agent: &crate::game::agent::Agent, area: Rect, buf: &mut Buffer) {
        let ax = agent.position.0 as u16;
        let ay = agent.position.1 as u16;

        let sprite = match agent.facing {
            Direction::Right => &sprites::AGENT_RIGHT,
            Direction::Left => &sprites::AGENT_LEFT,
        };

        let color = if agent.status == AgentStatus::Error {
            Color::Red
        } else {
            agent.sprite_color.to_color()
        };

        let style = Style::default().fg(color);

        // Top row: 2 cells wide
        let rows = [sprite.top, sprite.bottom];
        for (row_offset, row) in rows.iter().enumerate() {
            let sy = area.y + ay + row_offset as u16;
            if sy >= area.y + area.height {
                continue;
            }
            for (col_offset, ch_str) in row.iter().enumerate() {
                let sx = area.x + ax + col_offset as u16;
                if sx >= area.x + area.width {
                    continue;
                }
                if let Some(cell) = buf.cell_mut((sx, sy)) {
                    // Use the first char of the string (braille chars are single Unicode codepoints)
                    if let Some(ch) = ch_str.chars().next() {
                        cell.set_char(ch);
                        cell.set_style(style);
                    }
                }
            }
        }
    }

    fn render_ceo(&self, floor: &Floor, area: Rect, buf: &mut Buffer) {
        let (cx, cy) = floor.ceo_chair;
        let sprite = &sprites::CEO_SPRITE;
        let style = Style::default().fg(sprites::CEO_COLOR);

        let rows = [sprite.top, sprite.bottom];
        for (row_offset, row) in rows.iter().enumerate() {
            let sy = area.y + cy + row_offset as u16;
            if sy >= area.y + area.height {
                continue;
            }
            for (col_offset, ch_str) in row.iter().enumerate() {
                let sx = area.x + cx + col_offset as u16;
                if sx >= area.x + area.width {
                    continue;
                }
                if let Some(cell) = buf.cell_mut((sx, sy)) {
                    if let Some(ch) = ch_str.chars().next() {
                        cell.set_char(ch);
                        cell.set_style(style);
                    }
                }
            }
        }
    }
}
