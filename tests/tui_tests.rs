#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
pub mod test_utils;

#[cfg(test)]
pub mod tui_tests {
    #![allow(clippy::unwrap_used)]

    use crate::test_utils::TuiTester;
    use anyhow::Result;
    use chrono::{NaiveDate, NaiveDateTime};
    use crossterm::event::KeyCode;
    use tod::{
        model::{
            item::{Due, DueDate, Item},
            project::Project,
            Model,
        },
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
    fn add_new_todo_to_project() -> Result<()> {
        let mut model = Model::default();
        let project = Project::new("Project");
        let project_id = project.id.clone();
        let item1 = Item::new("Item 1", &project.id);
        let item2 = Item::new("Item 2", &project.id);
        model.projects.push(project);
        model.items.push(item1);
        model.items.push(item2);
        let app = App::new(&mut model);

        TuiTester::new(app, 80, 10)?
            .type_key(KeyCode::Tab)
            // down to select the first project
            .type_key(KeyCode::Down)
            .type_string("a")
            .expect_visible("New Todo")?
            .expect_visible("enter: add todo")?
            .expect_visible("escape: cancel")?
            .type_string("new todo text")
            .type_key(KeyCode::Enter)
            .expect_not_visible("New Todo")?
            .expect_visible("new todo text")?;

        assert_eq!(model.get_items_in_project(&project_id).len(), 3);

        Ok(())
    }

    #[test]
    fn add_new_todo_with_smart_date() -> Result<()> {
        let mut model = Model::default();
        let today = NaiveDate::parse_from_str("2023-10-08", "%Y-%m-%d").unwrap();
        let app = App::new_with_date(&mut model, today);

        TuiTester::new(app, 70, 10)?
            .type_string("a")
            .expect_visible("New Todo")?
            .type_string("buy potatoes tomorrow")
            .type_key(KeyCode::Enter)
            .expect_not_visible("New Todo")?
            // check that the new item is visible _without_ the tomorrow prefix
            .expect_visible("buy potatoes")?
            .expect_not_visible("buy potatoes tomorrow")?
            .expect_visible("2023-10-09")?;

        assert_eq!(model.get_inbox_items(true).len(), 1);

        Ok(())
    }

    #[test]
    fn complete_todo() -> Result<()> {
        let mut model = Model::default();
        model.add_item_to_inbox("Todo 1", None);
        model.add_item_to_inbox("Todo 2", None);
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
            .expect_visible("Item 2")?
            // tab and down to select the only item in the second project
            .type_key(KeyCode::Tab)
            .type_key(KeyCode::Down)
            // space to complete it
            .type_key(KeyCode::Char(' '))
            .expect_visible("✓ Item 2")?;

        Ok(())
    }

    #[test]
    fn show_key_hints() -> Result<()> {
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

        TuiTester::new(app, 100, 10)?
            // key hints in select item mode
            .expect_visible("q: quit")?
            .expect_visible("tab: change focus")?
            .expect_visible("↑↓: select")?
            .expect_visible("a: new todo")?
            .expect_visible("space: mark complete")?
            // key hints in add item mode
            // tab to move focus to the projects panel
            .type_string("a")
            .expect_visible("New Todo")?
            .expect_visible("enter: add todo")?
            .expect_visible("escape: cancel")?;

        Ok(())
    }

    #[test]
    fn show_due_dates_and_times() -> Result<()> {
        let mut model = Model::default();
        model.add_item_to_inbox(
            "Todo 1",
            Some(Due {
                date: DueDate::Date(
                    NaiveDate::parse_from_str("2011-11-12", "%Y-%m-%d").expect("parse date"),
                ),
            }),
        );

        model.add_item_to_inbox(
            "Todo 2",
            Some(Due {
                date: DueDate::DateTime(
                    NaiveDateTime::parse_from_str("2011-10-14 3:48", "%Y-%m-%d %H:%M")
                        .expect("parse datetime"),
                ),
            }),
        );

        let app = App::new(&mut model);

        TuiTester::new(app, 60, 10)?
            .expect_visible("Todo 1")?
            .expect_visible("2011-11-12")?
            .expect_visible("Todo 2")?
            .expect_visible("2011-10-14 03:48")?;

        Ok(())
    }
}
