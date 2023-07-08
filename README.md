# Drew's Rust Todoist Client / TUI App 
Woo! 

---

## Just my notes and task tracking, please ignore

### e2e flows -- positive scenarios
- [x] add a todo when user data (namely, inbox id) is already stored
  - reads from fs
  - one api call

- [x] add a todo when user data doesn't exist (so we need to get it!)
  - reads from and writes to fs
  - two api calls

- [ ] add a todo when there's no api key, so we need to prompt for it?

### negative scenarios
- [ ] invalid api key
- [ ] cannot reach server
- [ ] server returns error

### refactors
- [ ] move stuff out of `main.rs` and into `cli.rs` or something
- [ ] refactor tests to be more module (specifically for mocking)

### other todos
- [ ] add stricter clippy, including `unwrap` (replace `unwraps` with allowed `expects` in tests)
- [ ] figure out how to organize tests so that we can select when to run/not run e2e's

