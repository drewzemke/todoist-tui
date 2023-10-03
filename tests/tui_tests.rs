#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
pub mod test_utils;

#[cfg(test)]
pub mod tui_tests {
    use crate::test_utils::TuiTester;
    use anyhow::Result;
    use crossterm::event::KeyCode;
    use tod::{
        model::{item::Item, project::Project, Model},
        tui::app::App,
    };

    #[test]
    fn open_and_close_app() -> Result<()> {
        let mut model = Model::default();
        let app = App::new(&mut model);

        TuiTester::new(app, 20, 20)?
            .expect_visible("Inbox")?
            // regression: press down arrow twice in an empty inbox caused a crash
            .type_key(KeyCode::Down)
            .type_key(KeyCode::Down)
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
    fn view_items_in_different_projects() -> Result<()> {
        let mut model = Model::default();
        let project1 = Project::new("Project 1");
        let project2 = Project::new("Project 2");
        let item1 = Item::new("Item 1", &project1.id);
        let item2 = Item::new("Item 2", &project2.id);
        model.projects.push(project1);
        model.projects.push(project2);
        model.items.push(item1);
        model.items.push(item2);
        let app = App::new(&mut model);

        TuiTester::new(app, 40, 10)?
            .expect_visible("Project 1")?
            .expect_visible("Project 2")?
            // tab to move focus to the projects panel
            .type_key(KeyCode::Tab)
            // down to select the first project
            .type_key(KeyCode::Down)
            .expect_visible("Item 1")?
            // down to select the second project
            .type_key(KeyCode::Down)
            .expect_visible("Item 2")?;

        Ok(())
    }
}
