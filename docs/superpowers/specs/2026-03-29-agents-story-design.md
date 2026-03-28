# Agents Story — Design Spec

A Game Dev Story-inspired TUI visualizer for Claude Code, built with Rust and Ratatui. Renders a pixel-art office where AI agents move between rooms based on their real-time status, driven by JSONL streaming from active Claude Code sessions.

**Target terminal:** Ghostty (GPU-accelerated, full truecolor, full Unicode)

---

## 1. Project Structure & Dependencies

```
agents-story/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, tick loop, event dispatch
│   ├── app.rs               # App state, mode tracking, frame rate control
│   ├── game/
│   │   ├── mod.rs
│   │   ├── state.rs         # GameState: agents, positions, stats
│   │   ├── agent.rs         # Agent struct, status enum, animation state
│   │   ├── pathfinding.rs   # Waypoint-based pathfinding between rooms
│   │   └── floor.rs         # Room layout, grid cells, collision map
│   ├── stream/
│   │   ├── mod.rs
│   │   ├── discovery.rs     # Watch ~/.claude/ for active sessions
│   │   ├── reader.rs        # JSONL line reader per session
│   │   └── protocol.rs      # Claude Code message types (serde deserialize)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── floor_view.rs    # Render the rooms, agents, objects
│   │   ├── stats_bar.rs     # Model, agents, tasks, tokens, cost line
│   │   ├── agent_panel.rs   # Agent list with details
│   │   ├── sprites.rs       # Braille/half-block art definitions
│   │   └── bubbles.rs       # Text bubble rendering and lifecycle
│   └── input.rs             # Keyboard/mouse handler
```

### Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | TUI rendering framework |
| `crossterm` | Terminal backend (input, raw mode, colors) |
| `tokio` | Async runtime for JSONL readers and file watchers |
| `serde` + `serde_json` | JSONL deserialization |
| `notify` | Filesystem watcher for session discovery |

---

## 2. Architecture: Tick-Based Game Loop

The main loop runs in a `tokio` runtime. The tick loop polls for events within a frame budget.

```
loop {
    1. Poll input (keyboard/mouse) — non-blocking
    2. Drain JSONL events from channel
    3. Update game state (agent positions, stats, animation steps)
    4. Render frame via Ratatui
    5. Sleep for remainder of frame budget
}
```

### Adaptive Frame Rate

- **Active** (animations playing): 15 FPS (~66ms per frame)
- **Idle** (no animations, no new events): 2 FPS (~500ms per frame)
- Transitions instantly back to 15 FPS when a JSONL event arrives or an animation starts

### Threading Model

```
                    ┌─────────────────────┐
                    │   Main Thread        │
                    │   (tick loop)        │
                    │                     │
  mpsc::Receiver ──>│  drain events       │
                    │  update state        │
                    │  render              │
                    └─────────────────────┘
                              ^
              ┌───────────────┼───────────────┐
              │               │               │
     ┌────────────┐  ┌────────────┐  ┌────────────┐
     │ Discovery  │  │ Reader #1  │  │ Reader #2  │
     │ (tokio)    │  │ (tokio)    │  │ (tokio)    │
     └────────────┘  └────────────┘  └────────────┘
```

Background async tasks communicate via `tokio::sync::mpsc` channels:
- Session discovery task sends `SessionEvent::New(id, path)` / `SessionEvent::Ended(id)`
- One JSONL reader task per active session sends `StreamEvent` variants

---

## 3. Floor Layout

Two-row layout. Workspace spans full width on top. Lounge and CEO Office split the bottom.

```
┌─── Workspace (100% width, 65% height) ────────────────────────────────┐
│                                                                        │
│  D M    D M    D M    D M    D M    D M                               │
│                                                                        │
│  D M    D M    D M    D M    D M    D M                               │
│                                                                        │
│  D M    D M    D M    D M                                             │
│                                                                        │
├──┬──┬──────────────────────────────────────┬──┬────┬───────┬──┬───────┤
│  door                                      door    │       door       │
│  Lounge (75% width, 35% height)                    │  CEO (25%, 35%) │
│                                                    │                  │
│      PPP          ⣿⣷        ⣾⣿                    │    D M           │
│      PPP                                           │     ⣿⣷          │
│                                                    │                  │
└────────────────────────────────────────────────────┴──────────────────┘
```

### Grid System

Each cell is one of: `Empty`, `Wall`, `Door`, `Desk`, `Monitor`, `PingPongTable`, `CeoDesk`.

### Rooms

- **Workspace** — 100% width, 65% height. Contains rows of desks, each with 1-3 monitors. Agents assigned to specific desks.
- **Lounge** — 75% width, 35% height. Contains a centered ping pong table. Idle agents wander here.
- **CEO Office** — 25% width, 35% height. One desk with 2-3 monitors. CEO sprite sits here permanently.

### Doors

- **Lounge door-left**: top-left of Lounge, connects to Workspace
- **Lounge door-right**: top-right of Lounge, connects to Workspace
- **CEO door**: top of CEO Office, connects to Workspace
- No direct door between Lounge and CEO Office

### Visual Style

Hybrid of box-drawing and braille/half-block:
- **Walls and room structure**: box-drawing characters (`│`, `─`, `┌`, `┐`, `└`, `┘`, `├`, `┤`, `┬`, `┴`)
- **Agent sprites and objects**: braille dots (`⣿⣷⣾`) and half-block characters (`▄▀█▓▒░`)
- **Full 24-bit truecolor** for all elements

---

## 4. Pathfinding

Waypoint graph, not A*. Three rooms with fixed doorways make full pathfinding unnecessary.

### Waypoint Strategy

```
Agent's current position → nearest door out → door into target room → target position
```

Each agent's path is a `Vec<(u16, u16)>` of waypoints. On each tick, the agent advances toward the next waypoint at 4 tiles/second. When reached, it pops and aims for the next.

### Door Selection

Agents pick the closest door. For Lounge, the two doors allow agents to pick whichever is nearer to their origin/destination, reducing walking distance.

### Route Examples

- **Lounge → Workspace**: Lounge door (nearest) → up → Workspace floor → target desk
- **Lounge → CEO**: Lounge door → up through Workspace → down through CEO door → CEO position
- **CEO → Workspace**: CEO door → up → Workspace floor → target desk
- **Workspace → Lounge**: Current position → down to nearest Lounge door → Lounge position

### Collision

No agent-to-agent collision. Agents can overlap. This keeps pathfinding trivial and avoids traffic jams in doorways. Matches Game Dev Story behavior.

---

## 5. Agent Model

```rust
Agent {
    id: String,              // from Claude Code session
    name: String,            // e.g., "agent-a1", "Explore", "code-reviewer"
    status: AgentStatus,     // Working, Idle, Spawning, Finished, Error
    model: String,           // "opus-4", "sonnet-4", "haiku-4.5"
    task: Option<String>,    // "Fix auth bug", "Run tests", etc.
    session: SessionInfo,    // repo, branch, worktree path

    // Visual state
    position: (f32, f32),    // sub-tile position for smooth movement
    target_room: Room,       // where they should be
    path: Vec<(u16, u16)>,   // waypoints to walk
    sprite: SpriteVariant,   // visual appearance (color + shape)
    facing: Direction,       // Left or Right (flip sprite when walking)
}
```

### Status-to-Room Mapping

| Status | Room | Behavior |
|--------|------|----------|
| `Working` | Workspace | Walks to assigned desk, sits, monitors glow |
| `Idle` | Lounge | Walks to lounge, wanders near ping pong table |
| `Spawning` | Workspace | Appears at nearest door, walks to a desk |
| `Finished` | Lounge | Walks from desk to lounge, fades out after a few seconds |
| `Error` | Workspace | Stays at desk, sprite turns red |

When status changes, the agent gets a new `target_room` and a fresh path is computed.

### Desk Assignment

- Desks are numbered sequentially in the Workspace grid (left-to-right, top-to-bottom)
- Each new agent gets the next available desk
- If all desks are occupied, agents share a desk (stack visually — sprites overlap, same as collision rules)
- When an agent leaves (Finished/Idle), their desk is freed and can be reassigned

### Walking Speed

4 tiles/second. Crossing a room takes ~2-3 seconds, matching Game Dev Story pacing.

### Sprite Variants

Each agent gets a distinct color from a cycling palette:

```
 ⣿⣷    ⣿⣷    ⣿⣷    ⣿⣷
 ⣿⣿    ⣿⣿    ⣿⣿    ⣿⣿
green  cyan  magenta yellow
```

### Idle Animations

- **Working agent**: monitor flickers between `▓` and `▒` every ~2 seconds
- **Idle agent**: sways position by 1 sub-tile left/right slowly
- **CEO**: static, slight glow on desk

---

## 6. CEO Sprite

The CEO represents the user (the prompter). Permanently in the CEO Office.

### Appearance

- Gold/yellow color — unique from the agent palette
- Larger desk with 2-3 monitors
- Never walks to other rooms

### CEO State (derived from JSONL)

| Event | Visual | Bubble Examples |
|-------|--------|-----------------|
| Idle (no active prompt) | Sitting at desk, relaxed | `"thinking..."`, `"planning"`, `"hmm..."` |
| Prompt sent | Leans forward, desk glows | `"let's build"`, `"go go go"`, `"ship it"` |
| Waiting for agents | Sitting, occasional tap | `"waiting..."`, `"any updates?"`, `"come on..."` |
| All tasks complete | Leans back | `"nice work"`, `"that's a wrap"`, `"let's go home"` |
| Error from agent | Stands up | `"what happened?"`, `"uh oh"`, `"fix it!"` |
| Spawns many agents | Arms up | `"all hands!"`, `"everyone on this"` |

### State Derivation

- Prompt sent: a new `user` message in the JSONL stream
- Waiting: `user` message sent, no `result` yet
- All complete: all agents in `Finished` or `Idle`
- Error: any agent has `Error` status

---

## 7. Text Bubbles

Speech bubbles float above agent sprites, triggered heuristically based on status and JSONL events.

### Rendering

```
  ┌───────────┐
  │ shipping! │
  └─────┬─────┘
        ⣿⣷
        ⣿⣿
```

### Trigger Table

| Trigger | Example Bubbles | When |
|---------|----------------|------|
| Agent starts a task | `"on it!"`, `"let's go"`, `"diving in..."` | Status → Working |
| Agent finishes a task | `"shipped!"`, `"done!"`, `"nailed it"` | Status → Finished |
| Agent hits an error | `"ugh..."`, `"hmm..."`, `"not good"` | Status → Error |
| Agent goes idle | `"break time"`, `"brb"`, `"coffee..."` | Status → Idle |
| Agent enters lounge | `"ping pong?"`, `"nice"`, `"chill"` | Arrives in Lounge |
| Long task (>2 min) | `"still going..."`, `"almost..."`, `"deep in it"` | Timer while Working |
| Tool use detected | `"reading..."`, `"writing..."`, `"searching..."` | JSONL tool_use event |
| Large file edit | `"big change..."`, `"refactoring..."` | Edit event with many lines |
| Tests running | `"fingers crossed"`, `"testing..."` | Bash event matching test commands |
| Agent spawns subagent | `"need backup"`, `"calling in help"` | Agent tool_use event |

### Data-Driven Bubbles

Some bubbles pull from actual JSONL data for specificity:

| JSONL Data | Bubble |
|------------|--------|
| Tool: `Read("auth.rs")` | `"reading auth.rs"` |
| Tool: `Edit("main.rs")` | `"editing main.rs"` |
| Tool: `Bash("cargo test")` | `"running tests..."` |
| Task subject available | `"on: Fix auth bug"` |

### Rules

- One bubble per agent at a time (new replaces old)
- Random selection from pool per trigger to avoid repetition
- Lifetime: 3-5 seconds (randomized so they don't all vanish together)
- Max 3 bubbles visible simultaneously across all agents — newest wins
- Auto-positions above agent, shifts left/right if it would clip a wall

---

## 8. Stats Bar

Single line between the floor and agent panel. Always visible.

```
 Model: opus-4 │ Agents: 3/5 │ Tasks: 2/7 │ Tokens: 14.2k │ Cost: $0.42 │ Usage: 67%
```

| Stat | Source | Format | Color Logic |
|------|--------|--------|-------------|
| Model | JSONL `init` message | Short name (`opus-4`) | White |
| Agents | Active / total discovered | `3/5` | Green if all healthy, yellow if any errored |
| Tasks | Completed / total | `2/7` | Cyan |
| Tokens | Sum across all sessions | Auto-scaled (`14.2k`, `1.2M`) | White |
| Cost | Sum across all sessions | `$0.42` (2 decimal places) | Green < $1, Yellow < $5, Red > $5 |
| Usage | Session context window % | `67%` | Green < 50%, Yellow < 80%, Red > 80% |

---

## 9. Agent Panel

Scrollable list below the stats bar. Supports selection and expansion.

### Collapsed View (one line per agent)

```
├─ Agents ─────────────────────────────────────────────────────────────────┤
│ ● agent-a1   WORKING  "Fix auth bug"          main @ repo-x (wt-1)     │
│ ● agent-b2   WORKING  "Add tests"             feat @ repo-x (wt-2)     │
│ ○ agent-c3   IDLE                              main @ repo-x            │
└──────────────────────────────────────────────────────────────────────────┘
```

Status indicators: `●` green = working, `●` red = error, `○` gray = idle, `◌` dim = finished

### Expanded View (Enter on selected agent)

```
│ ▼ agent-a1   WORKING  "Fix auth bug"          main @ repo-x (wt-1)     │
│   Model: opus-4  Tokens: 4.2k  Cost: $0.12                             │
│   Current tool: Edit("src/auth.rs")                                     │
│   Duration: 1m 32s                                                      │
```

---

## 10. Session Discovery & JSONL Protocol

### Discovery

The TUI auto-discovers active Claude Code sessions:

1. On startup, scan `~/.claude/` for active session indicators (lock files, JSONL streams, PID files)
2. Use `notify` crate to watch for new sessions appearing
3. When a new session is detected, spawn a tokio task to read its JSONL stream
4. When a session ends (process exits, stream closes), mark its agents as `Finished`

### Pre-build Spike Required

Before building, a spike is needed to:
- Run `claude --output-format stream-json` and capture the actual message types
- Map `~/.claude/` directory structure to understand where sessions live
- Determine if subagent streams are separate files or nested in the parent stream

### Protocol Types

```rust
enum StreamEvent {
    Init { model: String, session_id: String },
    TextDelta { text: String },
    ToolUse { tool: String, args: serde_json::Value },
    ToolResult { tool: String, output: String },
    AgentSpawn { agent_id: String, name: String, description: String },
    AgentResult { agent_id: String },
    TaskUpdate { task_id: String, status: String, subject: String },
    Stats { tokens: u64, cost: f64, usage_percent: f32 },
    Result { text: String },
    Error { message: String },
}
```

These variants are best-guess and will be refined during the spike. The parser uses `parse_line(line: &str) -> Option<StreamEvent>` and gracefully ignores unknown message types.

### Event-to-State Mapping

| StreamEvent | Game State Change |
|-------------|-------------------|
| `Init` | Register new session, create CEO state |
| `AgentSpawn` | Create Agent, assign desk, start walk-in animation |
| `ToolUse` | Update agent's current task, trigger bubble |
| `AgentResult` | Set agent → Finished, walk to Lounge |
| `TaskUpdate` | Update stats bar task count |
| `Stats` | Update stats bar tokens/cost/usage |
| `Error` | Set agent → Error, red sprite, bubble |

---

## 11. Input & Interaction

Read-only with light interaction. No text input.

### Keybindings

| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit |
| `j` / `↓` | Select next agent in panel |
| `k` / `↑` | Select previous agent in panel |
| `Enter` | Toggle expanded view on selected agent |
| `Tab` | Cycle focus: floor → stats → agent panel |
| `1` / `2` / `3` | Jump focus to Workspace / Lounge / CEO Office (highlights room) |
| `?` | Toggle help overlay |

### Mouse Support (optional)

- Click agent sprite on floor → selects in agent panel and expands
- Click a room → highlights it

### Focus Model

- One panel has focus at a time (brighter border indicates focus)
- `Tab` cycles: floor → stats → agent panel
- Keyboard actions are context-dependent based on focused panel

---

## 12. Resource Budget

| Metric | Target |
|--------|--------|
| RAM | < 5 MB |
| CPU idle | < 1% (2 FPS) |
| CPU animating | < 2% (15 FPS) |
| Binary size | < 5 MB |
| Startup time | < 100ms |
