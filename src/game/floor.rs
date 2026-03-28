use crate::game::agent::{Room, SpriteColor};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum CellType {
    Empty,
    Wall,
    Door,
    Desk,
    #[allow(dead_code)]
    Monitor,
    PingPongTable,
    PingPongNet,
    CeoDesk,
    CeoMonitor,
    Couch,
    CoffeeTable,
    VendingMachine,
    BulletinBoard,
}

pub const MIN_DESKS: usize = 0;
pub const DESK_HEIGHT: u16 = 3;
pub const DESK_SPACING_X: u16 = 9;
pub const DESK_SPACING_Y: u16 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeskVariant {
    Dual,
}

impl DeskVariant {
    pub fn width(self) -> u16 {
        match self {
            DeskVariant::Dual => 7,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeskSlot {
    pub desk_x: u16,
    pub desk_y: u16,
    pub chair_x: u16,
    pub chair_y: u16,
    pub occupied: bool,
    pub agent_color: Option<SpriteColor>,
    pub variant: DeskVariant,
}

#[derive(Debug, Clone)]
pub struct DoorPos {
    pub x: u16,
    pub y: u16,
    pub connects: [Room; 2],
}

#[derive(Debug, Clone)]
pub struct Floor {
    pub width: u16,
    pub height: u16,
    pub grid: Vec<Vec<CellType>>,
    pub workspace: (u16, u16, u16, u16),
    pub lounge: (u16, u16, u16, u16),
    pub ceo_office: (u16, u16, u16, u16),
    pub desks: Vec<DeskSlot>,
    pub doors: Vec<DoorPos>,
    pub ceo_chair: (u16, u16),
    #[allow(dead_code)]
    pub ping_pong: (u16, u16, u16, u16),
}

impl Floor {
    pub fn generate(width: u16, height: u16) -> Self {
        let workspace_h = (height as f32 * 0.65) as u16;
        let bottom_h = height - workspace_h;
        let lounge_w = (width as f32 * 0.75) as u16;
        let ceo_w = width - lounge_w;

        let mut grid = vec![vec![CellType::Empty; width as usize]; height as usize];

        // Draw outer border walls
        #[allow(clippy::needless_range_loop)]
        for x in 0..width as usize {
            grid[0][x] = CellType::Wall;
            grid[height as usize - 1][x] = CellType::Wall;
        }
        #[allow(clippy::needless_range_loop)]
        for y in 0..height as usize {
            grid[y][0] = CellType::Wall;
            grid[y][width as usize - 1] = CellType::Wall;
        }

        // Horizontal divider at workspace_h
        #[allow(clippy::needless_range_loop)]
        for x in 0..width as usize {
            grid[workspace_h as usize][x] = CellType::Wall;
        }

        // Vertical divider at lounge_w (only in bottom section)
        #[allow(clippy::needless_range_loop)]
        for y in workspace_h as usize..height as usize {
            grid[y][lounge_w as usize] = CellType::Wall;
        }

        // Doors on the horizontal divider (8 cells wide each)
        let door_w: usize = 8;
        let lounge_left_door_x: u16 = 2;
        for i in 0..door_w {
            grid[workspace_h as usize][lounge_left_door_x as usize + i] = CellType::Door;
        }

        let lounge_right_door_x: u16 = lounge_w - door_w as u16 - 1;
        for i in 0..door_w {
            grid[workspace_h as usize][lounge_right_door_x as usize + i] = CellType::Door;
        }

        let ceo_door_x: u16 = lounge_w + ceo_w / 2 - door_w as u16 / 2;
        for i in 0..door_w {
            grid[workspace_h as usize][ceo_door_x as usize + i] = CellType::Door;
        }

        let doors = vec![
            DoorPos {
                x: lounge_left_door_x,
                y: workspace_h,
                connects: [Room::Workspace, Room::Lounge],
            },
            DoorPos {
                x: lounge_right_door_x,
                y: workspace_h,
                connects: [Room::Workspace, Room::Lounge],
            },
            DoorPos {
                x: ceo_door_x,
                y: workspace_h,
                connects: [Room::Workspace, Room::CeoOffice],
            },
        ];

        // Desks start empty — ensure_minimum_desks() adds the initial rows
        let desks = Vec::new();

        // Ping pong table (6x2) centered in lounge
        let lounge_center_x = lounge_w / 2;
        let lounge_center_y = workspace_h + bottom_h / 2;
        let pp_w: u16 = 6;
        let pp_h: u16 = 2;
        let pp_x = lounge_center_x.saturating_sub(pp_w / 2);
        let pp_y = lounge_center_y.saturating_sub(pp_h / 2);
        for py in pp_y..pp_y + pp_h {
            for px in pp_x..pp_x + pp_w {
                if (py as usize) < height as usize && (px as usize) < width as usize {
                    // Net runs vertically through the center of the table
                    if px == pp_x + pp_w / 2 {
                        grid[py as usize][px as usize] = CellType::PingPongNet;
                    } else {
                        grid[py as usize][px as usize] = CellType::PingPongTable;
                    }
                }
            }
        }
        let ping_pong = (pp_x, pp_y, pp_w, pp_h);

        // Lounge furniture: couches and coffee table
        // Couch 1: 6×1, left side of lounge
        let couch1_x = 2u16;
        let couch1_y = workspace_h + 3;
        for cx in couch1_x..couch1_x + 6 {
            if (couch1_y as usize) < height as usize && (cx as usize) < lounge_w as usize {
                grid[couch1_y as usize][cx as usize] = CellType::Couch;
            }
        }

        // Couch 2: 6×1, right side of lounge
        let couch2_x = lounge_w.saturating_sub(9);
        let couch2_y = workspace_h + bottom_h - 3;
        for cx in couch2_x..couch2_x + 6 {
            if (couch2_y as usize) < height as usize && (cx as usize) < lounge_w as usize {
                grid[couch2_y as usize][cx as usize] = CellType::Couch;
            }
        }

        // Coffee table: 3×1, center of lounge
        let ct_x = lounge_w / 2 - 1;
        let ct_y = workspace_h + bottom_h / 2 + 2;
        for cx in ct_x..ct_x + 3 {
            if (ct_y as usize) < height as usize && (cx as usize) < lounge_w as usize {
                grid[ct_y as usize][cx as usize] = CellType::CoffeeTable;
            }
        }

        // Vending machine: 2×2, top-right corner of lounge
        let vm_x = lounge_w - 4;
        let vm_y = workspace_h + 2;
        for vy in vm_y..vm_y + 2 {
            for vx in vm_x..vm_x + 2 {
                if (vy as usize) < height as usize && (vx as usize) < lounge_w as usize {
                    grid[vy as usize][vx as usize] = CellType::VendingMachine;
                }
            }
        }

        // CEO desk + monitor in CEO office center
        let ceo_center_x = lounge_w + ceo_w / 2;
        let ceo_center_y = workspace_h + bottom_h / 2;
        if (ceo_center_y as usize) < height as usize && (ceo_center_x as usize) < width as usize {
            grid[ceo_center_y as usize][ceo_center_x as usize] = CellType::CeoDesk;
        }
        if (ceo_center_y as usize) < height as usize
            && (ceo_center_x as usize + 1) < width as usize
        {
            grid[ceo_center_y as usize][ceo_center_x as usize + 1] = CellType::CeoMonitor;
        }
        let ceo_chair = (ceo_center_x, ceo_center_y + 1);

        // Bulletin board: 4×2, on the right wall of CEO office
        let bb_x = lounge_w + ceo_w - 6;
        let bb_y = workspace_h + 2;
        for by in bb_y..bb_y + 2 {
            for bx in bb_x..bb_x + 4 {
                if (by as usize) < height as usize && (bx as usize) < width as usize {
                    grid[by as usize][bx as usize] = CellType::BulletinBoard;
                }
            }
        }

        let workspace = (0, 0, width, workspace_h);
        let lounge = (0, workspace_h, lounge_w, bottom_h);
        let ceo_office = (lounge_w, workspace_h, ceo_w, bottom_h);

        Floor {
            width,
            height,
            grid,
            workspace,
            lounge,
            ceo_office,
            desks,
            doors,
            ceo_chair,
            ping_pong,
        }
    }

    pub fn room_center(&self, room: Room) -> (u16, u16) {
        match room {
            Room::Workspace => {
                let (x, y, w, h) = self.workspace;
                (x + w / 2, y + h / 2)
            }
            Room::Lounge => {
                let (x, y, w, h) = self.lounge;
                (x + w / 2, y + h / 2)
            }
            Room::CeoOffice => {
                let (x, y, w, h) = self.ceo_office;
                (x + w / 2, y + h / 2)
            }
        }
    }

    pub fn assign_desk(&mut self) -> Option<usize> {
        for (i, desk) in self.desks.iter_mut().enumerate() {
            if !desk.occupied {
                desk.occupied = true;
                return Some(i);
            }
        }
        // All occupied — add one desk and relayout the grid evenly
        let new_count = self.desks.len() + 1;
        self.relayout_desks(new_count);
        let idx = self.desks.len() - 1;
        self.desks[idx].occupied = true;
        Some(idx)
    }

    pub fn free_desk(&mut self, index: usize) {
        if let Some(desk) = self.desks.get_mut(index) {
            desk.occupied = false;
            desk.agent_color = None;
        }
    }

    pub fn ensure_minimum_desks(&mut self) {
        if self.desks.len() < MIN_DESKS {
            self.relayout_desks(MIN_DESKS);
        }
    }

    /// Clear all desk cells from the grid, then re-place `count` desks
    /// in an evenly distributed centered grid using ceil(sqrt(n)) columns.
    fn relayout_desks(&mut self, count: usize) {
        // Clear old desk cells from grid
        for desk in &self.desks {
            let w = desk.variant.width();
            for row in 0..DESK_HEIGHT {
                for col in 0..w {
                    let gy = (desk.desk_y + row) as usize;
                    let gx = (desk.desk_x + col) as usize;
                    if gy < self.height as usize && gx < self.width as usize {
                        self.grid[gy][gx] = CellType::Empty;
                    }
                }
            }
        }

        // Preserve occupied state and agent_color for existing desks
        let old_states: Vec<(bool, Option<SpriteColor>)> = self.desks
            .iter()
            .map(|d| (d.occupied, d.agent_color))
            .collect();

        // Calculate grid dimensions: ceil(sqrt(n)) columns
        let cols = (count as f32).sqrt().ceil() as u16;
        let rows = ((count as f32) / cols as f32).ceil() as u16;

        // Center horizontally
        let usable_w = self.width.saturating_sub(2);
        let total_w = cols * DESK_SPACING_X;
        let start_x = 1 + usable_w.saturating_sub(total_w) / 2;

        // Center vertically in workspace
        let total_h = rows * DESK_SPACING_Y;

        // Grow workspace if needed
        while total_h + 2 > self.workspace.3.saturating_sub(2) {
            self.grow_workspace(DESK_SPACING_Y);
        }
        let workspace_inner = self.workspace.3.saturating_sub(2);
        let start_y = 1 + workspace_inner.saturating_sub(total_h) / 2;

        // Place desks
        self.desks.clear();
        let variant = DeskVariant::Dual;
        let w = variant.width();

        for i in 0..count {
            let col = i as u16 % cols;
            let row = i as u16 / cols;
            let dx = start_x + col * DESK_SPACING_X;
            let dy = start_y + row * DESK_SPACING_Y;

            // Mark grid cells
            for r in 0..DESK_HEIGHT {
                for c in 0..w {
                    let gy = (dy + r) as usize;
                    let gx = (dx + c) as usize;
                    if gy < self.height as usize && gx < self.width as usize {
                        self.grid[gy][gx] = CellType::Desk;
                    }
                }
            }

            // Restore state from old desk at this index if it existed
            let (occupied, agent_color) = old_states.get(i).copied().unwrap_or((false, None));

            self.desks.push(DeskSlot {
                desk_x: dx,
                desk_y: dy,
                chair_x: dx + w / 2,
                chair_y: dy + DESK_HEIGHT,
                occupied,
                agent_color,
                variant,
            });
        }
    }

    fn grow_workspace(&mut self, extra_rows: u16) {
        let insert_y = self.workspace.3 as usize;

        for _ in 0..extra_rows {
            self.grid.insert(insert_y, vec![CellType::Empty; self.width as usize]);
        }

        self.height += extra_rows;
        self.workspace.3 += extra_rows;
        self.lounge.1 += extra_rows;
        self.ceo_office.1 += extra_rows;

        for door in &mut self.doors {
            if door.y >= insert_y as u16 {
                door.y += extra_rows;
            }
        }

        let new_div_y = self.workspace.3 as usize;
        for x in 0..self.width as usize {
            if new_div_y < self.grid.len() {
                self.grid[new_div_y][x] = CellType::Wall;
            }
        }
        for x in 0..self.width as usize {
            if self.grid[insert_y][x] == CellType::Wall {
                self.grid[insert_y][x] = CellType::Empty;
            }
        }

        for door in &mut self.doors {
            let dx = door.x as usize;
            let dy = door.y as usize;
            if dy < self.grid.len() {
                for i in 0..8 {
                    if dx + i < self.width as usize {
                        self.grid[dy][dx + i] = CellType::Door;
                    }
                }
            }
        }

        for y in insert_y..insert_y + extra_rows as usize {
            if y < self.grid.len() {
                self.grid[y][0] = CellType::Wall;
                self.grid[y][self.width as usize - 1] = CellType::Wall;
            }
        }

        self.ceo_chair.1 += extra_rows;
        self.ping_pong.1 += extra_rows;

        let pp = self.ping_pong;
        for py in pp.1..pp.1 + pp.3 {
            for px in pp.0..pp.0 + pp.2 {
                if (py as usize) < self.height as usize && (px as usize) < self.width as usize {
                    self.grid[py as usize][px as usize] = CellType::PingPongTable;
                }
            }
        }

        let (cx, cy) = self.ceo_chair;
        let ceo_desk_y = cy.saturating_sub(1);
        if (ceo_desk_y as usize) < self.height as usize && (cx as usize) < self.width as usize {
            self.grid[ceo_desk_y as usize][cx as usize] = CellType::CeoDesk;
        }
        if (ceo_desk_y as usize) < self.height as usize && (cx as usize + 1) < self.width as usize {
            self.grid[ceo_desk_y as usize][cx as usize + 1] = CellType::CeoMonitor;
        }

        let bot = self.height as usize - 1;
        for x in 0..self.width as usize {
            self.grid[bot][x] = CellType::Wall;
        }
    }

    pub fn nearest_door(
        &self,
        x: u16,
        y: u16,
        from_room: Room,
        to_room: Room,
    ) -> Option<&DoorPos> {
        self.doors
            .iter()
            .filter(|d| {
                (d.connects[0] == from_room && d.connects[1] == to_room)
                    || (d.connects[0] == to_room && d.connects[1] == from_room)
            })
            .min_by_key(|d| {
                let dx = (d.x as i32 - x as i32).abs();
                let dy = (d.y as i32 - y as i32).abs();
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
        assert_eq!(floor.grid.len(), 30);
        assert_eq!(floor.grid[0].len(), 80);
        assert_eq!(floor.width, 80);
        assert_eq!(floor.height, 30);
    }

    #[test]
    fn test_floor_has_three_doors() {
        let floor = Floor::generate(80, 30);
        assert_eq!(floor.doors.len(), 3);
    }

    #[test]
    fn test_floor_has_desks() {
        let mut floor = Floor::generate(80, 30);
        floor.ensure_minimum_desks();
        assert!(floor.desks.len() >= MIN_DESKS);
    }

    #[test]
    fn test_desk_assignment() {
        let mut floor = Floor::generate(80, 30);
        let first = floor.assign_desk();
        let second = floor.assign_desk();
        assert_eq!(first, Some(0));
        assert_eq!(second, Some(1));
    }

    #[test]
    fn test_desk_free_and_reassign() {
        let mut floor = Floor::generate(80, 30);
        let first = floor.assign_desk();
        assert_eq!(first, Some(0));
        floor.free_desk(0);
        let reassigned = floor.assign_desk();
        assert_eq!(reassigned, Some(0));
    }

    #[test]
    fn test_workspace_proportions() {
        let floor = Floor::generate(80, 30);
        // 65% of 30 = 19 (floor of 19.5)
        assert_eq!(floor.workspace.3, 19);
    }

    #[test]
    fn test_lounge_width_proportion() {
        let floor = Floor::generate(80, 30);
        // 75% of 80 = 60
        assert_eq!(floor.lounge.2, 60);
    }
}
