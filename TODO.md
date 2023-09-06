# A todo list for a todo list app, so meta.

## Tasks

### Refactor
- [x] make tests more modular (specifically for mocking)
- [x] build a type to represent the data model / storage
- [x] separate modules for file handling and api reqs
- [x] separate modules for data types (model, items, etc)
- [ ] dry up tests, separate by behavior
- [ ] move stuff out of `main.rs` and into `cli.rs` or something

### Basic Functionality
- [x] complete a todo
  - [ ] attach completed date when completing a todo
- [ ] add/display a todo with a due date
- [x] set the api token

### Interacting with the Sync API
- [x] full sync to read data, store it in a file (along with last sync date)
- [x] update existing flows (or make copies?) to mutate local data rather than do api calls
- [x] incremental sync to update server data

### Negative Scenarios
- [x] missing api token
- [ ] invalid api token (might be covered by the above)
- [ ] cannot reach server
- [ ] server returns error

### Modules 
- [x] `Client`
  - responsible for making/returning requests
  - owns sync url
  - encapsulates reqwest logic
- [x] `FileManager` 
  - owns data and config locations
  - resposible for reading/writing from both locations

### Misc.
- [x] add stricter clippy, including `unwrap` (replace `unwraps` with allowed `expects` in tests)
- [x] rename this app :)