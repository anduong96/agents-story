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
    TV,
    CeoDesk,
    CeoMonitor,
    Couch,
    CoffeeTable,
    VendingMachine,
    BulletinBoard,
    Plant,
    Bookshelf,
}

pub const MIN_DESKS: usize = 0;
pub const DESK_HEIGHT: u16 = 3;
pub const DESK_SPACING_X: u16 = 12;  // accommodate widest (3 monitors = 10)
pub const DESK_SPACING_Y: u16 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeskVariant {
    Single,  // 1 monitor, 4 wide
    Dual,    // 2 monitors, 7 wide
    Triple,  // 3 monitors, 10 wide
}

impl DeskVariant {
    pub fn width(self) -> u16 {
        10 // all desks same width
    }

    /// Pick variant from agent name (deterministic)
    pub fn from_name(name: &str) -> Self {
        let hash: u32 = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        match hash % 3 {
            0 => DeskVariant::Single,
            1 => DeskVariant::Dual,
            _ => DeskVariant::Triple,
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
        let mut desks = Vec::new();

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

        // Large TV: 10×2, centered on top wall of lounge
        let tv_w: u16 = 10;
        let tv_h: u16 = 2;
        let tv_x = lounge_w / 2 - tv_w / 2;
        let tv_y = workspace_h + 2;
        for ty in tv_y..tv_y + tv_h {
            for tx in tv_x..tv_x + tv_w {
                if (ty as usize) < height as usize && (tx as usize) < lounge_w as usize {
                    grid[ty as usize][tx as usize] = CellType::TV;
                }
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

        // CEO desk: 10 wide × 3 tall (same as workspace desks), centered in CEO office
        let ceo_desk_w: u16 = 10;
        let ceo_desk_x = lounge_w + (ceo_w.saturating_sub(ceo_desk_w)) / 2;
        let ceo_desk_y = workspace_h + bottom_h / 2 - 1;
        for row in 0..DESK_HEIGHT {
            for col in 0..ceo_desk_w {
                let gy = (ceo_desk_y + row) as usize;
                let gx = (ceo_desk_x + col) as usize;
                if gy < height as usize && gx < width as usize {
                    grid[gy][gx] = CellType::Desk;
                }
            }
        }
        // Add as a DeskSlot with Single variant (1 monitor)
        desks.push(DeskSlot {
            desk_x: ceo_desk_x,
            desk_y: ceo_desk_y,
            chair_x: ceo_desk_x + ceo_desk_w / 2,
            chair_y: ceo_desk_y + DESK_HEIGHT,
            occupied: true,  // CEO always at desk
            agent_color: None,
            variant: DeskVariant::Single,
        });
        let ceo_chair = (ceo_desk_x + ceo_desk_w / 2, ceo_desk_y + DESK_HEIGHT);

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

        // Bookshelf: 6×1, along the bottom wall of CEO office
        let bs_x = lounge_w + 2;
        let bs_y = workspace_h + bottom_h - 2;
        for bx in bs_x..bs_x + 6 {
            if (bs_y as usize) < height as usize && (bx as usize) < width as usize {
                grid[bs_y as usize][bx as usize] = CellType::Bookshelf;
            }
        }

        // Plants/trees in every room corners
        let plant_positions = [
            // Workspace corners
            (2u16, 1u16),
            (width - 3, 1),
            (2, workspace_h - 2),
            (width - 3, workspace_h - 2),
            // Lounge corners
            (2, workspace_h + 2),
            (lounge_w - 3, workspace_h + bottom_h - 2),
            // CEO office corners
            (lounge_w + 2, workspace_h + 2),
            (width - 3, workspace_h + bottom_h - 2),
        ];
        for (px, py) in plant_positions {
            if (py as usize) < height as usize && (px as usize) < width as usize {
                if grid[py as usize][px as usize] == CellType::Empty {
                    grid[py as usize][px as usize] = CellType::Plant;
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

    pub fn assign_desk(&mut self, agent_name: &str) -> Option<usize> {
        for (i, desk) in self.desks.iter_mut().enumerate() {
            if !desk.occupied {
                desk.occupied = true;
                return Some(i);
            }
        }
        // All occupied — add one desk and relayout the grid evenly
        let variant = DeskVariant::from_name(agent_name);
        let new_count = self.desks.len() + 1;
        self.relayout_desks(new_count, Some(variant));
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
            self.relayout_desks(MIN_DESKS, None);
        }
    }

    /// Clear all desk cells from the grid, then re-place `count` desks
    /// in an evenly distributed centered grid using ceil(sqrt(n)) columns.
    /// If `new_variant` is Some, the last (new) desk uses that variant.
    fn relayout_desks(&mut self, count: usize, new_variant: Option<DeskVariant>) {
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

        // Preserve existing desk state
        let old_desks: Vec<(bool, Option<SpriteColor>, DeskVariant)> = self.desks
            .iter()
            .map(|d| (d.occupied, d.agent_color, d.variant))
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

        for i in 0..count {
            // Reuse old variant, or use new_variant for the last desk, or default to Dual
            let variant = if let Some(&(_, _, v)) = old_desks.get(i) {
                v
            } else {
                new_variant.unwrap_or(DeskVariant::Dual)
            };
            let w = variant.width();

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

            let (occupied, agent_color) = old_desks.get(i)
                .map(|&(o, ac, _)| (o, ac))
                .unwrap_or((false, None));

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
        // Index 0 is the CEO desk (occupied), so first assign gets index 1
        let first = floor.assign_desk("test");
        let second = floor.assign_desk("test2");
        assert!(first.is_some());
        assert!(second.is_some());
        assert_ne!(first, second);
    }

    #[test]
    fn test_desk_free_and_reassign() {
        let mut floor = Floor::generate(80, 30);
        let first = floor.assign_desk("test");
        assert!(first.is_some());
        let idx = first.unwrap();
        floor.free_desk(idx);
        let reassigned = floor.assign_desk("test");
        assert_eq!(reassigned, Some(idx));
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
