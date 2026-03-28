# Agents Story

A Rust TUI application that visualizes Claude Code agent sessions as a pixel-art Kairosoft-style office.

## Architecture

- **`src/game/`** — Game logic layer
  - `floor.rs` — Floor grid with rooms (Workspace, Lounge, CEO Office), dynamic desk management with `DeskVariant` (Single/Dual/Triple monitors), furniture placement, workspace growth
  - `agent.rs` — Agent model with status machine (Spawning→Working→Finished/Error/Idle), position, pathfinding, sprite colors
  - `state.rs` — `GameState` holding floor, agents vec, aggregated stats
  - `pathfinding.rs` — A* pathfinding between rooms through doors

- **`src/ui/`** — Rendering layer (ratatui widgets)
  - `floor_view.rs` — Main rendering: textured floors, half-block desk sprites with multi-color screens, furniture overlays, room labels, agents, CEO
  - `sprites.rs` — All sprite definitions, color constants (Kairosoft palette), desk sprite rows
  - `agent_panel.rs` — Side panel listing agents with status/details
  - `stats_bar.rs` — Bottom stats bar (model, agents, tokens, cost)
  - `bubbles.rs` — Speech bubble system for agent status/tool updates

- **`src/`** — App layer
  - `main.rs` — Event loop, stream message handling, agent spawn/transition wiring
  - `app.rs` — App state (GameState + UI state + tick loop)
  - `demo.rs` — Demo mode (`--demo`) with synthetic agent events
  - `input.rs` — Keyboard input handling
  - `stream/` — Protocol for streaming Claude Code session events

## Key Patterns

- **Desks are dynamic**: Start empty, `ensure_minimum_desks()` adds rows (min 4). When all occupied, `assign_desk()` auto-grows via `add_desk_row()` → `grow_workspace()`.
- **Half-block rendering**: Monitor screens use `▀` with fg=top pixel, bg=bottom pixel for 2x vertical color detail.
- **Sprites as overlays**: Desk/furniture visuals are rendered as overlays on top of the floor texture grid. The grid uses `CellType::Desk` for pathfinding obstacles only.
- **Adaptive FPS**: 15fps when animating, 2fps when idle.

## Running

```bash
cargo run -- --demo    # Demo mode with synthetic agents
./demo.sh              # Hot reload demo (installs cargo-watch)
```

## Code Quality

- **Always clean up dead code** — run `cargo build` and fix all warnings before committing. No unused constants, functions, variables, or imports.
- **Run `cargo test`** before committing to ensure nothing is broken.

## Testing

```bash
cargo test
```
