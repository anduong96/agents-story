use ratatui::style::Color;

// Agent sprite — 2 cells wide × 2 cells tall using braille characters
pub struct Sprite {
    pub top: [&'static str; 2],
    pub bottom: [&'static str; 2],
}

pub const AGENT_RIGHT: Sprite = Sprite {
    top: ["⣿", "⣷"],
    bottom: ["⣿", "⣿"],
};

pub const AGENT_LEFT: Sprite = Sprite {
    top: ["⣾", "⣿"],
    bottom: ["⣿", "⣿"],
};

pub const CEO_SPRITE: Sprite = Sprite {
    top: ["⣿", "⣷"],
    bottom: ["⣿", "⣿"],
};

pub const CEO_COLOR: Color = Color::Rgb(255, 215, 0); // Gold

// Monitor sprite — 2 cells wide × 2 tall
pub struct MonitorSprite {
    pub top: [&'static str; 2],
    pub bottom: [&'static str; 2],
}

pub const MONITOR: MonitorSprite = MonitorSprite {
    top: ["█▓", "▓█"],
    bottom: ["▀▀", "▀▀"],
};

pub const MONITOR_FLICKER: MonitorSprite = MonitorSprite {
    top: ["█▒", "▒█"],
    bottom: ["▀▀", "▀▀"],
};

// Ping pong table: 6 wide × 2 tall
pub const PING_PONG_TABLE: [[&str; 6]; 2] = [
    ["▄", "▄", "▄", "▄", "▄", "▄"],
    ["█", "▒", "▒", "▒", "▒", "█"],
];

pub const PING_PONG_COLOR: Color = Color::Rgb(0, 128, 0);

// Room wall/door colors
pub const WALL_COLOR: Color = Color::Rgb(100, 100, 120);
pub const DOOR_COLOR: Color = Color::Rgb(139, 90, 43);

// Room background tints
pub const WORKSPACE_BG: Color = Color::Rgb(15, 15, 25);
pub const LOUNGE_BG: Color = Color::Rgb(20, 15, 15);
pub const CEO_BG: Color = Color::Rgb(25, 20, 10);

// Monitor/desk colors
pub const MONITOR_COLOR: Color = Color::Rgb(60, 180, 255);
pub const MONITOR_FLICKER_COLOR: Color = Color::Rgb(40, 140, 200);
pub const DESK_COLOR: Color = Color::Rgb(139, 90, 43);
