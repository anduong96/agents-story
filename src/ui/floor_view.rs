use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::game::agent::AgentStatus;
use crate::game::floor::{CellType, DeskVariant, Floor};
use crate::game::state::GameState;
use crate::ui::sprites;

pub struct FloorView<'a> {
    pub state: &'a GameState,
    pub tick: u64,
    pub scroll_y: u16,
    pub ceo_pos: (f32, f32),
}

impl<'a> FloorView<'a> {
    pub fn new(state: &'a GameState) -> Self {
        let ceo_pos = (
            state.floor.ceo_chair.0 as f32,
            state.floor.ceo_chair.1 as f32,
        );
        FloorView {
            state,
            tick: 0,
            scroll_y: 0,
            ceo_pos,
        }
    }

    pub fn with_tick(mut self, tick: u64) -> Self {
        self.tick = tick;
        self
    }

    pub fn with_scroll(mut self, scroll_y: u16) -> Self {
        self.scroll_y = scroll_y;
        self
    }

    pub fn with_ceo_pos(mut self, pos: (f32, f32)) -> Self {
        self.ceo_pos = pos;
        self
    }

    /// Convert grid Y to screen Y, returning None if off-screen.
    fn grid_to_screen(&self, gy: u16, area: &Rect) -> Option<u16> {
        if gy < self.scroll_y {
            return None;
        }
        let sy = area.y + gy - self.scroll_y;
        if sy >= area.y + area.height {
            None
        } else {
            Some(sy)
        }
    }
}

/// Returns (top_color, bottom_color) for a half-block screen pixel.
/// Each cell shows TWO colors via ▀ (fg=top, bg=bottom), doubling visual detail.
fn screen_pixel_colors(
    desk_x: u16,
    desk_y: u16,
    col: usize,
    occupied: bool,
    tick: u64,
) -> (Color, Color) {
    if !occupied {
        let off = sprites::DESK_SCREEN_OFF_COLOR;
        let dim = sprites::DESK_SCREEN_DIM_COLOR;
        return if (desk_x as usize + col).is_multiple_of(2) {
            (off, dim)
        } else {
            (dim, off)
        };
    }
    // Animate: shift colors every ~20 ticks, each desk at its own phase
    let phase = (tick / 20) as usize + desk_x as usize * 3 + desk_y as usize * 7;
    let len = sprites::SCREEN_PIXELS.len();
    let h1 = (phase + col * 31) % len;
    let h2 = (phase + col * 17 + 3) % len;
    (sprites::SCREEN_PIXELS[h1], sprites::SCREEN_PIXELS[h2])
}

/// Returns (char, fg, bg) for textured floor at the given grid position.
fn floor_texture(floor: &Floor, gx: usize, gy: usize) -> (char, Color, Color) {
    let workspace_h = floor.workspace.3 as usize;
    let lounge_w = floor.lounge.2 as usize;

    if gy < workspace_h {
        // Brick pattern: 6 wide × 2 tall, offset every other row
        let tile_w = 6;
        let tile_h = 2;
        let row_group = gy / tile_h;
        let offset = if row_group.is_multiple_of(2) {
            0
        } else {
            tile_w / 2
        };
        let tile_idx = ((gx + offset) / tile_w + row_group) % 2;
        if tile_idx == 0 {
            (
                ' ',
                sprites::WORKSPACE_FLOOR_BG_EVEN,
                sprites::WORKSPACE_FLOOR_BG_EVEN,
            )
        } else {
            (
                ' ',
                sprites::WORKSPACE_FLOOR_BG_ALT,
                sprites::WORKSPACE_FLOOR_BG_ALT,
            )
        }
    } else if gx < lounge_w {
        ('▩', sprites::LOUNGE_FLOOR_FG, sprites::LOUNGE_FLOOR_BG_EVEN)
    } else {
        if gy.is_multiple_of(2) {
            (
                sprites::CEO_FLOOR_CHAR_EVEN,
                sprites::CEO_FLOOR_FG_EVEN,
                sprites::CEO_FLOOR_BG_EVEN,
            )
        } else {
            (
                sprites::CEO_FLOOR_CHAR_ODD,
                sprites::CEO_FLOOR_BG_ODD,
                sprites::CEO_FLOOR_BG_ODD,
            )
        }
    }
}

fn floor_bg(floor: &Floor, gx: usize, gy: usize) -> Color {
    let (_, _, bg) = floor_texture(floor, gx, gy);
    bg
}

impl<'a> Widget for FloorView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let floor = &self.state.floor;
        let grid_w = floor.width as usize;
        let grid_h = floor.height as usize;
        let scroll = self.scroll_y as usize;

        // Single pass: render grid cells with scroll offset
        for gy in scroll..grid_h {
            let sy = area.y + (gy - scroll) as u16;
            if sy >= area.y + area.height {
                break;
            }

            for gx in 0..grid_w {
                let sx = area.x + gx as u16;
                if sx >= area.x + area.width {
                    continue;
                }

                let cell_type = floor.grid[gy][gx];
                let bg = floor_bg(floor, gx, gy);

                let (ch, fg, actual_bg) = match cell_type {
                    CellType::Wall => ('│', sprites::WALL_COLOR, Color::Reset),
                    CellType::Door => ('▒', sprites::DOOR_COLOR, Color::Reset),
                    CellType::Desk
                    | CellType::Monitor
                    | CellType::CeoDesk
                    | CellType::CeoMonitor => floor_texture(floor, gx, gy),
                    CellType::PingPongTable => ('▒', sprites::PING_PONG_COLOR, bg),
                    CellType::PingPongNet => {
                        ('│', sprites::PING_PONG_NET_COLOR, sprites::PING_PONG_COLOR)
                    }
                    CellType::Arcade => {
                        // Check if an idle agent is nearby (within 2 cells)
                        let in_use = self.state.agents.iter().any(|a| {
                            a.status == AgentStatus::Idle
                                && !a.is_animating()
                                && (a.position.0 as i32 - gx as i32).abs() <= 2
                                && (a.position.1 as i32 - gy as i32).abs() <= 2
                        });
                        let above_is_arcade = gy > 0 && floor.grid[gy - 1][gx] == CellType::Arcade;
                        let is_top = !above_is_arcade;
                        if is_top && in_use {
                            // Top row: animated screen when in use
                            let len = sprites::ARCADE_SCREEN_COLORS.len();
                            let phase = ((self.tick / 8) as usize + gx * 5) % len;
                            (
                                '█',
                                sprites::ARCADE_SCREEN_COLORS[phase],
                                sprites::ARCADE_CABINET_COLOR,
                            )
                        } else {
                            // Off or bottom row: dark cabinet
                            (
                                '█',
                                sprites::ARCADE_CABINET_COLOR,
                                sprites::ARCADE_TRIM_COLOR,
                            )
                        }
                    }
                    CellType::Bookshelf => {
                        let book_idx = (gx * 3 + gy * 5) % 4;
                        (
                            '▐',
                            sprites::BOOKSHELF_BOOK_COLORS[book_idx],
                            sprites::BOOKSHELF_COLOR,
                        )
                    }
                    CellType::Plant => ('♣', sprites::PLANT_COLOR, sprites::PLANT_POT_COLOR),
                    CellType::TreeSmall => {
                        ('▲', sprites::TREE_SMALL_COLOR, sprites::TREE_SMALL_TRUNK)
                    }
                    CellType::Whiteboard => {
                        ('░', sprites::WHITEBOARD_FRAME, sprites::WHITEBOARD_COLOR)
                    }
                    CellType::Bush => ('▓', sprites::BUSH_COLOR, sprites::BUSH_BG_COLOR),
                    CellType::TreeLarge => {
                        ('♠', sprites::TREE_LARGE_COLOR, sprites::TREE_LARGE_TRUNK)
                    }
                    CellType::BulletinBoard => {
                        let pin1 = (gx * 3 + gy * 7) % 4;
                        let pin2 = (gx * 5 + gy * 11 + 1) % 4;
                        (
                            '▀',
                            sprites::BULLETIN_PIN_COLORS[pin1],
                            sprites::BULLETIN_PIN_COLORS[pin2],
                        )
                    }
                    CellType::WaterCooler => {
                        ('▐', sprites::WATER_COOLER_COLOR, sprites::WATER_COOLER_BODY)
                    }
                    CellType::Empty => floor_texture(floor, gx, gy),
                };

                if let Some(cell) = buf.cell_mut((sx, sy)) {
                    cell.set_char(ch);
                    cell.set_style(Style::default().fg(fg).bg(actual_bg));
                }
            }
        }

        // Overlays (all scroll-aware)
        self.render_room_labels(floor, area, buf);
        self.render_horizontal_walls(floor, area, buf);
        self.render_desks(floor, area, buf);

        for agent in &self.state.agents {
            self.render_agent(agent, area, buf);
        }
        self.render_ceo(floor, area, buf);

        // Scroll indicators
        if self.scroll_y > 0 {
            let mid_x = area.x + area.width / 2;
            if let Some(cell) = buf.cell_mut((mid_x, area.y)) {
                cell.set_char('▲');
                cell.set_style(Style::default().fg(Color::Rgb(180, 180, 180)));
            }
        }
        if (floor.height as usize) > scroll + area.height as usize {
            let mid_x = area.x + area.width / 2;
            let bot = area.y + area.height - 1;
            if let Some(cell) = buf.cell_mut((mid_x, bot)) {
                cell.set_char('▼');
                cell.set_style(Style::default().fg(Color::Rgb(180, 180, 180)));
            }
        }
    }
}

impl<'a> FloorView<'a> {
    fn render_room_labels(&self, floor: &Floor, area: Rect, buf: &mut Buffer) {
        let divider_y = floor.workspace.3;
        let lounge_w = floor.lounge.2;

        if divider_y > 1 {
            if let Some(sy) = self.grid_to_screen(divider_y - 1, &area) {
                let label = "[ Workspace ]";
                let lx = area.x + floor.workspace.2 / 4;
                self.render_label_at(label, lx, sy, Color::Rgb(150, 150, 180), area, buf);
            }
        }

        if let Some(sy) = self.grid_to_screen(divider_y + 1, &area) {
            let label = "[ Lounge ]";
            let lx = area.x + lounge_w / 4;
            self.render_label_at(label, lx, sy, Color::Rgb(180, 140, 120), area, buf);

            let label = "[ CEO ]";
            let ceo_x = lounge_w + (floor.ceo_office.2 / 4);
            let lx = area.x + ceo_x;
            self.render_label_at(label, lx, sy, Color::Rgb(200, 180, 80), area, buf);
        }
    }

    fn render_label_at(
        &self,
        label: &str,
        lx: u16,
        sy: u16,
        fg: Color,
        area: Rect,
        buf: &mut Buffer,
    ) {
        for (i, ch) in label.chars().enumerate() {
            let sx = lx + i as u16;
            if sx >= area.x + area.width {
                break;
            }
            if let Some(cell) = buf.cell_mut((sx, sy)) {
                cell.set_char(ch);
                cell.set_style(Style::default().fg(fg));
            }
        }
    }

    fn render_horizontal_walls(&self, floor: &Floor, area: Rect, buf: &mut Buffer) {
        let left_x = area.x;
        let right_x = area.x + floor.width - 1;
        let wall_style = Style::default().fg(sprites::WALL_COLOR).bg(Color::Reset);
        let div_style = Style::default()
            .fg(sprites::VERTICAL_DIVIDER_COLOR)
            .bg(Color::Reset);
        let lounge_w = floor.lounge.2;

        // Top border
        if let Some(sy) = self.grid_to_screen(0, &area) {
            if let Some(c) = buf.cell_mut((left_x, sy)) {
                c.set_char('┌');
                c.set_style(wall_style);
            }
            if let Some(c) = buf.cell_mut((right_x, sy)) {
                c.set_char('┐');
                c.set_style(wall_style);
            }
            for gx in 1..floor.width - 1 {
                if let Some(c) = buf.cell_mut((area.x + gx, sy)) {
                    c.set_char('─');
                    c.set_style(wall_style);
                }
            }
        }

        // Bottom border
        if let Some(sy) = self.grid_to_screen(floor.height - 1, &area) {
            if let Some(c) = buf.cell_mut((left_x, sy)) {
                c.set_char('└');
                c.set_style(wall_style);
            }
            if let Some(c) = buf.cell_mut((right_x, sy)) {
                c.set_char('┘');
                c.set_style(wall_style);
            }
            for gx in 1..floor.width - 1 {
                if let Some(c) = buf.cell_mut((area.x + gx, sy)) {
                    c.set_char('─');
                    c.set_style(wall_style);
                }
            }
        }

        // Horizontal divider
        if let Some(sy) = self.grid_to_screen(floor.workspace.3, &area) {
            if let Some(c) = buf.cell_mut((left_x, sy)) {
                c.set_char('├');
                c.set_style(wall_style);
            }
            if let Some(c) = buf.cell_mut((right_x, sy)) {
                c.set_char('┤');
                c.set_style(wall_style);
            }
            let div_jx = area.x + lounge_w;
            if div_jx < right_x {
                if let Some(c) = buf.cell_mut((div_jx, sy)) {
                    c.set_char('┬');
                    c.set_style(wall_style);
                }
            }
            for gx in 1..floor.width - 1 {
                let cell_type = floor.grid[floor.workspace.3 as usize][gx as usize];
                if cell_type == CellType::Wall {
                    if let Some(c) = buf.cell_mut((area.x + gx, sy)) {
                        c.set_char('─');
                        c.set_style(wall_style);
                    }
                }
            }
        }

        // Vertical divider between lounge and CEO
        let lounge_wall_x = area.x + lounge_w;
        for gy in (floor.workspace.3 + 1)..floor.height - 1 {
            if let Some(sy) = self.grid_to_screen(gy, &area) {
                if lounge_wall_x < area.x + area.width {
                    if let Some(c) = buf.cell_mut((lounge_wall_x, sy)) {
                        c.set_char('│');
                        c.set_style(div_style);
                    }
                }
            }
        }
        if let Some(sy) = self.grid_to_screen(floor.height - 1, &area) {
            if let Some(c) = buf.cell_mut((lounge_wall_x, sy)) {
                c.set_char('┴');
                c.set_style(div_style);
            }
        }
    }

    fn render_desks(&self, floor: &Floor, area: Rect, buf: &mut Buffer) {
        // Workspace desks
        for (desk_idx, desk) in floor.desks.iter().enumerate() {
            let agent_seated = self
                .state
                .agents
                .iter()
                .any(|a| a.assigned_desk == Some(desk_idx) && !a.is_animating());
            self.render_single_desk(desk, agent_seated, false, area, buf);
        }
        // CEO desk (always on)
        if let Some(ref desk) = floor.ceo_desk {
            self.render_single_desk(desk, true, true, area, buf);
        }
    }

    fn render_single_desk(
        &self,
        desk: &crate::game::floor::DeskSlot,
        screen_on: bool,
        is_ceo_desk: bool,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let (row0, row1, row2, screen_cols): (&[char], &[char], &[char], &[usize]) =
            match desk.variant {
                DeskVariant::Single => (
                    &sprites::DESK1_ROW0,
                    &sprites::DESK1_ROW1,
                    &sprites::DESK1_ROW2,
                    sprites::DESK1_SCREEN_COLS,
                ),
                DeskVariant::Dual => (
                    &sprites::DESK2_ROW0,
                    &sprites::DESK2_ROW1,
                    &sprites::DESK2_ROW2,
                    sprites::DESK2_SCREEN_COLS,
                ),
                DeskVariant::Triple => (
                    &sprites::DESK3_ROW0,
                    &sprites::DESK3_ROW1,
                    &sprites::DESK3_ROW2,
                    sprites::DESK3_SCREEN_COLS,
                ),
            };

        let rows: [(&[char], u16); 3] = [(row0, 0), (row1, 1), (row2, 2)];
        for &(row, row_off) in &rows {
            let gy = desk.desk_y + row_off;
            if let Some(sy) = self.grid_to_screen(gy, &area) {
                for (col, &ch) in row.iter().enumerate() {
                    let sx = area.x + desk.desk_x + col as u16;
                    if sx >= area.x + area.width {
                        continue;
                    }

                    if row_off == 1 && screen_cols.contains(&col) {
                        let (top, bottom) = screen_pixel_colors(
                            desk.desk_x,
                            desk.desk_y,
                            col,
                            screen_on,
                            self.tick,
                        );
                        if let Some(cell) = buf.cell_mut((sx, sy)) {
                            cell.set_char('▀');
                            cell.set_style(Style::default().fg(top).bg(bottom));
                        }
                    } else {
                        if let Some(cell) = buf.cell_mut((sx, sy)) {
                            cell.set_char(ch);
                            let surface = if is_ceo_desk {
                                sprites::CEO_DESK_SURFACE_COLOR
                            } else {
                                sprites::DESK_SURFACE_COLOR
                            };
                            cell.set_style(
                                Style::default()
                                    .fg(sprites::DESK_FRAME_COLOR)
                                    .bg(surface),
                            );
                        }
                    }
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

        // 8-bit style: 2×2 solid blocks
        // Row 0: ██  head (unique skin tone per agent)
        // Row 1: ██  body (agent color)
        let skin = agent.sprite_color.skin_color();
        if ay < self.scroll_y {
            return;
        }
        let sy0 = area.y + ay - self.scroll_y;
        if sy0 < area.y + area.height {
            for col in 0..2u16 {
                let sx = area.x + ax + col;
                if sx < area.x + area.width {
                    if let Some(cell) = buf.cell_mut((sx, sy0)) {
                        cell.set_char('█');
                        cell.set_style(Style::default().fg(skin));
                    }
                }
            }
        }
        let sy1 = area.y + ay + 1 - self.scroll_y;
        if sy1 < area.y + area.height {
            for col in 0..2u16 {
                let sx = area.x + ax + col;
                if sx < area.x + area.width {
                    if let Some(cell) = buf.cell_mut((sx, sy1)) {
                        cell.set_char('█');
                        cell.set_style(Style::default().fg(color));
                    }
                }
            }
        }
    }

    fn render_ceo(&self, _floor: &Floor, area: Rect, buf: &mut Buffer) {
        let cx = self.ceo_pos.0 as u16;
        let cy = self.ceo_pos.1 as u16;

        // 8-bit CEO
        if cy < self.scroll_y {
            return;
        }
        let sy0 = area.y + cy - self.scroll_y;
        if sy0 < area.y + area.height {
            for col in 0..2u16 {
                let sx = area.x + cx + col;
                if sx < area.x + area.width {
                    if let Some(cell) = buf.cell_mut((sx, sy0)) {
                        cell.set_char('█');
                        cell.set_style(Style::default().fg(sprites::CEO_SKIN_COLOR));
                    }
                }
            }
        }
        if cy + 1 < self.scroll_y {
            return;
        }
        let sy1 = area.y + cy + 1 - self.scroll_y;
        if sy1 < area.y + area.height {
            for col in 0..2u16 {
                let sx = area.x + cx + col;
                if sx < area.x + area.width {
                    if let Some(cell) = buf.cell_mut((sx, sy1)) {
                        cell.set_char('█');
                        cell.set_style(Style::default().fg(sprites::CEO_OUTFIT_COLOR));
                    }
                }
            }
        }
    }
}
