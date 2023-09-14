#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
pub mod test_utils;

#[cfg(test)]
pub mod tui_tests {
    use anyhow::Result;
    use ratatui::{backend::TestBackend, Terminal};
    use tod::{model::Model, tui::render};

    #[test]
    fn run_tui() -> Result<()> {
        let backend = TestBackend::new(100, 100);
        let mut terminal = Terminal::new(backend)?;
        let mut model = Model::default();

        // TODO: this renders the screen, but how do we test interactivity?
        render(&mut terminal, &mut model)?;

        // TODO: throw in new lines and other chars to get this to pretty print?
        let o: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol.clone())
            .collect();

        // TODO: write a wrapper around a line like the one below that prints the buffer if the check fails
        assert!(
            o.contains("Inbox"),
            "The string was not found in this buffer:\n {o}"
        );

        Ok(())
    }
}
