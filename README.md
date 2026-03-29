# Agents Story

> Because watching a progress bar wasn't unproductive enough.

A completely unnecessary pixel-art TUI that lets you watch your Claude Code agents pretend to work in a tiny office. They walk to desks. They sit down. Their monitors flicker with colorful pixels. They accomplish absolutely nothing visual — but hey, at least you burned some extra tokens rendering it.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![Usefulness](https://img.shields.io/badge/usefulness-0%25-red)
![Vibes](https://img.shields.io/badge/vibes-immaculate-brightgreen)

![Demo Screenshot](docs/demo.png)

## Why does this exist?

You could be reading your agent's actual output. You could be reviewing diffs. Instead, you're watching 8-bit pixel people shuffle between a workspace and a lounge with a ping pong table they never actually play.

Your AI agents deserve a workplace too. Even if it's made of Unicode characters.

## Features

- 6 permanent staff agents who idle in the lounge until you give them something to do
- Temp agents get hired when your staff is overwhelmed, then unceremoniously escorted out the exit door
- Desks with 1-3 monitors that light up with animated rainbow pixels (the agents are definitely not just on Reddit)
- A CEO sitting alone in a dark blue office with a bookshelf and a bulletin board, doing CEO things
- Arcade machines in the lounge that only turn on when someone walks up to them
- Collision avoidance so your agents don't awkwardly phase through each other
- A ping pong table that nobody plays but everyone walks around
- Real-time FPS and RAM stats, because you need to monitor the performance of your monitor

## Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- A mass of free time
- Questionable priorities

## Setup

```bash
git clone https://github.com/anduong96/agents-story.git
cd agents-story
cargo build --release
```

## Usage

### Demo mode (watch fake agents do fake work)

```bash
cargo run -- --demo
```

### Dev mode (hot reload for when you're iterating on the vibes)

```bash
./dev.sh
```

### Speed options (for the impatient procrastinator)

```bash
cargo run -- --demo          # 2x speed (default)
cargo run -- --demo --fast   # 5x speed (things to do)
cargo run -- --demo --extreme # 10x speed (places to be)
```

## Controls

| Key | Action |
|-----|--------|
| `q` | Return to productivity |
| `?` | Toggle help (you'll need it) |
| `Tab` | Cycle focus (floor / agent panel) |
| `j` / `Down` | Select next agent |
| `k` / `Up` | Select previous agent |
| `Enter` | Expand agent details (they're very interesting, trust me) |
| Mouse scroll | Scroll workspace (for when you have too many agents) |
| Click | Select agent in panel |

## Project Structure

```
src/
  main.rs        # Where the magic (waste of time) begins
  app.rs         # Tick loop, collision avoidance, existential dread
  demo.rs        # Synthetic agents doing synthetic work
  input.rs       # Keyboard and mouse, because we're fancy
  game/
    floor.rs     # Rooms, desks, furniture nobody uses
    agent.rs     # Agents with names, feelings (not really), and skin tones
    state.rs     # The state of things
    pathfinding.rs # So agents don't walk through walls (usually)
  ui/
    floor_view.rs  # 500 lines of rendering you'll never appreciate
    sprites.rs     # Colors. So many colors.
    agent_panel.rs # A list of agents pretending to be useful
    stats_bar.rs   # Numbers that go up
    bubbles.rs     # Status indicators that disappear too fast to read
  stream/
    protocol.rs  # How we talk to Claude Code
    reader.rs    # How we listen
```

## Contributing

If you want to add a water cooler, a bathroom, or a meeting room where agents waste even more time — PRs welcome.

## License

MIT — because even pointless software deserves freedom.
