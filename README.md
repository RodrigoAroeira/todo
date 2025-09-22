# Tick

<!--toc:start-->
- [Tick](#tick)
  - [New features](#new-features)
  - [Differences](#differences)
  - [Quick Start](#quick-start)
<!--toc:end-->

Tick is a terminal-based TODO application inspired by [todo-rs](https://github.com/tsoding/todo-rs).
While maintaining the original's core functionality, Tick eliminates the `ncurses`
dependency and uses a manual rendering approach instead of the immediate tui layout.
This simplifies code and makes UI easier to maintain

It uses the exact same TODO file structure as todo-rs, so existing files work
seamlessly. Tick also introduces a few added functionalities and improvements in
usability.

## New features

- Quit without saving
- Line splitting so items don't overlap
- Output file defaults to `$HOME/TODO` if no file argument is provided

## Differences & Highlights

- <kbd>r</kbd> was changed to <kbd>e</kbd> for "edit"
- As of now there is no notification bar
- Manual rendering for simpler maintainable UI
- No ctrlc handling

## Quick Start

```bash
cargo run TODO
```

or

```bash
cargo run
```
