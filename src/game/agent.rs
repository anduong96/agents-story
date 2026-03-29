use ratatui::style::Color;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AgentStatus {
    Working,
    Idle,
    Spawning,
    Finished,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Room {
    Workspace,
    Lounge,
    CeoOffice,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteColor(pub u8);

impl SpriteColor {
    #[allow(dead_code)]
    pub const GREEN: SpriteColor = SpriteColor(0);
    #[allow(dead_code)]
    pub const CYAN: SpriteColor = SpriteColor(1);
    #[allow(dead_code)]
    pub const MAGENTA: SpriteColor = SpriteColor(2);
    #[allow(dead_code)]
    pub const YELLOW: SpriteColor = SpriteColor(3);
    #[allow(dead_code)]
    pub const BLUE: SpriteColor = SpriteColor(4);
    #[allow(dead_code)]
    pub const RED: SpriteColor = SpriteColor(5);
    #[allow(dead_code)]
    pub const WHITE: SpriteColor = SpriteColor(6);
    #[allow(dead_code)]
    pub const ORANGE: SpriteColor = SpriteColor(7);

    pub const PALETTE: [Color; 8] = [
        Color::Green,
        Color::Cyan,
        Color::Magenta,
        Color::Yellow,
        Color::Blue,
        Color::Red,
        Color::White,
        Color::Rgb(255, 165, 0),
    ];

    pub fn from_index(index: usize) -> Self {
        SpriteColor((index % 8) as u8)
    }

    pub fn to_color(self) -> Color {
        Self::PALETTE[self.0 as usize % 8]
    }

    pub const SKIN_TONES: [Color; 6] = [
        Color::Rgb(240, 200, 150),  // light
        Color::Rgb(220, 180, 130),  // fair
        Color::Rgb(190, 150, 100),  // medium
        Color::Rgb(160, 120, 80),   // olive
        Color::Rgb(130, 90, 60),    // brown
        Color::Rgb(90, 60, 40),     // dark
    ];

    pub fn skin_color(self) -> Color {
        Self::SKIN_TONES[self.0 as usize % Self::SKIN_TONES.len()]
    }
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub repo: String,
    pub branch: String,
    pub worktree: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub model: String,
    pub task: Option<String>,
    pub current_tool: Option<String>,
    pub session: SessionInfo,
    pub tokens: u64,
    pub cost: f64,
    pub started_at: std::time::Instant,
    pub position: (f32, f32),
    pub target_room: Room,
    pub path: Vec<(u16, u16)>,
    pub assigned_desk: Option<usize>,
    pub sprite_color: SpriteColor,
    pub facing: Direction,
    pub is_permanent: bool,
}

impl Agent {
    pub fn new(
        id: String,
        name: String,
        model: String,
        session: SessionInfo,
        color_index: usize,
    ) -> Self {
        Agent {
            id,
            name,
            status: AgentStatus::Spawning,
            model,
            task: None,
            current_tool: None,
            session,
            tokens: 0,
            cost: 0.0,
            started_at: std::time::Instant::now(),
            position: (0.0, 0.0),
            target_room: Room::Workspace,
            path: Vec::new(),
            assigned_desk: None,
            sprite_color: SpriteColor::from_index(color_index),
            facing: Direction::Right,
            is_permanent: false,
        }
    }

    pub fn target_room_for_status(status: &AgentStatus) -> Room {
        match status {
            AgentStatus::Working | AgentStatus::Spawning | AgentStatus::Error => Room::Workspace,
            AgentStatus::Idle | AgentStatus::Finished => Room::Lounge,
        }
    }

    pub fn is_animating(&self) -> bool {
        !self.path.is_empty()
    }
}
