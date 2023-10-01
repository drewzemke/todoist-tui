#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
pub mod test_utils;

#[cfg(test)]
pub mod tui_tests {
    use crate::test_utils::TuiTester;
    use anyhow::Result;
    use crossterm::event::KeyCode;
    use tod::{
        model::{project::Project, Model},
        tui::app::App,
    };

    #[test]
    fn open_and_close_app() -> Result<()> {
        let mut model = Model::default();
        let app = App::new(&mut model);

        TuiTester::new(app, 20, 20)?
            .expect_visible("Inbox")?
            .type_string("q")
            .expect_exiting();

        Ok(())
    }

    #[test]
    fn add_new_todo() -> Result<()> {
        let mut model = Model::default();
        let app = App::new(&mut model);

        TuiTester::new(app, 40, 10)?
            .type_string("a")
            .expect_visible("New Todo")?
            .type_string("new todo text")
            .type_key(KeyCode::Enter)
            .expect_not_visible("New Todo")?
            .expect_visible("new todo text")?;

        assert_eq!(model.get_inbox_items(true).len(), 1);

        Ok(())
    }

    #[test]
    fn complete_todo() -> Result<()> {
        let mut model = Model::default();
        model.add_item("Todo 1");
        model.add_item("Todo 2");
        let app = App::new(&mut model);

        TuiTester::new(app, 40, 10)?
            .expect_visible("Todo 1")?
            .expect_visible("Todo 2")?
            // down arrow to select the first todo
            .type_key(KeyCode::Down)
            // space to complete that item
            .type_key(KeyCode::Char(' '))
            .expect_visible("✓ Todo 1")?
            .expect_visible("- Todo 2")?;

        assert_eq!(model.get_inbox_items(true).len(), 1);

        Ok(())
    }

    #[test]
    fn render_projects() -> Result<()> {
        let mut model = Model::default();
        model.projects.push(Project::new("Project Name!"));
        let app = App::new(&mut model);

        TuiTester::new(app, 40, 10)?.expect_visible("Project Name")?;

        Ok(())
    }
}
