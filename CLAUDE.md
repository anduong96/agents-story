# Agents Story

A Rust TUI application that visualizes Claude Code agent sessions as a pixel-art office.

## Architecture

- **`src/game/`** — Game logic layer
  - `floor.rs` — Floor grid with rooms (Workspace, Lounge, CEO Office), dynamic desk management with `DeskVariant` (Single/Dual/Triple monitors), furniture placement
  - `agent.rs` — Agent model with status machine (Spawning→Working→Finished/Error/Idle), position, pathfinding, sprite colors, skin tones
  - `state.rs` — `GameState` holding floor, agents vec, aggregated stats
  - `pathfinding.rs` — Waypoint pathfinding between rooms through doors

- **`src/ui/`** — Rendering layer (ratatui widgets)
  - `floor_view.rs` — Main rendering: textured floors, half-block desk sprites with animated multi-color screens, scroll support, furniture, room labels, agents
  - `sprites.rs` — All color constants, desk sprite rows, decoration colors, status indicators
  - `agent_panel.rs` — Side panel listing agents with color-coded status indicators
  - `stats_bar.rs` — Bottom stats bar (model, agents, tokens, cost, FPS, RAM, hotkeys)
  - `bubbles.rs` — Lightweight status indicator system (single-char symbols)

- **`src/`** — App layer
  - `main.rs` — Event loop, stream message handling, 6 permanent staff agents, temp agent hire/fire
  - `app.rs` — App state, tick loop, collision avoidance, lounge wandering
  - `demo.rs` — Demo mode (`--demo`) with 6 staff + 3 temp agents
  - `input.rs` — Keyboard and mouse input handling
  - `stream/` — Protocol for streaming Claude Code session events

## Key Patterns

- **6 permanent staff**: Idle in lounge at startup, assigned to tasks on demand, return to lounge when done.
- **Temp agents**: Hired when all staff are busy, leave permanently when task completes.
- **Desks are on-demand**: Created when agents need them, freed when agents leave. Grid uses `ceil(sqrt(n))` columns for even distribution.
- **CEO desk is separate**: Stored in `floor.ceo_desk`, not in the workspace `desks` Vec.
- **Half-block rendering**: Monitor screens use `▀` with fg=top pixel, bg=bottom pixel for 2x vertical color detail. Screens animate with shifting colors.
- **Collision avoidance**: Agents pause when their next position would overlap another agent (2×2 bounding box check).
- **Scrollable workspace**: When desks overflow the view, mouse scroll navigates. `▲`/`▼` indicators show when content extends beyond view.
- **Adaptive FPS**: 15fps when animating, 2fps when idle.

## Running

```bash
cargo run -- --demo              # Demo mode (2x speed)
cargo run -- --demo --fast       # 5x speed
cargo run -- --demo --extreme    # 10x speed
./demo.sh                        # Hot reload demo with cargo-watch
```

## Code Quality

- **Always clean up dead code** — run `cargo build` and fix all warnings before committing. No unused constants, functions, variables, or imports.
- **Run `cargo test`** before committing to ensure nothing is broken.

## Testing

```bash
cargo test
```
