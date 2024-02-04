use super::widgets::projects;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AddingItem,
    SelectingItems,
    SelectingProjects,
    Exiting,
}

pub struct AppState<'a> {
    pub projects: projects::State<'a>,
    pub mode: Mode,
}
