use ratatui::style::Color;

// Agents are rendered as name tags (2-3 letter abbreviation in agent color)
// No sprite struct needed — render_agent handles it directly.

pub const CEO_COLOR: Color = Color::Rgb(255, 215, 0); // Gold
pub const CEO_SKIN_COLOR: Color = Color::Rgb(220, 180, 130); // CEO skin tone

// Monitor sprite — 2 cells wide × 2 tall
#[allow(dead_code)]
pub struct MonitorSprite {
    pub top: [&'static str; 2],
    pub bottom: [&'static str; 2],
}

#[allow(dead_code)]
pub const MONITOR: MonitorSprite = MonitorSprite {
    top: ["█▓", "▓█"],
    bottom: ["▀▀", "▀▀"],
};

#[allow(dead_code)]
pub const MONITOR_FLICKER: MonitorSprite = MonitorSprite {
    top: ["█▒", "▒█"],
    bottom: ["▀▀", "▀▀"],
};

// Ping pong table: 6 wide × 2 tall
#[allow(dead_code)]
pub const PING_PONG_TABLE: [[&str; 6]; 2] = [
    ["▄", "▄", "▄", "▄", "▄", "▄"],
    ["█", "▒", "▒", "▒", "▒", "█"],
];

pub const PING_PONG_COLOR: Color = Color::Rgb(0, 128, 0);
pub const PING_PONG_NET_COLOR: Color = Color::Rgb(220, 220, 220);

// Plants/trees
// Bookshelf
pub const BOOKSHELF_COLOR: Color = Color::Rgb(120, 85, 50);
pub const BOOKSHELF_BOOK_COLORS: [Color; 4] = [
    Color::Rgb(200, 70, 70),
    Color::Rgb(70, 130, 200),
    Color::Rgb(70, 170, 70),
    Color::Rgb(220, 200, 70),
];

pub const PLANT_COLOR: Color = Color::Rgb(60, 180, 60);
pub const PLANT_POT_COLOR: Color = Color::Rgb(160, 100, 50);


// Lounge TV
pub const TV_FRAME_COLOR: Color = Color::Rgb(40, 40, 45);
pub const TV_SCREEN_COLOR: Color = Color::Rgb(60, 120, 200);

// Room wall/door colors — Kairosoft uses LIGHT cream/beige walls, not dark gray
pub const WALL_COLOR: Color = Color::Rgb(190, 185, 170);     // cream/beige walls
pub const DOOR_COLOR: Color = Color::Rgb(180, 130, 70);      // warm wood door

// Monitor/desk colors
pub const MONITOR_COLOR: Color = Color::Rgb(60, 180, 255);
pub const MONITOR_FLICKER_COLOR: Color = Color::Rgb(40, 140, 200);

// Desk sprite colors
pub const DESK_FRAME_COLOR: Color = Color::Rgb(80, 85, 100);      // blue-gray monitor bezel
pub const DESK_SURFACE_COLOR: Color = Color::Rgb(210, 205, 195);   // off-white desk surface
pub const DESK_SCREEN_OFF_COLOR: Color = Color::Rgb(35, 40, 50);  // dark blue-gray off screen

// All desks same width (10w × 3h), monitor count varies inside
// 1 monitor — centered:
//  ┌────────┐
//  │  ▓▓▓▓  │
//  └────────┘
pub const DESK1_ROW0: [char; 10] = ['┌', '─', '─', '─', '─', '─', '─', '─', '─', '┐'];
pub const DESK1_ROW1: [char; 10] = ['│', ' ', ' ', '▓', '▓', '▓', '▓', ' ', ' ', '│'];
pub const DESK1_ROW2: [char; 10] = ['└', '─', '─', '─', '─', '─', '─', '─', '─', '┘'];
pub const DESK1_SCREEN_COLS: &[usize] = &[3, 4, 5, 6];

// 2 monitors — side by side:
//  ┌────────┐
//  │ ▓▓▓▓▓▓ │
//  └────────┘
pub const DESK2_ROW0: [char; 10] = ['┌', '─', '─', '─', '─', '─', '─', '─', '─', '┐'];
pub const DESK2_ROW1: [char; 10] = ['│', ' ', '▓', '▓', '▓', '▓', '▓', '▓', ' ', '│'];
pub const DESK2_ROW2: [char; 10] = ['└', '─', '─', '─', '─', '─', '─', '─', '─', '┘'];
pub const DESK2_SCREEN_COLS: &[usize] = &[2, 3, 4, 5, 6, 7];

// 3 monitors — full width screens:
//  ┌────────┐
//  │▓▓▓▓▓▓▓▓│
//  └────────┘
pub const DESK3_ROW0: [char; 10] = ['┌', '─', '─', '─', '─', '─', '─', '─', '─', '┐'];
pub const DESK3_ROW1: [char; 10] = ['│', '▓', '▓', '▓', '▓', '▓', '▓', '▓', '▓', '│'];
pub const DESK3_ROW2: [char; 10] = ['└', '─', '─', '─', '─', '─', '─', '─', '─', '┘'];
pub const DESK3_SCREEN_COLS: &[usize] = &[1, 2, 3, 4, 5, 6, 7, 8];

// Workspace floor — two-shade brick pattern
pub const WORKSPACE_FLOOR_BG_EVEN: Color = Color::Rgb(150, 115, 70);  // lighter brick
pub const WORKSPACE_FLOOR_BG_ALT: Color = Color::Rgb(140, 105, 62);   // darker brick

// Lounge floor — felt with colon texture
pub const LOUNGE_FLOOR_FG: Color = Color::Rgb(120, 135, 120);         // dot color
pub const LOUNGE_FLOOR_BG_EVEN: Color = Color::Rgb(100, 115, 100);    // felt base

// CEO floor — rich dark blue carpet
pub const CEO_FLOOR_FG_EVEN: Color = Color::Rgb(70, 75, 120);
pub const CEO_FLOOR_BG_EVEN: Color = Color::Rgb(55, 58, 95);
pub const CEO_FLOOR_BG_ODD: Color = Color::Rgb(48, 50, 85);
pub const CEO_FLOOR_CHAR_EVEN: char = '·';
pub const CEO_FLOOR_CHAR_ODD: char = ' ';

// Vibrant screen pixel colors (Kairosoft-style rainbow)
pub const SCREEN_PIXELS: [Color; 10] = [
    Color::Rgb(255, 100, 130),   // pink
    Color::Rgb(100, 220, 100),   // green
    Color::Rgb(80, 160, 255),    // blue
    Color::Rgb(255, 210, 60),    // yellow/gold
    Color::Rgb(255, 150, 50),    // orange
    Color::Rgb(190, 120, 255),   // purple
    Color::Rgb(60, 220, 220),    // cyan
    Color::Rgb(255, 85, 85),     // red
    Color::Rgb(180, 255, 130),   // lime
    Color::Rgb(255, 170, 210),   // light pink
];

// Lounge furniture — warm, inviting Kairosoft colors
pub const COUCH_COLOR: Color = Color::Rgb(180, 120, 60);       // warm leather brown
pub const COUCH_FRAME_COLOR: Color = Color::Rgb(140, 90, 45);  // darker wood frame
pub const COFFEE_TABLE_COLOR: Color = Color::Rgb(160, 110, 55);
pub const VENDING_MACHINE_COLOR: Color = Color::Rgb(100, 150, 220);   // cheerful blue
pub const VENDING_LIGHT_COLOR: Color = Color::Rgb(140, 220, 140);     // bright green

// CEO office — premium feel
pub const BULLETIN_BOARD_COLOR: Color = Color::Rgb(200, 165, 100);    // bright cork
pub const BULLETIN_PIN_COLORS: [Color; 4] = [
    Color::Rgb(255, 70, 70),     // red
    Color::Rgb(70, 210, 70),     // green
    Color::Rgb(70, 130, 255),    // blue
    Color::Rgb(255, 230, 50),    // yellow
];
// Lightweight status indicators (replace heavy chat bubbles)
pub const STATUS_WORKING: char = '⚙';
pub const STATUS_IDLE: char = '○';
pub const STATUS_ERROR: char = '!';
pub const STATUS_FINISHED: char = '✓';
pub const STATUS_SPAWNING: char = '◆';
pub const STATUS_TOOL_READ: char = '◇';
pub const STATUS_TOOL_EDIT: char = '✎';
pub const STATUS_TOOL_BASH: char = '▸';
pub const STATUS_TOOL_SEARCH: char = '◎';

pub const STATUS_WORKING_COLOR: Color = Color::Rgb(100, 255, 100);   // bright green
pub const STATUS_IDLE_COLOR: Color = Color::Rgb(180, 180, 180);      // gray
pub const STATUS_ERROR_COLOR: Color = Color::Rgb(255, 80, 80);       // red
pub const STATUS_FINISHED_COLOR: Color = Color::Rgb(100, 200, 255);  // cyan
pub const STATUS_SPAWNING_COLOR: Color = Color::Rgb(255, 220, 80);   // yellow
pub const STATUS_TOOL_COLOR: Color = Color::Rgb(255, 200, 100);      // warm yellow
