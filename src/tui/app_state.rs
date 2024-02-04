use crate::tui::projects::State;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AddingItem,
    SelectingItems,
    SelectingProjects,
    Exiting,
}

pub struct AppState<'a> {
    pub projects: State<'a>,
    pub mode: Mode,
}
