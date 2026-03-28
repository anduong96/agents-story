use crate::game::agent::Room;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum CellType {
    Empty,
    Wall,
    Door,
    Desk,
    Monitor,
    PingPongTable,
    CeoDesk,
    CeoMonitor,
}

#[derive(Debug, Clone)]
pub struct DeskSlot {
    #[allow(dead_code)]
    pub desk_x: u16,
    #[allow(dead_code)]
    pub desk_y: u16,
    pub chair_x: u16,
    pub chair_y: u16,
    pub occupied: bool,
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

        // Doors on the horizontal divider
        // lounge-left door at x=2 (2 cells wide)
        let lounge_left_door_x: u16 = 2;
        grid[workspace_h as usize][lounge_left_door_x as usize] = CellType::Door;
        grid[workspace_h as usize][lounge_left_door_x as usize + 1] = CellType::Door;

        // lounge-right door at x=lounge_w-3 (2 cells wide)
        let lounge_right_door_x: u16 = lounge_w - 3;
        grid[workspace_h as usize][lounge_right_door_x as usize] = CellType::Door;
        grid[workspace_h as usize][lounge_right_door_x as usize + 1] = CellType::Door;

        // CEO door at x=lounge_w + ceo_w/2 (2 cells wide)
        let ceo_door_x: u16 = lounge_w + ceo_w / 2;
        // CEO door is on the vertical divider row — place on horizontal divider at ceo_door_x
        grid[workspace_h as usize][ceo_door_x as usize] = CellType::Door;
        grid[workspace_h as usize][ceo_door_x as usize + 1] = CellType::Door;

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

        // Place desks in workspace grid: start (3,2), spacing 6x horizontal, 3y vertical
        let mut desks = Vec::new();
        let desk_start_x: u16 = 3;
        let desk_start_y: u16 = 2;
        let desk_spacing_x: u16 = 6;
        let desk_spacing_y: u16 = 3;

        let mut dy = desk_start_y;
        while dy + 1 < workspace_h - 1 {
            let mut dx = desk_start_x;
            while dx + 1 < width - 1 {
                grid[dy as usize][dx as usize] = CellType::Desk;
                grid[dy as usize][(dx + 1) as usize] = CellType::Monitor;
                desks.push(DeskSlot {
                    desk_x: dx,
                    desk_y: dy,
                    chair_x: dx,
                    chair_y: dy + 1,
                    occupied: false,
                });
                dx += desk_spacing_x;
            }
            dy += desk_spacing_y;
        }

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
                    grid[py as usize][px as usize] = CellType::PingPongTable;
                }
            }
        }
        let ping_pong = (pp_x, pp_y, pp_w, pp_h);

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
        // All full — return index 0 as fallback
        if !self.desks.is_empty() {
            Some(0)
        } else {
            None
        }
    }

    pub fn free_desk(&mut self, index: usize) {
        if let Some(desk) = self.desks.get_mut(index) {
            desk.occupied = false;
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
        let floor = Floor::generate(80, 30);
        assert!(!floor.desks.is_empty());
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
