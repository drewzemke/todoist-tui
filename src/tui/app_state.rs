use crate::tui::projects_pane::ProjectsState;

pub struct AppState<'a> {
    pub projects: ProjectsState<'a>,
}
