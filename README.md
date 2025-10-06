# doist - Todoist CLI Client

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/hoetaek/doist/ci.yml?branch=main)](https://github.com/hoetaek/doist/actions)

An unofficial [Todoist](https://todoist.com/) API v1 CLI client written in Rust.

## About

<p align="center">
  <img width="1200" src="https://raw.githubusercontent.com/hoetaek/doist/main/vhs/doist.gif">
</p>

This is an unofficial Todoist CLI that focuses on being easy to use. It is
currently not feature complete, but covers some basic common use-cases and adds
more as we go along.

## Installation

Check out the [latest releases](https://github.com/hoetaek/doist/releases) for
various pre-built binaries or follow the following steps:

### Homebrew

For OSX users, a homebrew tap is available:

```bash
brew install hoetaek/tap/doist
```

### Build from source

```bash
# Install Rust first: https://www.rust-lang.org/tools/install
git clone git@github.com:hoetaek/doist
cd doist
cargo build --release
./target/release/doist
```

More options coming eventually.

## How to use

### Auth

First, set up your API token. Go into your [Todoist settings](https://todoist.com/app/settings/integrations),
go to `Integrations` and copy out the `API token`. Plug it into the tool:

```bash
doist auth MY_TOKEN
```

Now you're authenticated and can use the other functions of the tool.

### List tasks

Listing tasks and then working with them interactively is the recommended way to
work with the CLI.

By default the list view shows todays tasks and lets you work with them:

```bash
doist
# Alternatively: `doist list` or `doist l`.
```

This will allow you to type parts of the output until you select the task you
want to work with (fuzzy search). Selecting will allow you to select various
other subcommands, like closing, changing due dates or even editing tasks.

By default, the output is non-interactive and can be piped or used elsewhere. For interactive mode, use:

```bash
doist list --select
# Or for continuous interactive mode:
doist list --interactive
```

By default all interactive commands have a filter applied to show the most
relevant tasks. See the
[documentation](https://todoist.com/help/articles/introduction-to-filters) to
see what inputs are accepted. To then use the filter, add it to the command
parameters:

```bash
doist list --filter "all"
# Alternatively: `doist l -f all`
```

### Interactive (continuous) mode

To continue to use `doist list` with the currently applied filters after each
action (so you can close multiple tasks one after the other for example), a
super-interactive (continuous) mode is also available. This makes the experience
closer to the official app.

```bash
doist list --interactive
# Alternatively: `doist -i`
```

To close out of this mode, press `ESC` during the main list selection.

### Adding tasks

A quick way to add a task is:

```bash
doist add "Do the laundry" --desc "I always forget" --due "tomorrow"
# Alternatively: `doist a "Do the laundry" -D "I always forget" -d tomorrow`
```

Only the task name is required, everything else will assume a default of
*nothing*.

### Interactive task creation

Another way to fully interactively create tasks is:

```bash
doist create
# Alternatively: `doist A`
```

Which will prompt you for the task name and then give you an interactive menu
where you can fill in the details as necessary.

### More about tasks

It's also possible to provide the task with a priority:

```bash
doist add "Party hard" --priority 1
# Alternatively: `doist a "Party hard" -p1`
```

There are several other things you can do to add richer information to a task.
All inputs can be partially provided and will fuzzy match to the closest name
you probably had in mind:

```bash
# Adding project information
doist add "Party hard" --project "personal"
# Alternatively: `doist a "Party hard" -P personal`
```

```bash
# Adding section information. Will automatically attach to the correct project,
# but setting the project will narrow it down.
doist add "Party hard" --section "weekend"
# Alternatively: `doist a "Party hard" -S weekend`
doist add "Party hard" --project personal --section weekend
# Alternatively: `doist a "Party hard" -P personal -S weekend`
```

```bash
# Multiple labels can be provided when creating tasks as well
doist add "Party hard" --label dance --label happy
# Alternatively: `doist a "Party hard" -L dance -L happy`
```

Instead of providing names to be matched, you can also directly provide their
API IDs if you use this tool for automated tooling.

### Closing tasks

A quick way to close one of todays tasks is:

```bash
doist close
# Alternatively: `doist c`
```

And then fuzzy finding the task you want to close. Submitting the ID directly
also works if you're more comfortable with that:

```bash
doist close "BIG_ID_FROM_API"
# Alternatively: `doist c BIG_ID_FROM_API`
```

### View details

To view details of tasks and the comments associated with a task:

```bash
doist view
# Alternatively: `doist v`
```

This accepts the same parameters as `doist list` for task selection.

### Completed tasks

View tasks that you've completed within a date range:

```bash
# View today's completed tasks (default)
doist completed
# Alternatively: `doist comp`
```

Convenient date flags are available:

```bash
doist completed --today           # Today's completed tasks
doist completed --yesterday       # Yesterday's completed tasks
doist completed --this-week       # This week (Mon-today)
doist completed --last-week       # Last week (Mon-Sun)
doist completed --this-month      # This month (1st-today)
```

You can also specify custom date ranges:

```bash
doist completed --since 2025-10-01 --until 2025-10-06
```

By default, tasks are filtered by completion date. To filter by due date instead:

```bash
doist completed --today --by-due-date
```

Completed tasks can be filtered and grouped like regular tasks:

```bash
doist completed --this-week --group-by project
doist completed --today --project work
```

**New in v0.4.1:** Completed tasks now display completion time in a readable format (MM/DD HH:MM). Task IDs are hidden by default for cleaner output - use `--show-id` to display them when needed.

```bash
doist completed --today              # Shows completion time, hides task IDs
doist completed --today --show-id    # Shows both completion time and task IDs
doist list --show-id                 # Also works with list command
```

## Configuration

### Disable colors

If you're not a fan of emojis or colors, you can disable all doist-induced
colors by setting the environment variable `NO_COLOR`:

```bash
NO_COLOR=1 doist
```

### Custom default filter

If you don't like the default filter of `(today | upcoming)`, you can set a
different default filter in the `~/.config/doist/config.toml` like this:

```toml
default_filter="all"
```

See the [Todoist article on filtering](https://todoist.com/help/articles/introduction-to-filters)
for more information.

### Help

Feel free to browse the help output for more help:

```bash
doist help
```
