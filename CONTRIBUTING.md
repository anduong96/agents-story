# Contributing

Thanks for wanting to make this slightly less pointless.

## Getting Started

```bash
git clone https://github.com/anduong96/agents-story.git
cd agents-story
cargo build
cargo run -- --demo --fast
```

## Development

```bash
./dev.sh    # hot reload with cargo-watch
```

## Before Submitting

```bash
cargo build            # no errors
cargo test             # all pass
cargo clippy -- -D warnings  # no lint warnings
cargo fmt -- --check   # formatted
```

No unused code. If you add something, use it. If you remove something, clean up the references.

## Adding Furniture

Most visual additions follow this pattern:

1. Add a `CellType` variant in `src/game/floor.rs`
2. Place it in `Floor::generate()`
3. Add a color constant in `src/ui/sprites.rs`
4. Add a match arm in `src/ui/floor_view.rs`

## Adding Agent Behavior

Agent logic lives in `src/app.rs` (tick loop) and `src/main.rs` (event handling). Pathfinding is waypoint-based in `src/game/pathfinding.rs`.

## Code Style

- Rust stable, no nightly features
- No `unsafe`
- Keep files focused — don't put rendering logic in game files or vice versa
- Comments only where the logic isn't obvious

## Issues

Found a bug? Agent walking through a wall? Desks disappearing? Open an issue. Screenshots help.

## Ideas Welcome

The roadmap is short and the bar is low. If you want to add a water cooler, a bathroom, or a conference room — go for it.
