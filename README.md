# Drew's Rust Todoist Client / TUI App 

## CLI Usage

```shell
# Get your API token from the Todoist web app first, then store it:
todoist set-token [your_api_key]

# Sync your data with Todoist's servers:
todoist sync

# Add some todos to your inbox:
todoist add "Do a barrel roll!"
todoist add "Use the boost to get through!"

# List the contents of your inbox:
todoist list
> [1] "Do a barrel roll!"
> [2] "Use the boost to get through!"

# Mark a todo complete using its number in the list:
todoist complete 2
```

---

## Just my notes and task tracking, please ignore

### Next up:
- [ ] refactor!
  - [x] bring in `anyhow`
  - [x] build a type to represent the data model / storage
  - [x] separate modules for file handling and api reqs
  - [x] separate modules for data types (model, items, etc)
  - [ ] dry up tests, separate by behavior

### new functionality
- [x] complete a todo
  - [ ] attach completed date when completing a todo
- [ ] add/display a todo with a due date
- [ ] find a better name for this app ._.

### e2e flows -- positive scenarios
- [x] add a todo when user data (namely, inbox id) is already stored
- [x] add a todo when user data doesn't exist (so we need to get it!)
- [x] set the api token

### get syncin'
- [x] full sync to read data, store it in a file (along with last sync date)
- [x] update existing flows (or make copies?) to mutate local data rather than do api calls
- [x] incremental sync to update server data

### negative scenarios
- [x] missing api token
- [ ] cannot reach server
- [ ] server returns error
- [ ] invalid api token (may be covered by the above)

### refactors
- [x] refactor tests to be more modular (specifically for mocking)
- [ ] move stuff out of `main.rs` and into `cli.rs` or something

### other todos
- [x] add stricter clippy, including `unwrap` (replace `unwraps` with allowed `expects` in tests)
- [ ] better test organization

---

## Abstraction Ideas

### `Client` [DONE]
- responsible for making/returning requests
- owns sync url
- encapsulates reqwest logic

### `FileManager` [IN PROGRESS]
- owns data and config locations
- resposible for reading/writing from both locations
