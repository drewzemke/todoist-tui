#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
pub mod test_utils;

#[cfg(test)]
pub mod tui_tests {
    use crate::test_utils::TuiTester;
    use anyhow::Result;
    use tod::{model::Model, tui::app::App};

    #[test]
    fn open_and_close_app() -> Result<()> {
        let mut model = Model::default();
        let app = App::new(&mut model);

        TuiTester::new(app, 20, 20)?
            .expect_visible("Inbox")?
            .press_keys("q")
            .expect_exiting();

        Ok(())
    }

    #[test]
    fn add_new_todo() -> Result<()> {
        let mut model = Model::default();
        let app = App::new(&mut model);

        TuiTester::new(app, 30, 10)?
            .press_keys("a")
            .expect_visible("New Todo")?
            .press_keys("new todo text")
            .press_enter()
            .expect_not_visible("New Todo")?
            .expect_visible("new todo text")?;

        Ok(())
    }
}
