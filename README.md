# wuu

[![CI](https://github.com/dualys/wuu/actions/workflows/ci.yml/badge.svg)](https://github.com/dualys/wuu/actions/workflows/ci.yml)

`wuu` is a `no_std` terminal UI library for operating systems and VCS tools. It
provides formatted terminal output, status messages, tables, diffs, progress
bars, interactive prompts, cursor controls, and a server dashboard renderer.
Its UI data structures and formatting helpers avoid heap allocation; the
terminal backend and your OS may have their own allocation requirements.

The core library does not own your input loop. Prompts are event-driven: the
host application passes key events to `wuu`. The optional `linux-keyboard`
feature supplies a Linux raw-terminal reader; an OS can instead use the
allocation-free ASCII/ANSI decoder or pass its own keyboard events.

## Runnable examples

Run the VCS output and prompt demonstrations with:

```sh
cargo run --example vcs_output
cargo run --example prompts --features linux-keyboard
```

The OS dashboard example creates a temporary in-memory framebuffer, renders a
dashboard into it, and reports how many bytes changed:

```sh
cargo run --example os_dashboard
```

In an OS, replace that temporary buffer with the framebuffer supplied by your
bootloader or display driver, then keep the memory alive for as long as the
terminal is using it. The examples are also checked by CI with
`cargo check --examples --all-features --locked`.

## Setup

Add `wuu` to your dependencies:

```toml
[dependencies]
wuu = "0.1"
```

`wuu` can be used from a `no_std` crate. Initialize
[`TERMINAL`](src/framebuffer.rs) with your OS's `os-terminal` instance before
calling terminal print helpers. For code that does not have a configured
terminal, the `write_to` APIs render to any `core::fmt::Write` destination.

## Command-line arguments

[`Command`](src/console.rs) and [`Arg`](src/console.rs) provide a declarative
command-line parser without heap allocation. In a `no_std` application, the OS
or bootloader must provide and tokenize the arguments; `wuu` does not read
process arguments itself. Pass arguments without the executable name:

```rust
use wuu::{Arg, Command};

let definitions = [
    Arg::new("verbose")
        .short('v')
        .long("verbose")
        .help("Show detailed output"),
    Arg::new("config")
        .short('c')
        .long("config")
        .takes_value("FILE")
        .required()
        .help("Configuration file"),
    Arg::new("input").takes_value("INPUT"),
];
let command = Command::new("wuu")
    .about("A no_std command-line application")
    .args(&definitions);
let argv = ["-v", "--config=app.toml", "source.wuu"];
let matches = command.parse(&argv)?;

assert!(matches.flag("verbose"));
assert_eq!(matches.value("config"), Some("app.toml"));
assert_eq!(matches.value("input"), Some("source.wuu"));
```

The parser supports long options, short options (including combined flags),
`--name value` and `--name=value`, positional arguments, required arguments,
and `--` to end option parsing. `Command::write_help` writes usage and argument
help to any `core::fmt::Write` destination. Definitions are borrowed, and a
command supports up to `MAX_CONSOLE_ARGS` (32) arguments. For simple raw
argument access, `Console::new` remains available.

## Formatted output and macros

The `print!` and `println!` macros write formatted text to the configured
terminal. The `info!`, `success!`, `warning!`, and `error!` macros render
OpenRC-style status lines, padded to the terminal width and colored without
allocating:

```rust
wuu::println!("wuu VCS starting");
wuu::info!("Scanning {} files", 12);
wuu::success!("Commit {} created", "a1b2c3d");
wuu::warning!("{} files are unstaged", 2);
wuu::error!("Could not open {}", "config.wuu");
```

The output looks like this (the marker and status are colored on an ANSI
terminal; dot padding follows the terminal width):

```text
wuu VCS starting
* Scanning 12 files .................. [ MESSAGE ]
* Commit a1b2c3d created ............. [ SUCCESS ]
! 2 files are unstaged ............... [ WARNING ]
! Could not open config.wuu .......... [ FAILURE ]
```

`info!`, `success!`, `warning!`, and `error!` use the statuses `[ MESSAGE ]`,
`[ SUCCESS ]`, `[ WARNING ]`, and `[ FAILURE ]` respectively. Informational and
successful lines use the `*` marker; warnings and errors use `!`. Plain
`println!` output remains unchanged.

[`format_stack!`](src/stack_string.rs) formats into a fixed 256-byte buffer:

```rust
let line = wuu::format_stack!("branch: {} ({})", "main", "clean");
wuu::println!("{}", line.as_str());
```

The buffer truncates at its byte capacity. If truncation splits a UTF-8
character, `as_str()` returns an empty string.

For more control, write to another formatter or choose whether to emit ANSI
colors:

```rust
use wuu::{ColorMode, MessageLevel, print_styled_message, write_message};

let mut text = String::new();
write_message(&mut text, MessageLevel::Info, "Working tree clean").unwrap();
assert_eq!(text, "[INFO] Working tree clean\n");

print_styled_message(MessageLevel::Success, "Commit created", ColorMode::Ansi);
print_styled_message(MessageLevel::Warning, "No remote configured", ColorMode::Plain);
```

## OpenRC-style status lines

Use `ok()` and `ko()` to print successful and failed task lines. The marker and
status are colored, and dots fill the available terminal width. The custom
variants replace the default `ok` or `ko` label:

```rust
wuu::ok("Starting network");
wuu::ko("Loading configuration");
wuu::ok_custom("Starting database", "ready");
wuu::ko_custom("Checking configuration", "invalid");
```

This produces lines in the following style:

```text
* Starting network ......................... [ ok ]
! Loading configuration .................... [ ko ]
* Starting database ..................... [ ready ]
! Checking configuration ............... [ invalid ]
```

The `write_ok`, `write_ko`, `write_ok_custom`, and `write_ko_custom` variants
write to any `core::fmt::Write` destination and accept an explicit width and
color mode.

## Tables

Tables borrow their headers and rows. `print_auto()` uses the configured
terminal width; `print_width()` and `write_to_width()` accept a width directly:

```rust
use wuu::Table;

let headers = ["CHANGE", "STATUS", "PATH"];
let staged = ["A", "added", "src/commit.rs"];
let modified = ["M", "modified", "src/lib.rs"];
let rows: [&[&str]; 2] = [&staged, &modified];

Table::new(&headers, &rows).print_auto();
```

Example:

```text
+--------+----------+---------------+
| CHANGE | STATUS   | PATH          |
+--------+----------+---------------+
| A      | added    | src/commit.rs |
| M      | modified | src/lib.rs    |
+--------+----------+---------------+
```

Long paths are shortened from the left to preserve their ending, while other
cells are shortened from the right:

```rust
let headers = ["STATUS", "PATH"];
let row = ["modified", "src/very/long/path/to/file.rs"];
let rows: [&[&str]; 1] = [&row];
let table = Table::new(&headers, &rows);
table.print_width(30);
```

An abbreviated result:

```text
+----------+-----------------+
| STATUS   | PATH            |
+----------+-----------------+
| modified | ~/th/to/file.rs |
+----------+-----------------+
```

For custom output, call `write_to()` or `write_to_width(&mut output, width)`.
Cell widths count Unicode scalar values; use single-column characters for
predictable alignment. If there are more columns than can fit even at their
minimum width, the table may exceed the requested width.

Choose a border preset with `with_style()`:

```rust
use wuu::{Table, TableStyle};

let headers = ["HASH", "MESSAGE"];
let first = ["a1b2c3d", "Add rounded table styles"];
let rows: [&[&str]; 1] = [&first];

Table::new(&headers, &rows)
    .with_style(TableStyle::Rounded)
    .print_auto();
```

Available presets are `Ascii` (the default), `Modern`, `Rounded`, `Double`,
and `Transparent`. Transparent tables have no side borders and use a single
rule under the header:

```text
HASH     MESSAGE
───────  ────────────────────────
a1b2c3d  Add rounded table styles
```

## Diffs

`Diff` renders unified-diff text. Added lines are green, removed lines red,
and hunk headers cyan in ANSI mode. File headers (`+++` and `---`) remain
uncolored:

```rust
use wuu::{ColorMode, Diff};

let patch = "\
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,2 +1,2 @@
-let old_name = \"wuu\";
+let new_name = \"wuu\";
";
Diff::new(patch).print(ColorMode::Ansi);
```

Call `write_to(&mut output, ColorMode::Plain)` for plain output or to render to
a custom `core::fmt::Write` destination.

## Progress bars

Use `set_position()` when a task reports completed units, or `inc()` when each
iteration completes one unit:

```rust
use wuu::ProgressBar;

let mut progress = ProgressBar::with_message(3, "Fetching");
progress.set_width(12);

for completed in 1..=3 {
    // Fetch one object.
    progress.set_position(completed);
    progress.print();
}
progress.finish();
```

The final display is:

```text
Fetching [============] 100%
```

`write_to()` and `write_finish()` render to a custom formatter.
`set_position()` clamps to the total; a total of zero is considered complete.

## Interactive prompts

Prompts are event-driven. Convert keys from your OS or application event loop
to `PromptKey`, call `handle_key()`, then redraw with `print()` or `write_to()`.
`Escape` cancels any prompt. On Linux, enable the `linux-keyboard` feature and
use `keyboard::linux::LinuxKeyboard` to read actual key presses in raw mode.

`PromptKey::from_ascii()` maps one ASCII byte using `ASCII_KEY_TABLE`.
`AsciiKeyDecoder` incrementally decodes ANSI/SS3 navigation and editing keys:
arrows, Home, End, Delete, Page Up/Down, and Tab. It also maps ASCII control
bytes to `PromptKey::Control`. This is useful for serial terminals or an OS
keyboard driver that translates keys into ASCII bytes. Escape-prefixed
printable characters are reported as `PromptKey::Alt`. Call
`flush_escape()` after the OS's Escape timeout to distinguish an Escape press
from the prefix of an ANSI sequence. Alternatively, `PromptKey::from_ansi()`
recognizes complete sequences and `Prompt::handle_ansi()` dispatches them.

Try the selectable branch example with:

```sh
cargo run --example prompts --features linux-keyboard
```

It uses raw key input: move with the arrow keys, select with Enter, or cancel
with Escape. The blinking terminal cursor is hidden while the prompt is active
and restored afterward; the `>` selection marker remains visible. The Linux
adapter restores the terminal mode when it is dropped.
An OS can pass `PromptKey::Up`, `PromptKey::Down`, and other events directly;
it does not need to enable the Linux feature.

### Text input

```rust
use wuu::{Prompt, PromptKey, PromptResult};

let mut prompt = Prompt::text("Commit message");
for ch in "Fix".chars() {
    prompt.handle_key(PromptKey::Character(ch));
}
prompt.handle_key(PromptKey::Enter);

if let Some(PromptResult::Text(message)) = prompt.result() {
    // Store the commit message.
    assert_eq!(message, "Fix");
}
```

Text input holds up to `TEXT_CAPACITY` (256) bytes. Left/Right move the
insertion cursor; Home/End move to the beginning or end; Backspace and Delete
remove the previous or next Unicode character. Control-A/E move to the
beginning/end, Control-U/K remove text to the left/right, and Control-W removes
the previous word. Control-C cancels a prompt.

### Confirmation

Confirmations default to no. Toggle the answer with `Toggle` or enter `y`/`n`,
then accept with `Enter`:

```rust
use wuu::{Prompt, PromptKey, PromptResult};

let mut prompt = Prompt::confirm("Delete branch 'old'?");
prompt.handle_key(PromptKey::Character('y'));
prompt.handle_key(PromptKey::Enter);

if let Some(PromptResult::Confirm(yes)) = prompt.result() {
    assert!(yes);
}
```

### Single choice

```rust
use wuu::{Prompt, PromptKey, PromptResult};

let branches = ["main", "release", "feature"];
let mut prompt = Prompt::select("Switch to branch", &branches).unwrap();
prompt.handle_key(PromptKey::Down);
prompt.handle_key(PromptKey::Enter);

if let Some(PromptResult::Selected(index)) = prompt.result() {
    assert_eq!(branches[index], "release");
}
```

### Multiple choices

Multi-select uses a caller-owned boolean slice with one flag per option:

```rust
use wuu::{Prompt, PromptKey, PromptResult};

let actions = ["stage", "commit", "push"];
let mut selected = [false; 3];
let mut prompt = Prompt::multi_select("Choose actions", &actions, &mut selected).unwrap();
prompt.handle_key(PromptKey::Toggle);
prompt.handle_key(PromptKey::Down);
prompt.handle_key(PromptKey::Toggle);
prompt.handle_key(PromptKey::Enter);

if let Some(PromptResult::Multiple(choices)) = prompt.result() {
    assert_eq!(choices, &[true, true, false]);
}
```

`Up`/`Down` and Left/Right move through list items; Home/End select the first or
last item and Page Up/Down move by ten. `Toggle` or a space character
toggles the highlighted item. Invalid empty lists and selection lengths return
`PromptError`.

### Selecting a `Noun`

[`Noun`](https://crates.io/crates/noun) is a 32-byte BLAKE3 content identifier.
`noun_select()` chooses from identifiers already supplied by the caller; it
does not read or hash files. By default it displays the first eight bytes
(16 hexadecimal characters):

```rust
use wuu::{Noun, Prompt, PromptKey, PromptResult};

let objects = [Noun::of(b"first file"), Noun::of(b"second file")];
let mut prompt = Prompt::noun_select("Choose object", &objects)
    .unwrap()
    .noun_prefix_len(12);
prompt.handle_key(PromptKey::Down);
prompt.handle_key(PromptKey::Enter);

if let Some(PromptResult::Noun(noun)) = prompt.result() {
    assert_eq!(noun, &objects[1]);
}
```

`noun_prefix_len()` selects how many leading bytes to display (0–32).

## Dashboard and drawing

`Wuu` renders a character-cell server dashboard when given a `Context::Server`.
Panels have a title, body, status, and optional contextual terminal:

```rust
use wuu::{Context, DropDownTerm, Panel, Status, Wuu};

let panels = [
    Panel::new(
        "Working tree",
        "2 modified\n1 untracked",
        Status::Focused,
        DropDownTerm::new("wuu status"),
    ),
    Panel::new(
        "Last commit",
        "a1b2c3d: Improve table output",
        Status::Normal,
        DropDownTerm::new(""),
    ),
];

let ui = Wuu::new(Context::Server { cols: 80, rows: 24 });
ui.render_dashboard("WUU VCS", &panels);
```

The renderer arranges panels in one or two columns. `Status::Focused` and
`Status::Alert` mark panels; `DropDownTerm::open_with(prompt, output, input)`
opens a panel's terminal drawer. Dashboard output requires `TERMINAL` to be
configured.

For direct character-cell drawing, use the [`Draw`](src/drawing.rs) trait with
the server backend:

```rust
use wuu::{Draw, Serve};

Serve::draw_container(2, 2, 30, 5, Some(" STATUS "), "Working tree clean");
Serve::draw_horizontal_split(2, 7, 30);
```

This draws a bordered box and divider at the supplied zero-based coordinates.
The trait also provides point, line, rectangle, square, fill, and text methods.
Border characters such as `TOP_LEFT_CORNER` and `HORIZONTAL_LINE` are public
constants.

## Cursor, terminal, and framebuffer

`cursor` provides cursor positioning, relative movement, save/restore,
visibility, shape, and screen clearing. Coordinates are zero-based:

```rust
use wuu::cursor;

cursor::set_cursor_pos(4, 2);
assert_eq!(cursor::get_cursor_pos(), (4, 2));
cursor::move_cursor_by(1, 1);
cursor::save_cursor_pos();
cursor::home();
cursor::restore_cursor_pos();
```

`cursor::with_hidden_cursor(|| { ... })` hides the cursor during a draw and
restores its position and prior visibility afterward.

For OS integration, `ScreenData` implements `os_terminal::DrawTarget` over a
32-bit framebuffer. The OS must provide a valid writable framebuffer pointer,
dimensions, and row pitch; the memory must remain valid while a terminal uses
it. Store the initialized `os_terminal::Terminal<ScreenData>` in
`TERMINAL.lock()` before using print helpers. `TERMINAL` is a spin mutex around
an optional terminal, so initialize it during single-threaded startup when
possible.

## `no_std` notes

The library itself does not require `std`. Prompt text uses a fixed-size inline
buffer, tables borrow their data, and message/diff/progress writers use
`core::fmt::Write`. Your OS remains responsible for framebuffer ownership,
terminal setup, key decoding, and application-specific VCS actions. The
terminal backend may have separate allocator requirements.
