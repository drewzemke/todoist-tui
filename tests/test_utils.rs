#[cfg(test)]
pub use wiremock_wrapper::ApiMockBuilder;

#[cfg(test)]
mod wiremock_wrapper {
    use serde::{Deserialize, Serialize};
    use wiremock::{
        matchers::{self, header_regex},
        Mock, MockServer, Request, ResponseTemplate,
    };

    pub struct ApiMockBuilder {
        mock_server: MockServer,
    }

    impl ApiMockBuilder {
        pub async fn new() -> Self {
            ApiMockBuilder {
                mock_server: MockServer::start().await,
            }
        }

        // HACK: Not sure if the typing on `F` here is all necessary, or if there's a way around the `Clone` constraint
        pub async fn mock_response<F, T, R>(self, path: &str, condition: F, response: R) -> Self
        where
            F: Fn(T) -> bool + Send + Sync + Clone + 'static,
            T: for<'de> Deserialize<'de>,
            R: Serialize,
        {
            let matcher =
                move |request: &Request| request.body_json::<T>().is_ok_and(condition.clone());
            let response = ResponseTemplate::new(200).set_body_json(response);
            Mock::given(matchers::path(path))
                .and(matcher)
                // TODO: Make this configurable?
                .and(header_regex("Authorization", "MOCK_API_TOKEN"))
                .respond_with(response)
                .mount(&self.mock_server)
                .await;
            self
        }

        #[must_use]
        pub fn uri(&self) -> String {
            self.mock_server.uri()
        }
    }
}

#[cfg(test)]
pub use assert_fs_wrapper::FsMockBuilder;

#[cfg(test)]
mod assert_fs_wrapper {
    use anyhow::Result;
    use assert_fs::{
        prelude::{FileTouch, FileWriteStr, PathChild},
        TempDir,
    };
    use std::{fmt::Display, path::Path};

    pub struct FsMockBuilder {
        mock_dir: TempDir,
    }

    impl FsMockBuilder {
        /// # Errors
        ///
        /// Returns an error if the mock directory cannot be created.
        pub fn new() -> Result<Self> {
            let mock_dir = TempDir::new()?;
            Ok(FsMockBuilder { mock_dir })
        }

        /// # Errors
        ///
        /// Returns an error if the mock directory cannot be written to.
        pub fn mock_file_contents<T>(self, path: &str, contents: T) -> Result<Self>
        where
            T: Display,
        {
            let mock_path = self.mock_dir.child(path);
            mock_path.touch()?;
            mock_path.write_str(contents.to_string().as_str())?;
            Ok(self)
        }

        #[must_use]
        pub fn path(&self) -> &Path {
            self.mock_dir.path()
        }
    }
}

#[cfg(test)]
pub use tui_tester::TuiTester;

#[cfg(test)]
mod tui_tester {
    use anyhow::Result;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{backend::TestBackend, buffer::Cell, Terminal};
    use std::fmt::Write;
    use tod::tui::app::App;
    use tod::tui::app_state::Mode;

    // TODO: make this generic? just for fun I guess
    pub struct TuiTester<'a> {
        terminal: Terminal<TestBackend>,
        app: App<'a>,
    }

    impl<'a> TuiTester<'a> {
        /// Make a new tester.
        ///
        /// # Errors
        /// Returns an error if the test terminal cannot be initialized.
        pub fn new(app: App<'a>, width: u16, height: u16) -> Result<Self> {
            let terminal = Terminal::new(TestBackend::new(width, height))?;
            Ok(Self { terminal, app })
        }

        /// Renders the buffer and asserts that the given string is visible.
        ///
        /// # Panics
        /// If `needle` cannot be found in the current buffer.
        ///
        /// # Errors
        /// If something goes wrong while drawing to the screen.
        ///
        /// # Note
        /// This currently fails to find strings that are broken up by line breaks
        pub fn expect_visible(&mut self, needle: &str) -> Result<&mut Self> {
            let screen = self.render_to_string()?;
            assert!(
                screen.contains(needle),
                "The string '{needle}' was not found on this screen:\n{screen}"
            );
            Ok(self)
        }

        /// Renders the buffer and asserts that the given string is *not* visible.
        ///
        /// # Panics
        /// If `needle` is present be found in the current buffer.
        ///
        /// # Errors
        /// If something goes wrong while drawing to the screen.
        ///
        /// # Note
        /// This currently fails to find strings that are broken up by line breaks
        pub fn expect_not_visible(&mut self, needle: &str) -> Result<&mut Self> {
            let screen = self.render_to_string()?;
            assert!(
                !screen.contains(needle),
                "The string '{needle}' was not expected on this screen:\n{screen}"
            );
            Ok(self)
        }

        fn render_to_string(&mut self) -> Result<String> {
            self.terminal.draw(|frame| {
                self.app.render(frame);
            })?;
            let width = self.terminal.backend().buffer().area.width as usize;

            let screen = self
                .terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .enumerate()
                .fold(String::new(), |mut string, (index, cell)| {
                    let _ = write!(string, "{}", Cell::symbol(cell));
                    if (index + 1) % width == 0 {
                        let _ = writeln!(string);
                    }
                    string
                });
            Ok(screen)
        }

        /// Assert that the app is in an exiting state.
        ///
        /// # Panics
        /// If it isn't.
        pub fn expect_exiting(&self) {
            assert_eq!(self.app.state.mode, Mode::Exiting);
        }

        /// Sends the characters in the given string as individual keypresses to the app.
        /// Note that this does not render the app in between keypresses.
        pub fn type_string(&mut self, keys: &str) -> &mut Self {
            keys.chars().for_each(|c| {
                self.app
                    .handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
            });
            self
        }

        /// Sends a enter key press to the app.
        pub fn type_key(&mut self, key: KeyCode) -> &mut Self {
            self.app.handle_key(KeyEvent::new(key, KeyModifiers::NONE));
            self
        }
    }
}
