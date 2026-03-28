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

// Room wall/door colors — Kairosoft uses LIGHT cream/beige walls, not dark gray
pub const WALL_COLOR: Color = Color::Rgb(190, 185, 170);     // cream/beige walls
pub const DOOR_COLOR: Color = Color::Rgb(180, 130, 70);      // warm wood door

// Monitor/desk colors
pub const MONITOR_COLOR: Color = Color::Rgb(60, 180, 255);
pub const MONITOR_FLICKER_COLOR: Color = Color::Rgb(40, 140, 200);

// Desk sprite colors — dark monitor frame on bright wood desk
pub const DESK_FRAME_COLOR: Color = Color::Rgb(50, 50, 55);       // dark monitor bezel
pub const DESK_SCREEN_OFF_COLOR: Color = Color::Rgb(35, 40, 50);  // dark blue-gray off screen

// Desk variants — continuous screens, no dividers between monitors
// 1 monitor (4w × 3h):
pub const DESK1_ROW0: [char; 4] = ['┌', '─', '─', '┐'];
pub const DESK1_ROW1: [char; 4] = ['│', '▓', '▓', '│'];
pub const DESK1_ROW2: [char; 4] = ['└', '─', '┬', '┘'];
pub const DESK1_SCREEN_COLS: &[usize] = &[1, 2];

// 2 monitors (7w × 3h) — continuous wide screen:
pub const DESK2_ROW0: [char; 7] = ['┌', '─', '─', '─', '─', '─', '┐'];
pub const DESK2_ROW1: [char; 7] = ['│', '▓', '▓', '▓', '▓', '▓', '│'];
pub const DESK2_ROW2: [char; 7] = ['└', '─', '─', '┬', '─', '─', '┘'];
pub const DESK2_SCREEN_COLS: &[usize] = &[1, 2, 3, 4, 5];

// 3 monitors (10w × 3h) — continuous ultra-wide screen:
pub const DESK3_ROW0: [char; 10] = ['┌', '─', '─', '─', '─', '─', '─', '─', '─', '┐'];
pub const DESK3_ROW1: [char; 10] = ['│', '▓', '▓', '▓', '▓', '▓', '▓', '▓', '▓', '│'];
pub const DESK3_ROW2: [char; 10] = ['└', '─', '─', '─', '┬', '─', '─', '─', '─', '┘'];
pub const DESK3_SCREEN_COLS: &[usize] = &[1, 2, 3, 4, 5, 6, 7, 8];

// Workspace floor — warm caramel/tan wood planks (BRIGHT, like Kairosoft)
pub const WORKSPACE_FLOOR_FG_EVEN: Color = Color::Rgb(170, 130, 80);   // visible wood grain
pub const WORKSPACE_FLOOR_BG_EVEN: Color = Color::Rgb(150, 115, 70);   // warm caramel
pub const WORKSPACE_FLOOR_BG_ODD: Color = Color::Rgb(140, 105, 65);    // slightly darker plank
pub const WORKSPACE_FLOOR_CHAR_EVEN: char = '━';   // thicker line for wood plank look
pub const WORKSPACE_FLOOR_CHAR_ODD: char = ' ';

// Lounge floor — lighter warm gray (like carpet/tile)
pub const LOUNGE_FLOOR_FG_EVEN: Color = Color::Rgb(160, 155, 145);
pub const LOUNGE_FLOOR_BG_EVEN: Color = Color::Rgb(140, 135, 125);
pub const LOUNGE_FLOOR_BG_ODD: Color = Color::Rgb(130, 125, 118);
pub const LOUNGE_FLOOR_CHAR_EVEN: char = '·';
pub const LOUNGE_FLOOR_CHAR_ODD: char = ' ';

// CEO floor — rich dark blue carpet
pub const CEO_FLOOR_FG_EVEN: Color = Color::Rgb(70, 75, 120);
pub const CEO_FLOOR_BG_EVEN: Color = Color::Rgb(55, 58, 95);
pub const CEO_FLOOR_BG_ODD: Color = Color::Rgb(48, 50, 85);
pub const CEO_FLOOR_CHAR_EVEN: char = '·';
pub const CEO_FLOOR_CHAR_ODD: char = ' ';

// Pink/magenta divider — bright and fun
pub const DIVIDER_COLOR: Color = Color::Rgb(240, 170, 200);
pub const DIVIDER_BG: Color = Color::Rgb(200, 130, 165);
pub const DIVIDER_CHAR: char = '░';

// Decorative elements — bright and visible
pub const PLANT_COLOR: Color = Color::Rgb(60, 200, 60);
pub const PLANT_CHAR: char = '♣';
pub const SPARKLE_COLOR: Color = Color::Rgb(230, 220, 120);
pub const SPARKLE_CHAR: char = '✦';

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
pub const CEO_DESK_COLOR: Color = Color::Rgb(160, 105, 55);
pub const CEO_CHAIR_COLOR: Color = Color::Rgb(70, 70, 75);
pub const BULLETIN_BOARD_COLOR: Color = Color::Rgb(200, 165, 100);    // bright cork
pub const BULLETIN_PIN_COLORS: [Color; 4] = [
    Color::Rgb(255, 70, 70),     // red
    Color::Rgb(70, 210, 70),     // green
    Color::Rgb(70, 130, 255),    // blue
    Color::Rgb(255, 230, 50),    // yellow
];
pub const CEO_FRAME_COLOR: Color = Color::Rgb(220, 200, 60);   // gold frame

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
