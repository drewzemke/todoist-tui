use super::widgets::{items, projects};

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AddingItem,
    SelectingItems,
    SelectingProjects,
    Exiting,
}

pub struct AppState {
    pub mode: Mode,
    pub projects: projects::State,
    pub items: items::State,
}
