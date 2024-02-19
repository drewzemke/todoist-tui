use super::widgets::{items, projects};

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AddingItem,
    SelectingItems,
    SelectingProjects,
    Exiting,
}

pub struct AppState<'a> {
    pub mode: Mode,
    pub projects: projects::State<'a>,
    pub items: items::State<'a>,
}
