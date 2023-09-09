#![warn(clippy::all, clippy::pedantic, clippy::unwrap_used)]
pub mod test_utils;

#[cfg(test)]
pub mod tui_tests {
    use anyhow::Result;
    use ratatui::{
        backend::TestBackend,
        prelude::{Buffer, Rect},
        Terminal,
    };
    use tod::tui::render;

    #[test]
    fn run_tui() -> Result<()> {
        let backend = TestBackend::new(100, 10);
        let mut terminal = Terminal::new(backend)?;

        render(&mut terminal)?;

        let o: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol.clone())
            .collect();

        // TODO: install nightly and try `assert_matches`
        assert!(o.contains("TUI"));

        Ok(())
    }
}
