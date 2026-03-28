use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::game::agent::{AgentStatus, Room};
use crate::game::floor::{CellType, DeskVariant, Floor};
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

/// Returns a textured floor cell with slight color variation for Kairosoft-style warmth.
/// Returns (char, fg, bg) where the bg has subtle per-tile variation.
fn textured_floor_cell(floor: &Floor, x: usize, y: usize) -> (char, Color, Color) {
    let base_bg = floor_bg(floor, x, y);
    // Add subtle checkerboard texture
    let offset = if (x + y) % 2 == 0 { 5i16 } else { -3i16 };
    let bg = match base_bg {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as i16 + offset).clamp(0, 255) as u8,
            (g as i16 + offset).clamp(0, 255) as u8,
            (b as i16 + offset).clamp(0, 255) as u8,
        ),
        other => other,
    };
    (' ', Color::Reset, bg)
}

/// Returns (top_color, bottom_color) for a half-block screen pixel.
/// Each cell shows TWO colors via ▀ (fg=top, bg=bottom), doubling visual detail.
fn screen_pixel_colors(desk_x: u16, desk_y: u16, col: usize, occupied: bool) -> (Color, Color) {
    if !occupied {
        let off = sprites::DESK_SCREEN_OFF_COLOR;
        // Slight variation even when off
        let dim = Color::Rgb(25, 25, 30);
        return if (desk_x as usize + col) % 2 == 0 { (off, dim) } else { (dim, off) };
    }
    let len = sprites::SCREEN_PIXELS.len();
    let h1 = (desk_x as usize * 7 + desk_y as usize * 13 + col * 31) % len;
    let h2 = (desk_x as usize * 11 + desk_y as usize * 23 + col * 17 + 3) % len;
    (sprites::SCREEN_PIXELS[h1], sprites::SCREEN_PIXELS[h2])
}

/// Returns (char, fg, bg) for textured floor at the given grid position.
fn floor_texture(floor: &Floor, gx: usize, gy: usize) -> (char, Color, Color) {
    let workspace_h = floor.workspace.3 as usize;
    let lounge_w = floor.lounge.2 as usize;

    if gy < workspace_h {
        if gy % 2 == 0 {
            (sprites::WORKSPACE_FLOOR_CHAR_EVEN, sprites::WORKSPACE_FLOOR_FG_EVEN, sprites::WORKSPACE_FLOOR_BG_EVEN)
        } else {
            (sprites::WORKSPACE_FLOOR_CHAR_ODD, sprites::WORKSPACE_FLOOR_BG_ODD, sprites::WORKSPACE_FLOOR_BG_ODD)
        }
    } else if gx < lounge_w {
        if gy % 2 == 0 {
            (sprites::LOUNGE_FLOOR_CHAR_EVEN, sprites::LOUNGE_FLOOR_FG_EVEN, sprites::LOUNGE_FLOOR_BG_EVEN)
        } else {
            (sprites::LOUNGE_FLOOR_CHAR_ODD, sprites::LOUNGE_FLOOR_BG_ODD, sprites::LOUNGE_FLOOR_BG_ODD)
        }
    } else {
        if gy % 2 == 0 {
            (sprites::CEO_FLOOR_CHAR_EVEN, sprites::CEO_FLOOR_FG_EVEN, sprites::CEO_FLOOR_BG_EVEN)
        } else {
            (sprites::CEO_FLOOR_CHAR_ODD, sprites::CEO_FLOOR_BG_ODD, sprites::CEO_FLOOR_BG_ODD)
        }
    }
}

fn floor_bg(floor: &Floor, gx: usize, gy: usize) -> Color {
    let (_, _, bg) = floor_texture(floor, gx, gy);
    bg
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
                let bg = floor_bg(floor, gx, gy);

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

                let bg = floor_bg(floor, gx, gy);
                let (ch, fg, actual_bg) = match cell_type {
                    CellType::Wall => ('│', sprites::WALL_COLOR, Color::Reset),
                    CellType::Door => ('▒', sprites::DOOR_COLOR, Color::Reset),
                    CellType::Desk | CellType::Monitor => {
                        let (tc, tf, tb) = floor_texture(floor, gx, gy);
                        (tc, tf, tb)
                    }
                    CellType::CeoDesk => ('▄', Color::Rgb(180, 120, 60), floor_bg(floor, gx, gy)),
                    CellType::CeoMonitor => {
                        let monitor_fg = if (self.tick % 60) < 30 {
                            sprites::MONITOR_COLOR
                        } else {
                            sprites::MONITOR_FLICKER_COLOR
                        };
                        ('▓', monitor_fg, floor_bg(floor, gx, gy))
                    }
                    CellType::PingPongTable => ('▒', sprites::PING_PONG_COLOR, bg),
                    CellType::Couch => ('█', sprites::COUCH_COLOR, bg),
                    CellType::CoffeeTable => ('▬', sprites::COFFEE_TABLE_COLOR, bg),
                    CellType::VendingMachine => ('▐', sprites::VENDING_MACHINE_COLOR, bg),
                    CellType::BulletinBoard => ('▒', sprites::BULLETIN_BOARD_COLOR, bg),
                    CellType::Empty => {
                        let (tc, tf, tb) = floor_texture(floor, gx, gy);
                        (tc, tf, tb)
                    }
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

        // Render detailed desk sprites with multi-color screens
        self.render_desks(floor, area, buf);

        // Render lounge furniture with detail overlays
        self.render_lounge_furniture(floor, area, buf);

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
        let divider_y = floor.workspace.3;
        let lounge_w = floor.lounge.2;

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
        let bottom_y = area.y + floor.height - 1;
        let left_x = area.x;
        let right_x = area.x + floor.width - 1;

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
        for gx in 1..floor.width - 1 {
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
        let lounge_w = floor.lounge.2;

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
        for gx in 1..floor.width - 1 {
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

        // Vertical divider between lounge and CEO (below the main divider)
        let lounge_wall_x = area.x + lounge_w;
        let div_style = Style::default().fg(Color::Rgb(80, 80, 120)).bg(Color::Reset);
        for gy in (floor.workspace.3 + 1)..floor.height - 1 {
            let sy = area.y + gy;
            if sy < area.y + area.height && lounge_wall_x < area.x + area.width {
                if let Some(c) = buf.cell_mut((lounge_wall_x, sy)) {
                    c.set_char('│');
                    c.set_style(div_style);
                }
            }
        }
        // Bottom junction
        if let Some(c) = buf.cell_mut((lounge_wall_x, area.y + floor.height - 1)) {
            c.set_char('┴');
            c.set_style(div_style);
        }
    }

    fn render_desks(&self, floor: &Floor, area: Rect, buf: &mut Buffer) {
        for desk in &floor.desks {
            let (row0, row1, row2, screen_cols): (&[char], &[char], &[char], &[usize]) = match desk.variant {
                DeskVariant::Single => (&sprites::DESK1_ROW0, &sprites::DESK1_ROW1, &sprites::DESK1_ROW2, sprites::DESK1_SCREEN_COLS),
                DeskVariant::Dual   => (&sprites::DESK2_ROW0, &sprites::DESK2_ROW1, &sprites::DESK2_ROW2, sprites::DESK2_SCREEN_COLS),
                DeskVariant::Triple => (&sprites::DESK3_ROW0, &sprites::DESK3_ROW1, &sprites::DESK3_ROW2, sprites::DESK3_SCREEN_COLS),
            };

            let rows: [(&[char], u16); 3] = [(row0, 0), (row1, 1), (row2, 2)];
            for &(row, row_off) in &rows {
                let sy = area.y + desk.desk_y + row_off;
                if sy >= area.y + area.height { continue; }
                for (col, &ch) in row.iter().enumerate() {
                    let sx = area.x + desk.desk_x + col as u16;
                    if sx >= area.x + area.width { continue; }

                    if row_off == 1 && screen_cols.contains(&col) {
                        let (top, bottom) = screen_pixel_colors(desk.desk_x, desk.desk_y, col, desk.occupied);
                        if let Some(cell) = buf.cell_mut((sx, sy)) {
                            cell.set_char('▀');
                            cell.set_style(Style::default().fg(top).bg(bottom));
                        }
                    } else {
                        if let Some(cell) = buf.cell_mut((sx, sy)) {
                            cell.set_char(ch);
                            cell.set_style(Style::default().fg(sprites::DESK_FRAME_COLOR).bg(sprites::DESK_SURFACE_COLOR));
                        }
                    }
                }
            }
        }
    }

    fn render_lounge_furniture(&self, floor: &Floor, area: Rect, buf: &mut Buffer) {
        let grid_h = floor.height as usize;
        let grid_w = floor.width as usize;

        for gy in 0..grid_h {
            for gx in 0..grid_w {
                let cell_type = floor.grid[gy][gx];
                let sx = area.x + gx as u16;
                let sy = area.y + gy as u16;
                if sx >= area.x + area.width || sy >= area.y + area.height {
                    continue;
                }

                match cell_type {
                    CellType::Couch => {
                        // Half-block couch: top=cushion, bottom=frame
                        if let Some(cell) = buf.cell_mut((sx, sy)) {
                            cell.set_char('▀');
                            cell.set_style(
                                Style::default()
                                    .fg(sprites::COUCH_COLOR)
                                    .bg(sprites::COUCH_FRAME_COLOR),
                            );
                        }
                    }
                    CellType::VendingMachine => {
                        // Half-block vending: top=body, bottom=light strip
                        if let Some(cell) = buf.cell_mut((sx, sy)) {
                            cell.set_char('▀');
                            cell.set_style(
                                Style::default()
                                    .fg(sprites::VENDING_MACHINE_COLOR)
                                    .bg(sprites::VENDING_LIGHT_COLOR),
                            );
                        }
                    }
                    CellType::BulletinBoard => {
                        // Half-block bulletin: two different pin colors
                        let pin1 = (gx * 3 + gy * 7) % 4;
                        let pin2 = (gx * 5 + gy * 11 + 1) % 4;
                        if let Some(cell) = buf.cell_mut((sx, sy)) {
                            cell.set_char('▀');
                            cell.set_style(
                                Style::default()
                                    .fg(sprites::BULLETIN_PIN_COLORS[pin1])
                                    .bg(sprites::BULLETIN_PIN_COLORS[pin2]),
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn render_agent(&self, agent: &crate::game::agent::Agent, area: Rect, buf: &mut Buffer) {
        let ax = agent.position.0 as u16;
        let ay = agent.position.1 as u16;

        let color = if agent.status == AgentStatus::Error {
            Color::Red
        } else {
            agent.sprite_color.to_color()
        };

        // Render agent as a 2-char name tag with colored background
        let tag: String = agent.name.chars().take(2).collect::<String>().to_uppercase();
        let style = Style::default().fg(color).bg(sprites::NAME_TAG_BG);

        let sy = area.y + ay;
        if sy >= area.y + area.height { return; }

        for (i, ch) in tag.chars().enumerate() {
            let sx = area.x + ax + i as u16;
            if sx >= area.x + area.width { continue; }
            if let Some(cell) = buf.cell_mut((sx, sy)) {
                cell.set_char(ch);
                cell.set_style(style);
            }
        }
    }

    fn render_ceo(&self, _floor: &Floor, area: Rect, buf: &mut Buffer) {
        let (cx, cy) = self.state.floor.ceo_chair;
        let style = Style::default().fg(sprites::CEO_COLOR).bg(sprites::NAME_TAG_BG);
        let tag = "CEO";

        let sy = area.y + cy;
        if sy >= area.y + area.height { return; }

        for (i, ch) in tag.chars().enumerate() {
            let sx = area.x + cx + i as u16;
            if sx >= area.x + area.width { continue; }
            if let Some(cell) = buf.cell_mut((sx, sy)) {
                cell.set_char(ch);
                cell.set_style(style);
            }
        }
    }
}
