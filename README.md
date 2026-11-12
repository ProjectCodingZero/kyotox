# Kyotox

Kyotox is a work-in-progress terminal emulator written in Rust. It spawns a real shell inside a PTY, parses the shell's output as ANSI/VT sequences (`vte`), stores the screen in a scrollback-aware grid, and renders it with [`ratatui`](https://ratatui.rs/).

The project is organized as a Cargo workspace with a reusable core library and separate frontends. Today only the TUI frontend is functional; the GUI and scripting crates are placeholders being prepared for future work.

> **Status:** early development. The TUI renders a shell and forwards keystrokes, but terminal emulation is still incomplete (see [Known limitations](#known-limitations)).

## Workspace layout

```
kyotox/
├── Cargo.toml              # Workspace definition and shared dependencies
├── init.lua                # Example Lua script (not wired up yet)
├── kyotox-core/            # Emulator engine (PTY, parser, grid, config, logging)
│   ├── src/
│   │   ├── pty.rs          # PTY session creation, resize, read/write
│   │   ├── parser.rs       # vte Perform implementation -> EventTerminal events
│   │   ├── term.rs         # Terminal state machine (grid + cursor + PTY + events)
│   │   ├── grid.rs         # Viewport rows and scrollback buffer
│   │   ├── grid/cell.rs    # Cell, Color, CellFlags, underline styles
│   │   ├── cursor.rs       # Cursor position
│   │   ├── config.rs       # TOML configuration loading (sync + async)
│   │   ├── logger.rs       # fern-based file logger
│   │   ├── thread.rs       # Global rayon thread pool
│   │   └── ky_async.rs     # Re-exports of smol primitives
│   └── tests/config.toml   # Example/test configuration
├── kyotox-tui/             # Terminal frontend (ratatui + crossterm)
│   └── src/
│       ├── main.rs         # Entry point, allocator, panic hook, event loop
│       ├── app.rs          # App state, rendering and keyboard input
│       └── clap_cli.rs     # CLI argument parsing
├── kyotox-gui/             # GUI frontend (stub: prints "Hello, world!")
└── kyotox-script/          # Lua scripting support (stub, depends on mlua)
```

## How it works

```
keyboard ──► App::handle_events ──► PTY writer ──► shell
                                                      │
                                                      ▼
ratatui ◄── Grid ◄── Terminal::handle_event ◄── smol channel ◄── VteBridge ◄── vte Parser ◄── PTY reader thread
```

1. `Terminal::new` spawns the default shell with `portable-pty` and starts a dedicated reader thread.
2. The reader feeds raw bytes into a `vte::Parser`, which drives `VteBridge` (a `vte::Perform` implementation).
3. `VteBridge` converts parsed actions into `EventTerminal` values (`PrintChar`, `NewLine`, `ChangeColor`, `ClearScreen`) and sends them over an unbounded `smol` channel.
4. `Terminal::handle_event` drains the channel and updates the `Grid`.
5. `App::draw` renders the grid area with `ratatui` and places the cursor.

Input goes the other way: `App::handle_events` translates `crossterm` key events into bytes (including Ctrl+letter, arrows, Tab, Esc, Enter and Backspace) and writes them to the PTY.

### Core types

- `Grid` — fixed-size viewport of `Row`s plus a bounded scrollback `VecDeque<Row>`.
- `Cell` — a character with optional foreground/background `Color` (`RGB`, `Ascii`, `Indexed`), `CellFlags` (bold, italic, underline variants, wide chars, etc.) and an `UnderlineStyle`.
- `Terminal` — owns the grid, cursor, optional PTY session, event receiver and `TerminalMode` (`Insert`, `Replace`, `Command`).
- `Config` / `CliConfig` — TOML configuration (`debug`, `cli.log`, `cli.scrollback_lines`).

## Getting started

### Requirements

- Rust with edition 2024 support (recent stable toolchain; developed with Rust 1.99)
- A C toolchain, required by `mlua` (`vendored` feature) when building `kyotox-script`
- Linux, macOS or Windows (PTY support is provided by `portable-pty`)

### Configuration

`kyotox-tui` loads `config.toml` from the **current working directory** at startup. The file is intentionally not committed, so create one before running:

```toml
debug = true

[cli]
scrollback_lines = 10000
log = "Trace"
```

`log` accepts `Off`, `Error`, `Warn`, `Info`, `Debug` or `Trace`. If the file is missing, startup currently fails because the loader result is unwrapped in `kyotox-tui/src/main.rs`. See `kyotox-core/tests/config.toml` for a working example.

### Build and run

```bash
# Run the TUI frontend
cargo run -p kyotox-tui

# Choose a log level
cargo run -p kyotox-tui -- --log debug

# Run the test suite
cargo test --workspace

# Lint (CI treats warnings as errors)
cargo clippy --workspace -- -D warnings

# Build everything
cargo build --workspace
```

Logs are written to `logs/<HH:MM:SS DD-MM-YYYY>.log` (ignored by Git).

### CLI

| Option | Short | Values | Default |
| --- | --- | --- | --- |
| `--log` | `-l` | `trace`, `debug`, `info`, `warn`, `error`, `off` | `trace` |

### Keybindings (TUI)

| Key | Action |
| --- | --- |
| Printable characters | Sent to the shell |
| `Ctrl` + letter | Sent as the corresponding control byte |
| `Enter` | Carriage return |
| `Backspace` | `0x7F` (DEL) |
| Arrow keys | ANSI cursor sequences |
| `Tab` | Tab |
| `Esc` | Escape |

## Configuration reference

```toml
# Enable debug behaviour (reserved)
debug = false

[cli]
# Number of rows kept in scrollback
scrollback_lines = 10000

# Log verbosity: Off | Error | Warn | Info | Debug | Trace
log = "Off"
```

Defaults are defined in `kyotox-core/src/config.rs`: `scrollback_lines = 10000`, `log = "Off"`.

## Development

### Testing

Unit tests live next to the code they cover (`pty.rs`, `parser.rs`, `grid.rs`, `config.rs`, `grid/cell.rs`). Run them with:

```bash
cargo test -p kyotox-core
cargo test --workspace
```

Note that the PTY test spawns `bash`, so it expects a Unix-like shell.

### Line endings and CI

`.gitattributes` enforces LF for `*.rs`, `*.sh` and `*.py`, and CRLF for `*.bat`. The Forgejo workflow (`.forgejo/workflows/ci.yml`) checks line endings, builds the workspace, and runs `cargo clippy -- -D warnings`. Changes should keep `cargo clippy` clean.

### Dependency highlights

| Crate | Purpose |
| --- | --- |
| `portable-pty` | Cross-platform PTY handling |
| `vte` | ANSI/VT escape sequence parser |
| `ratatui` + `crossterm` | TUI rendering and input events |
| `smol` | Async runtime and channels |
| `rayon` | Global worker thread pool |
| `serde` + `toml` | Configuration parsing |
| `fern` + `log` | Logging |
| `mimalloc` | Global allocator in the TUI binary |
| `mlua` | Lua scripting (future `kyotox-script`) |

## Known limitations

The project is mid-refactor. Current gaps include:

- **Cursor does not advance** on printed characters; `Terminal::handle_event` writes every `PrintChar` at the same position.
- **Limited escape sequence support.** Only `PrintChar`, newline, a hard-coded color change and clear-screen events are produced; cursor movement, scrolling regions, alternate screen and most SGR attributes are not handled yet.
- **Drawing is a placeholder.** `App::draw` renders a green `"Kyotox"` paragraph over the whole area instead of the grid cells.
- **No scrolling UI or mouse support** yet, despite the scrollback model in `Grid`.
- **Config errors silently fall back to defaults** (`Config::new` hides TOML parse errors), while a missing config file aborts startup.
- **`kyotox-gui` and `kyotox-script` are stubs.** `kyotox-gui` prints `Hello, world!`; `kyotox-script` only contains the default `add` example.
- **Lua integration is not connected.** `init.lua` references an `app` object that does not exist yet.
- **Legacy naming.** Some code, identifiers and UI strings still use the project's former name, *Herrumbre*.

## Roadmap

- Complete the VT parser: cursor movement, erase operations, SGR attributes, scrolling, wide characters
- Render `Grid` cells (colors, attributes) instead of the placeholder paragraph
- Wire up `Text`/`SharedData` input editing and command mode
- Connect the Lua scripting layer (`init.lua`, `app:*` API)
- Finish configuration handling (error reporting, defaults, CLI overrides)
- Build out the GUI frontend

## License

No license file is present in this repository yet. Add one before distributing or accepting external contributions.
