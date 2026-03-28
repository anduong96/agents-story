/// Advance an agent along its path by `speed` units per second.
/// Returns `true` if the path was fully consumed (agent arrived).
pub fn advance_along_path(
    position: &mut (f32, f32),
    path: &mut Vec<(u16, u16)>,
    speed: f32,
    delta_secs: f32,
) -> bool {
    let mut remaining = speed * delta_secs;

    while remaining > 0.0 {
        let next = match path.first() {
            Some(&p) => p,
            None => return true, // path exhausted
        };

        let target = (next.0 as f32, next.1 as f32);
        let dx = target.0 - position.0;
        let dy = target.1 - position.1;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist <= remaining {
            // Step fully onto this node
            *position = target;
            path.remove(0);
            remaining -= dist;
        } else {
            // Step partway
            let t = remaining / dist;
            position.0 += dx * t;
            position.1 += dy * t;
            remaining = 0.0;
        }
    }

    path.is_empty()
}
