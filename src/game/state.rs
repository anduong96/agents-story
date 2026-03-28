use crate::game::agent::{Agent, AgentStatus};
use crate::game::floor::Floor;

#[derive(Debug, Clone)]
pub struct Stats {
    pub model: String,
    pub active_agents: usize,
    pub total_agents: usize,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub usage_percent: f32,
    pub fps: u64,
    pub ram_mb: f64,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            model: "unknown".to_string(),
            active_agents: 0,
            total_agents: 0,
            completed_tasks: 0,
            total_tasks: 0,
            total_tokens: 0,
            total_cost: 0.0,
            usage_percent: 0.0,
            fps: 0,
            ram_mb: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CeoStatus {
    Idle,
    PromptSent,
    Waiting,
    AllComplete,
    Error,
}

#[derive(Debug)]
pub struct GameState {
    pub floor: Floor,
    pub agents: Vec<Agent>,
    pub stats: Stats,
    pub ceo_status: CeoStatus,
    pub next_color_index: usize,
}

impl GameState {
    pub fn new(width: u16, height: u16) -> Self {
        let mut floor = Floor::generate(width, height);
        floor.ensure_minimum_desks();
        GameState {
            floor,
            agents: Vec::new(),
            stats: Stats::default(),
            ceo_status: CeoStatus::Idle,
            next_color_index: 0,
        }
    }

    pub fn has_animations(&self) -> bool {
        self.agents.iter().any(|a| a.is_animating())
    }

    pub fn update_stats(&mut self) {
        let active = self
            .agents
            .iter()
            .filter(|a| {
                matches!(
                    a.status,
                    AgentStatus::Working | AgentStatus::Spawning
                )
            })
            .count();

        let total_tokens: u64 = self.agents.iter().map(|a| a.tokens).sum();
        let total_cost: f64 = self.agents.iter().map(|a| a.cost).sum();

        self.stats.active_agents = active;
        self.stats.total_agents = self.agents.len();
        self.stats.total_tokens = total_tokens;
        self.stats.total_cost = total_cost;
    }
}
