use super::agent::Room;
use super::floor::Floor;

/// Compute a waypoint-based path from (from_x, from_y) in from_room to (to_x, to_y) in to_room.
///
/// Returns a Vec of (u16, u16) waypoints the agent should walk through in order.
pub fn compute_path(
    from_x: u16,
    from_y: u16,
    from_room: Room,
    to_room: Room,
    to_x: u16,
    to_y: u16,
    floor: &Floor,
) -> Vec<(u16, u16)> {
    if from_room == to_room {
        return vec![(to_x, to_y)];
    }

    match (from_room, to_room) {
        // Adjacent: Workspace <-> Lounge
        (Room::Workspace, Room::Lounge) => {
            if let Some(door) = floor.nearest_door(from_x, from_y, Room::Workspace, Room::Lounge) {
                let door_x = door.x;
                let door_y = door.y;
                // Walk to just above the door (workspace side), step through, walk to target
                vec![(door_x, door_y - 1), (door_x, door_y + 1), (to_x, to_y)]
            } else {
                vec![(to_x, to_y)]
            }
        }
        (Room::Lounge, Room::Workspace) => {
            if let Some(door) = floor.nearest_door(from_x, from_y, Room::Lounge, Room::Workspace) {
                let door_x = door.x;
                let door_y = door.y;
                // Walk to just below the door (lounge side), step through, walk to target
                vec![(door_x, door_y + 1), (door_x, door_y - 1), (to_x, to_y)]
            } else {
                vec![(to_x, to_y)]
            }
        }

        // Adjacent: Workspace <-> CeoOffice
        (Room::Workspace, Room::CeoOffice) => {
            if let Some(door) = floor.nearest_door(from_x, from_y, Room::Workspace, Room::CeoOffice)
            {
                let door_x = door.x;
                let door_y = door.y;
                vec![(door_x, door_y - 1), (door_x, door_y + 1), (to_x, to_y)]
            } else {
                vec![(to_x, to_y)]
            }
        }
        (Room::CeoOffice, Room::Workspace) => {
            if let Some(door) = floor.nearest_door(from_x, from_y, Room::CeoOffice, Room::Workspace)
            {
                let door_x = door.x;
                let door_y = door.y;
                vec![(door_x, door_y + 1), (door_x, door_y - 1), (to_x, to_y)]
            } else {
                vec![(to_x, to_y)]
            }
        }

        // Same room (already handled above, but required for exhaustiveness)
        (Room::Workspace, Room::Workspace)
        | (Room::Lounge, Room::Lounge)
        | (Room::CeoOffice, Room::CeoOffice) => vec![(to_x, to_y)],

        // Non-adjacent: Lounge <-> CeoOffice must go through Workspace
        (Room::Lounge, Room::CeoOffice) => {
            let workspace_center = floor.room_center(Room::Workspace);
            // Exit lounge -> workspace
            let exit_door = floor.nearest_door(from_x, from_y, Room::Lounge, Room::Workspace);
            // Enter workspace -> ceo office
            let enter_door = floor.nearest_door(
                workspace_center.0,
                workspace_center.1,
                Room::Workspace,
                Room::CeoOffice,
            );

            let mut path = Vec::new();
            if let Some(d) = exit_door {
                path.push((d.x, d.y + 1)); // lounge side
                path.push((d.x, d.y - 1)); // workspace side
            }
            path.push(workspace_center);
            if let Some(d) = enter_door {
                path.push((d.x, d.y - 1)); // workspace side
                path.push((d.x, d.y + 1)); // ceo side
            }
            path.push((to_x, to_y));
            path
        }
        (Room::CeoOffice, Room::Lounge) => {
            let workspace_center = floor.room_center(Room::Workspace);
            // Exit ceo -> workspace
            let exit_door = floor.nearest_door(from_x, from_y, Room::CeoOffice, Room::Workspace);
            // Enter workspace -> lounge
            let enter_door = floor.nearest_door(
                workspace_center.0,
                workspace_center.1,
                Room::Workspace,
                Room::Lounge,
            );

            let mut path = Vec::new();
            if let Some(d) = exit_door {
                path.push((d.x, d.y + 1)); // ceo side
                path.push((d.x, d.y - 1)); // workspace side
            }
            path.push(workspace_center);
            if let Some(d) = enter_door {
                path.push((d.x, d.y - 1)); // workspace side
                path.push((d.x, d.y + 1)); // lounge side
            }
            path.push((to_x, to_y));
            path
        }
    }
}

/// Advance an agent's position along the path by speed * delta_secs tiles.
///
/// Returns true if the agent is still animating (path non-empty), false if finished.
pub fn advance_along_path(
    position: &mut (f32, f32),
    path: &mut Vec<(u16, u16)>,
    speed: f32,
    delta_secs: f32,
) -> bool {
    if path.is_empty() {
        return false;
    }

    let mut remaining = speed * delta_secs;

    while remaining > 0.0 {
        if path.is_empty() {
            break;
        }

        let next = path[0];
        let nx = next.0 as f32;
        let ny = next.1 as f32;

        let dx = nx - position.0;
        let dy = ny - position.1;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist <= remaining {
            // Snap to waypoint and consume only the distance traveled
            remaining -= dist;
            position.0 = nx;
            position.1 = ny;
            path.remove(0);
        } else {
            // Move proportionally toward next waypoint
            let t = remaining / dist;
            position.0 += dx * t;
            position.1 += dy * t;
            remaining = 0.0;
        }
    }

    !path.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::floor::Floor;

    fn make_floor() -> Floor {
        Floor::generate(80, 30)
    }

    #[test]
    fn test_same_room_direct_path() {
        let floor = make_floor();
        let path = compute_path(5, 5, Room::Workspace, Room::Workspace, 10, 10, &floor);
        assert_eq!(path.len(), 1);
        assert_eq!(path[0], (10, 10));
    }

    #[test]
    fn test_workspace_to_lounge_goes_through_door() {
        let floor = make_floor();
        let (cx, cy) = floor.room_center(Room::Workspace);
        let (tx, ty) = floor.room_center(Room::Lounge);
        let path = compute_path(cx, cy, Room::Workspace, Room::Lounge, tx, ty, &floor);
        assert!(
            path.len() >= 3,
            "Expected at least 3 waypoints, got {}",
            path.len()
        );
    }

    #[test]
    fn test_lounge_to_ceo_goes_through_workspace() {
        let floor = make_floor();
        let (lx, ly) = floor.room_center(Room::Lounge);
        let (cx, cy) = floor.room_center(Room::CeoOffice);
        let path = compute_path(lx, ly, Room::Lounge, Room::CeoOffice, cx, cy, &floor);
        assert!(
            path.len() >= 5,
            "Expected at least 5 waypoints (non-adjacent route), got {}",
            path.len()
        );
    }

    #[test]
    fn test_advance_reaches_waypoint() {
        // speed=4, delta=3 -> step=12; target is 10 tiles away on x-axis -> should arrive
        let mut pos: (f32, f32) = (0.0, 0.0);
        let mut path: Vec<(u16, u16)> = vec![(10, 0)];
        let still_moving = advance_along_path(&mut pos, &mut path, 4.0, 3.0);
        assert!(
            !still_moving,
            "Path should be empty after reaching waypoint"
        );
        assert!(path.is_empty(), "Path should be drained");
        assert!(
            (pos.0 - 10.0).abs() < 0.001,
            "x should be ~10, got {}",
            pos.0
        );
        assert!((pos.1 - 0.0).abs() < 0.001, "y should be ~0, got {}", pos.1);
    }

    #[test]
    fn test_advance_partial_movement() {
        // speed=4, delta=1 -> step=4; target is 10 tiles away on x-axis -> partial move
        let mut pos: (f32, f32) = (0.0, 0.0);
        let mut path: Vec<(u16, u16)> = vec![(10, 0)];
        let still_moving = advance_along_path(&mut pos, &mut path, 4.0, 1.0);
        assert!(still_moving, "Should still be animating");
        assert!(!path.is_empty(), "Path should still have the waypoint");
        assert!(
            (pos.0 - 4.0).abs() < 0.001,
            "x should be ~4.0, got {}",
            pos.0
        );
    }

    #[test]
    fn test_advance_empty_path() {
        let mut pos: (f32, f32) = (5.0, 5.0);
        let mut path: Vec<(u16, u16)> = vec![];
        let still_moving = advance_along_path(&mut pos, &mut path, 4.0, 1.0);
        assert!(!still_moving);
        // Position should be unchanged
        assert!((pos.0 - 5.0).abs() < 0.001);
        assert!((pos.1 - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_advance_chain_waypoints() {
        // Two waypoints: (5,0) and (10,0); speed=10, delta=1 -> step=10; enough to reach both
        let mut pos: (f32, f32) = (0.0, 0.0);
        let mut path: Vec<(u16, u16)> = vec![(5, 0), (10, 0)];
        let still_moving = advance_along_path(&mut pos, &mut path, 10.0, 1.0);
        assert!(!still_moving, "Should have consumed both waypoints");
        assert!(path.is_empty());
        assert!(
            (pos.0 - 10.0).abs() < 0.001,
            "x should be ~10, got {}",
            pos.0
        );
    }
}
