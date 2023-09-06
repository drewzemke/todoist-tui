# `tod-ui`, a Todoist TUI & CLI App 

## Installation

I'll eventually get around to hosting this on crates.io. For now, you can clone the repo 
and manually build if you have the Rust toolchain installed.

```shell
git clone git@github.com/drewzemke/tod-ui.git
cd tod-ui
cargo install --path .
```


## CLI Usage

```shell
# Get your API token from the Todoist web app first, then store it:
tod set-token [your_api_key]

# Sync your data with Todoist's servers:
tod sync

# Add some todos to your inbox:
tod add "Do a barrel roll!"
tod add "Use the boost to get through!"

# List the contents of your inbox:
tod list
> [1] "Do a barrel roll!"
> [2] "Use the boost to get through!"

# Mark a todo complete using its number in the list:
tod complete 2
```
