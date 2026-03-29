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
    #[allow(dead_code)]
    CeoDesk,
    #[allow(dead_code)]
    CeoMonitor,
    Couch,
    CoffeeTable,
    VendingMachine,
    BulletinBoard,
    Plant,      // small potted plant ♣
    TreeSmall,  // small tree ▲
    TreeLarge,  // large tree ♠
    Chair,      // lunch table chair
    Bookshelf,
}

pub const MIN_DESKS: usize = 0;
pub const DESK_HEIGHT: u16 = 3;
pub const DESK_SPACING_X: u16 = 12;  // accommodate widest (3 monitors = 10)
pub const DESK_SPACING_Y: u16 = 5; // desk(3) + agent(2), no gap

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
    pub ceo_desk: Option<DeskSlot>,
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

        // === LOUNGE LAYOUT: 3 zones ===
        let lounge_cx = lounge_w / 2;  // horizontal center of lounge
        let lounge_top = workspace_h + 2;
        let lounge_bot = workspace_h + bottom_h - 2;
        let lounge_mid = workspace_h + bottom_h / 2;

        // --- Zone 1: TV + Sofa area (top-left) ---
        // TV on wall
        let tv_w: u16 = 8;
        let tv_x = 4u16;
        for tx in tv_x..tv_x + tv_w {
            if (lounge_top as usize) < height as usize && (tx as usize) < lounge_w as usize {
                grid[lounge_top as usize][tx as usize] = CellType::TV;
            }
        }
        // Two sofas facing the TV
        let sofa_y1 = lounge_top + 2;
        let sofa_y2 = lounge_top + 4;
        for sx in tv_x..tv_x + 6 {
            if (sofa_y1 as usize) < height as usize && (sx as usize) < lounge_w as usize {
                grid[sofa_y1 as usize][sx as usize] = CellType::Couch;
            }
            if (sofa_y2 as usize) < height as usize && (sx as usize) < lounge_w as usize {
                grid[sofa_y2 as usize][sx as usize] = CellType::Couch;
            }
        }
        // Coffee table between sofas
        for sx in (tv_x + 1)..(tv_x + 5) {
            if (sofa_y1 + 1) < height && (sx as usize) < lounge_w as usize {
                grid[(sofa_y1 + 1) as usize][sx as usize] = CellType::CoffeeTable;
            }
        }

        // --- Zone 2: Ping pong (center) ---
        let pp_w: u16 = 6;
        let pp_h: u16 = 2;
        let pp_x = lounge_cx.saturating_sub(pp_w / 2);
        let pp_y = lounge_mid.saturating_sub(pp_h / 2);
        for py in pp_y..pp_y + pp_h {
            for px in pp_x..pp_x + pp_w {
                if (py as usize) < height as usize && (px as usize) < lounge_w as usize {
                    if px == pp_x + pp_w / 2 {
                        grid[py as usize][px as usize] = CellType::PingPongNet;
                    } else {
                        grid[py as usize][px as usize] = CellType::PingPongTable;
                    }
                }
            }
        }
        let ping_pong = (pp_x, pp_y, pp_w, pp_h);

        // --- Zone 3: Lunch area (bottom-right of lounge) ---
        let lunch_x = lounge_w / 2;
        let lunch_y = lounge_bot - 3;
        let lunch_w: u16 = 6;
        // Lunch table
        for lx in lunch_x..lunch_x + lunch_w {
            if (lunch_y as usize) < height as usize && (lx as usize) < lounge_w as usize {
                grid[lunch_y as usize][lx as usize] = CellType::CoffeeTable;
            }
        }
        // Chairs above table
        for lx in lunch_x..lunch_x + lunch_w {
            if (lunch_y - 1) >= workspace_h && ((lunch_y - 1) as usize) < height as usize && (lx as usize) < lounge_w as usize {
                grid[(lunch_y - 1) as usize][lx as usize] = CellType::Chair;
            }
        }
        // Chairs below table
        for lx in lunch_x..lunch_x + lunch_w {
            if ((lunch_y + 1) as usize) < height as usize && (lx as usize) < lounge_w as usize {
                grid[(lunch_y + 1) as usize][lx as usize] = CellType::Chair;
            }
        }

        // Vending machine near lunch area
        let vm_x = lounge_w - 4;
        let vm_y = lounge_bot - 2;
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
        let ceo_desk = Some(DeskSlot {
            desk_x: ceo_desk_x,
            desk_y: ceo_desk_y,
            chair_x: ceo_desk_x + (ceo_desk_w - 2) / 2,
            chair_y: ceo_desk_y + DESK_HEIGHT,
            occupied: true,
            agent_color: None,
            variant: DeskVariant::Single,
        });
        let ceo_chair = (ceo_desk_x + (ceo_desk_w - 2) / 2, ceo_desk_y + DESK_HEIGHT);

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

        // Plants and trees — (x, y, type)
        let decorations: Vec<(u16, u16, CellType)> = vec![
            // Workspace — potted plants in corners, small trees along walls
            (2, 1, CellType::Plant),
            (width - 3, 1, CellType::TreeSmall),
            (2, workspace_h - 2, CellType::TreeSmall),
            (width - 3, workspace_h - 2, CellType::Plant),
            (width / 3, 1, CellType::TreeLarge),
            (width * 2 / 3, 1, CellType::TreeLarge),
            // Lounge — mix of trees and plants
            (2, workspace_h + 2, CellType::TreeLarge),
            (lounge_w - 3, workspace_h + 2, CellType::TreeSmall),
            (2, workspace_h + bottom_h - 2, CellType::Plant),
            (lounge_w - 3, workspace_h + bottom_h - 2, CellType::TreeLarge),
            (lounge_w / 3, workspace_h + bottom_h - 2, CellType::Plant),
            // CEO office — elegant plants
            (lounge_w + 2, workspace_h + 2, CellType::TreeSmall),
            (width - 3, workspace_h + 2, CellType::Plant),
            (width - 3, workspace_h + bottom_h - 2, CellType::TreeSmall),
        ];
        for (px, py, cell) in decorations {
            if (py as usize) < height as usize && (px as usize) < width as usize {
                if grid[py as usize][px as usize] == CellType::Empty {
                    grid[py as usize][px as usize] = cell;
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
            ceo_desk,
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
    pub fn relayout_desks(&mut self, count: usize, new_variant: Option<DeskVariant>) {
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

        // Center vertically in workspace (never grow — protect lounge height)
        let total_h = rows * DESK_SPACING_Y;
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
                chair_x: dx + (w - 2) / 2,  // center 2-wide agent in 10-wide desk
                chair_y: dy + DESK_HEIGHT - 1, // head overlaps desk bottom row
                occupied,
                agent_color,
                variant,
            });
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

    #[test]
    fn test_chair_is_below_desk() {
        let mut floor = Floor::generate(80, 30);
        let idx = floor.assign_desk("test").unwrap();
        let desk = &floor.desks[idx];
        // Chair Y: head overlaps desk bottom row
        assert_eq!(desk.chair_y, desk.desk_y + DESK_HEIGHT - 1,
            "Agent head (chair_y={}) should be at desk_y({}) + DESK_HEIGHT({}) - 1",
            desk.chair_y, desk.desk_y, DESK_HEIGHT);
    }

    #[test]
    fn test_chair_centered_horizontally() {
        let mut floor = Floor::generate(80, 30);
        let idx = floor.assign_desk("test").unwrap();
        let desk = &floor.desks[idx];
        let w = desk.variant.width();
        // Agent is 2 cells wide, should be centered in desk
        let expected_x = desk.desk_x + (w - 2) / 2;
        assert_eq!(desk.chair_x, expected_x,
            "Agent left edge (chair_x={}) should be at desk_x({}) + ({}-2)/2 = {}",
            desk.chair_x, desk.desk_x, w, expected_x);
    }

    #[test]
    fn test_agent_fits_between_desk_rows() {
        let mut floor = Floor::generate(80, 30);
        floor.assign_desk("a1").unwrap();
        floor.assign_desk("a2").unwrap();
        floor.assign_desk("a3").unwrap();
        floor.assign_desk("a4").unwrap();
        floor.assign_desk("a5").unwrap();

        // For desks in different rows, verify agent doesn't overlap next desk
        for (i, desk) in floor.desks.iter().enumerate() {
            let agent_bottom = desk.chair_y + 2; // agent is 2 rows tall
            // Find next desk in same column
            for other in floor.desks.iter().skip(i + 1) {
                if other.desk_x == desk.desk_x && other.desk_y > desk.desk_y {
                    assert!(agent_bottom <= other.desk_y,
                        "Agent at desk {} (bottom={}) overlaps desk at y={}",
                        i, agent_bottom, other.desk_y);
                    break;
                }
            }
        }
    }

    #[test]
    fn test_all_desks_consistent_chair_position() {
        let mut floor = Floor::generate(80, 30);
        for i in 0..6 {
            floor.assign_desk(&format!("agent-{}", i)).unwrap();
        }
        // Every desk (except CEO at index 0) should have consistent positioning
        for desk in &floor.desks {
            let w = desk.variant.width();
            assert_eq!(desk.chair_y, desk.desk_y + DESK_HEIGHT - 1,
                "Desk at ({},{}) has chair_y={}, expected {}",
                desk.desk_x, desk.desk_y, desk.chair_y, desk.desk_y + DESK_HEIGHT - 1);
            assert_eq!(desk.chair_x, desk.desk_x + (w - 2) / 2,
                "Desk at ({},{}) has chair_x={}, expected {}",
                desk.desk_x, desk.desk_y, desk.chair_x, desk.desk_x + (w - 2) / 2);
        }
    }
}
