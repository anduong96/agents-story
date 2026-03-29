# Agents Story

A pixel-art TUI that turns Claude Code agent sessions into a tiny office simulation. Does nothing useful. Entertaining anyway.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![Vibes](https://img.shields.io/badge/vibes-immaculate-brightgreen)

![Demo Screenshot](docs/demo.png)

## What is this

Your Claude Code agents read files, write code, run tests. You experience this as scrolling text. This replaces that with pixel people walking to desks and sitting down.

It won't make your agents faster or smarter. It's a screensaver with opinions.

## Features

- **6 staff agents** idle in the lounge until assigned work. They wander near the arcade machines.
- **Temp contractors** appear when demand exceeds headcount. They leave through the top door when done.
- **A CEO** who sprints to the whiteboard and yells when a new task arrives.
- **Desks with animated monitors** showing rainbow pixels. Definitely not Reddit.
- **Arcade machines** that light up when someone walks near them.
- **Collision avoidance.** Standards.
- **FPS and RAM stats.** For monitoring the performance of your monitoring tool.

## Setup

```bash
git clone https://github.com/anduong96/agents-story.git
cd agents-story
cargo build --release
```

Requires [Rust](https://rustup.rs/) 1.70+.

## Usage

```bash
cargo run -- --demo              # demo mode
cargo run -- --demo --fast       # 5x
cargo run -- --demo --extreme    # 10x
./dev.sh                         # hot reload
```

## Controls

| Key | Action |
|-----|--------|
| `q` | Quit |
| `?` | Help |
| `Tab` | Switch floor / agent panel |
| `j` / `k` | Navigate agents |
| `Enter` | Expand agent details |
| Scroll | Scroll workspace |
| Click | Select agent |

## Roadmap

- [ ] Homebrew formula (`brew install agents-story`)
- [ ] Connect to live Claude Code sessions
- [ ] Water cooler
- [ ] Meeting room where agents accomplish nothing

## Contributing

PRs welcome.

## License

MIT
