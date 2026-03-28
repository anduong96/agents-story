/// State for the agent panel sidebar (selection + expand/collapse).
#[derive(Debug, Clone)]
pub struct AgentPanelState {
    pub selected: usize,
    pub expanded: bool,
    pub count: usize,
}

impl AgentPanelState {
    pub fn new() -> Self {
        AgentPanelState {
            selected: 0,
            expanded: false,
            count: 0,
        }
    }

    pub fn select_next(&mut self) {
        if self.count == 0 {
            return;
        }
        self.selected = (self.selected + 1) % self.count;
    }

    pub fn select_prev(&mut self) {
        if self.count == 0 {
            return;
        }
        if self.selected == 0 {
            self.selected = self.count - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn toggle_expand(&mut self) {
        self.expanded = !self.expanded;
    }
}

impl Default for AgentPanelState {
    fn default() -> Self {
        Self::new()
    }
}
