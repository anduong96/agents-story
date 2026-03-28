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

// Room wall/door colors
pub const WALL_COLOR: Color = Color::Rgb(70, 70, 85);            // darker for contrast
pub const DOOR_COLOR: Color = Color::Rgb(160, 110, 55);          // warmer door

// Monitor/desk colors
pub const MONITOR_COLOR: Color = Color::Rgb(60, 180, 255);
pub const MONITOR_FLICKER_COLOR: Color = Color::Rgb(40, 140, 200);

// Desk sprite colors
pub const DESK_FRAME_COLOR: Color = Color::Rgb(35, 35, 40);
pub const DESK_SCREEN_OFF_COLOR: Color = Color::Rgb(30, 30, 35);

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

// Workspace floor (rich amber/caramel wood planks)
pub const WORKSPACE_FLOOR_FG_EVEN: Color = Color::Rgb(90, 70, 40);
pub const WORKSPACE_FLOOR_BG_EVEN: Color = Color::Rgb(65, 50, 30);
pub const WORKSPACE_FLOOR_BG_ODD: Color = Color::Rgb(55, 42, 25);
pub const WORKSPACE_FLOOR_CHAR_EVEN: char = '─';
pub const WORKSPACE_FLOOR_CHAR_ODD: char = ' ';

// Lounge floor (cooler gray concrete)
pub const LOUNGE_FLOOR_FG_EVEN: Color = Color::Rgb(95, 95, 100);
pub const LOUNGE_FLOOR_BG_EVEN: Color = Color::Rgb(75, 75, 82);
pub const LOUNGE_FLOOR_BG_ODD: Color = Color::Rgb(65, 65, 72);
pub const LOUNGE_FLOOR_CHAR_EVEN: char = '·';
pub const LOUNGE_FLOOR_CHAR_ODD: char = ' ';

// CEO floor (darker blue)
pub const CEO_FLOOR_FG_EVEN: Color = Color::Rgb(50, 50, 90);
pub const CEO_FLOOR_BG_EVEN: Color = Color::Rgb(30, 30, 60);
pub const CEO_FLOOR_BG_ODD: Color = Color::Rgb(25, 25, 50);
pub const CEO_FLOOR_CHAR_EVEN: char = '─';
pub const CEO_FLOOR_CHAR_ODD: char = ' ';

// Pink/magenta divider
pub const DIVIDER_COLOR: Color = Color::Rgb(230, 150, 180);
pub const DIVIDER_BG: Color = Color::Rgb(180, 100, 140);
pub const DIVIDER_CHAR: char = '░';

// Decorative elements
pub const PLANT_COLOR: Color = Color::Rgb(80, 190, 60);
pub const PLANT_CHAR: char = '♣';
pub const SPARKLE_COLOR: Color = Color::Rgb(200, 200, 100);
pub const SPARKLE_CHAR: char = '✦';

// Vibrant screen pixel colors (Kairosoft style — rainbow of bright colors)
pub const SCREEN_PIXELS: [Color; 10] = [
    Color::Rgb(255, 120, 150),  // pink
    Color::Rgb(120, 220, 120),  // green
    Color::Rgb(100, 180, 255),  // blue
    Color::Rgb(255, 200, 80),   // yellow/gold
    Color::Rgb(255, 140, 60),   // orange
    Color::Rgb(180, 130, 255),  // purple
    Color::Rgb(80, 220, 220),   // cyan
    Color::Rgb(255, 100, 100),  // red
    Color::Rgb(200, 255, 150),  // lime
    Color::Rgb(255, 180, 220),  // light pink
];

// Lounge furniture
pub const COUCH_COLOR: Color = Color::Rgb(160, 100, 50);     // warm brown
pub const COUCH_FRAME_COLOR: Color = Color::Rgb(120, 75, 35); // darker brown frame
pub const COFFEE_TABLE_COLOR: Color = Color::Rgb(140, 90, 45);
pub const VENDING_MACHINE_COLOR: Color = Color::Rgb(80, 130, 200);  // blue
pub const VENDING_LIGHT_COLOR: Color = Color::Rgb(120, 200, 120);   // green light

// CEO office furniture
pub const CEO_DESK_COLOR: Color = Color::Rgb(140, 90, 45);        // rich brown desk
pub const CEO_CHAIR_COLOR: Color = Color::Rgb(60, 60, 65);        // dark chair
pub const BULLETIN_BOARD_COLOR: Color = Color::Rgb(180, 140, 80); // cork board
pub const BULLETIN_PIN_COLORS: [Color; 4] = [
    Color::Rgb(255, 80, 80),   // red pin
    Color::Rgb(80, 200, 80),   // green pin
    Color::Rgb(80, 140, 255),  // blue pin
    Color::Rgb(255, 220, 60),  // yellow pin
];
pub const CEO_FRAME_COLOR: Color = Color::Rgb(200, 180, 50);   // gold frame
