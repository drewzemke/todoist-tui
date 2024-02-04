use crate::tui::projects_pane::ProjectsState;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AddingItem,
    SelectingItems,
    SelectingProjects,
    Exiting,
}

pub struct AppState<'a> {
    pub projects: ProjectsState<'a>,
    pub mode: Mode,
}
