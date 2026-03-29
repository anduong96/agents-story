# Agents Story

> You've been staring at a stream of text in your terminal for forty minutes. It's fine. Everything is fine.

Agents Story replaces the deeply engaging experience of watching Claude Code output scroll past with something marginally more interesting: a pixel-art office where tiny people sit at desks and pretend to be productive. Just like the rest of us.

It doesn't make your agents faster. It doesn't make them smarter. It doesn't do anything, really. But at least now when someone walks by your screen, it looks like you're playing a game instead of waiting for a build to finish. Which is arguably a lateral move.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![Usefulness](https://img.shields.io/badge/usefulness-negligible-red)
![Vibes](https://img.shields.io/badge/vibes-immaculate-brightgreen)
![Status](https://img.shields.io/badge/status-it%20compiles-yellow)

![Demo Screenshot](docs/demo.png)

## The Problem

Claude Code agents work in your terminal. They read files, edit code, run tests. Important stuff. And the way you experience all of this is: text. Scrolling. Forever.

You could read it. You could also not read it and watch pixel people walk to their desks instead. We're not here to judge.

## What You Get

- **6 permanent employees** who hang out in the lounge until you give them something to do. They wander near the arcade machines. They stand near the ping pong table. They do not play ping pong.
- **Temp contractors** who show up when your staff can't keep up, do their job, and leave through the top door without saying goodbye.
- **A CEO** who sits alone in a blue office with a bookshelf and a bulletin board. When a new task arrives, the CEO sprints to the whiteboard and yells. This is the most realistic part of the simulation.
- **Desks with monitors** that display animated rainbow pixels. Your agents are definitely working and not browsing Reddit.
- **Collision avoidance** so agents don't walk through each other. We have standards.
- **Real-time FPS and RAM stats** in the status bar, because you should absolutely be monitoring the performance of your monitoring tool.

## Setup

You need [Rust](https://rustup.rs/) 1.70+ and a tolerance for whimsy.

```bash
git clone https://github.com/anduong96/agents-story.git
cd agents-story
cargo build --release
```

That's it. We didn't say it was hard. We said it was pointless.

## Usage

```bash
cargo run -- --demo              # Watch fake agents do fake work
cargo run -- --demo --fast       # Same thing, but faster
cargo run -- --demo --extreme    # You're in a hurry to waste time
./dev.sh                         # Hot reload. For the craftsperson.
```

## Controls

| Key | What it does |
|-----|-------------|
| `q` | Leave. Go be productive. We won't stop you. |
| `?` | Help. You need it. |
| `Tab` | Switch between the floor view and the agent list |
| `j` / `k` | Navigate agents. Like vim, but less useful. |
| `Enter` | Expand agent details. Model, tokens, cost. Things that matter, rendered in a context where they don't. |
| Scroll | Scroll the workspace when you've hired too many people |
| Click | Select an agent. Yes, we support mouse input. We're not animals. |

## The Office

```
┌─────────────────────────────────────┐
│  ♠  Workspace                   ♠  │
│     ┌────────┐  ┌────────┐         │
│     │▓▓▓▓▓▓▓▓│  │ ▓▓▓▓▓▓ │    ░   │
│     └────────┘  └────────┘    ░░   │
│       ██  ██      ██  ██     ░░░   │
│       ██  ██      ██  ██    ░░░░   │
│   ♣                         ░░░░░  │  ← whiteboard
├────────────────────┬────────────────┤
│  Lounge       ▒▒▒│▒▒  │  CEO      │
│  ♠   ██  ██   ▒▒▒│▒▒  │  ┌──────┐ │
│      ██  ██        │  ██  │  ▓▓  │ │
│  █▓█▓             │  ██  │      │ │
│  █▓█▓  ♣          │      └──────┘ │
└─────────┴──────────┴────────────────┘
```

The workspace has desks, trees, and a whiteboard. The lounge has a ping pong table (decorative), arcade machines (functional when someone stands near them), and plants (also decorative). The CEO has a bookshelf and opinions.

## Contributing

The office needs a water cooler. It needs a bathroom. It needs a meeting room where four agents can sit for an hour and accomplish nothing. If any of this speaks to you, PRs are open.

## License

MIT — because restricting access to something this unnecessary felt wrong.
