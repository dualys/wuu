# wuu

`wuu` provides allocation-free terminal UI helpers for VCS-style applications:
status messages, tables, diffs, progress bars, and event-driven prompts. The
library is `no_std`; it does not read stdin or own your application's input
loop.

## Terminal output

The print helpers write to the shared `TERMINAL`. Configure it with your
terminal/framebuffer before calling `print`, `finish`, or another print helper.
If you don't have a terminal configured yet, use the `write_to` functions to
render into any `core::fmt::Write` destination.

## Status messages

Use message levels for short feedback after VCS operations:

```rust
use wuu::{ColorMode, MessageLevel, print_styled_message};

print_styled_message(
    MessageLevel::Success,
    "Commit created",
    ColorMode::Ansi,
);
print_styled_message(
    MessageLevel::Warning,
    "Working tree has unstaged changes",
    ColorMode::Plain,
);
```

Available levels are `Info`, `Success`, `Warning`, and `Error`. Choose
`ColorMode::Plain` when ANSI colors are unsupported. To format a message
without using the global terminal:

```rust
use wuu::{MessageLevel, write_message};

let mut output = String::new();
write_message(&mut output, MessageLevel::Info, "Scanning working tree").unwrap();
assert_eq!(output, "[INFO] Scanning working tree\n");
```

## Tables

Tables borrow their headers and rows, so building one does not require `wuu` to
allocate. Use `print_auto` to fit the configured terminal width:

```rust
use wuu::Table;

let headers = ["CHANGE", "STATUS", "PATH"];
let modified = ["M", "modified", "src/lib.rs"];
let untracked = ["?", "untracked", "README.md"];
let rows: [&[&str]; 2] = [&modified, &untracked];

Table::new(&headers, &rows).print_auto();
```

For a caller-supplied width or a custom output destination:

```rust
use wuu::Table;

let headers = ["STATUS", "PATH"];
let changed = ["modified", "src/very/long/path/to/file.rs"];
let rows: [&[&str]; 1] = [&changed];
let table = Table::new(&headers, &rows);

let mut output = String::new();
table.write_to_width(&mut output, 32).unwrap();
assert!(output.lines().all(|line| line.chars().count() <= 32));
```

Long paths are shortened from the left to retain their ending; other cells are
shortened from the right. `write_to` and `print` use the natural table width.
Cell widths count Unicode scalar values, so use single-column characters for
predictable alignment.

## Diffs

`Diff` accepts unified-diff text and colors added, removed, and hunk-header
lines. File headers (`+++` and `---`) are left uncolored:

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

Use `ColorMode::Plain` for terminals without ANSI support, or call
`Diff::write_to(&mut output, mode)` to render elsewhere.

## Progress bars

Update a bar as work completes, then call `finish` to end its line:

```rust
use wuu::ProgressBar;

let files_to_fetch = 100;
let mut progress = ProgressBar::with_message(files_to_fetch, "Fetching");
progress.set_width(30);

for completed_files in 1..=files_to_fetch {
    // Fetch one file here.
    progress.set_position(completed_files);
    progress.print();
}
progress.finish();
```

Alternatively, use `inc()` after each item. The position is clamped to the
total; a total of zero is displayed as complete. `write_to` and `write_finish`
render to any `core::fmt::Write` destination.

## Interactive prompts

Prompts are event-driven. Your application translates keyboard events into
`PromptKey`, passes them to `handle_key`, then calls `print` to redraw. `wuu`
does not read stdin or decide how platform-specific key events are collected.

### Text entry

```rust
use wuu::{Prompt, PromptKey, PromptResult};

let mut prompt = Prompt::text("Commit message");
prompt.print();

// In your input loop, translate received keys and redraw:
prompt.handle_key(PromptKey::Character('F'));
prompt.handle_key(PromptKey::Character('i'));
prompt.handle_key(PromptKey::Character('x'));
prompt.handle_key(PromptKey::Enter);
prompt.print();

if let Some(PromptResult::Text(message)) = prompt.result() {
    // Pass `message` to the commit operation.
}
```

Text input stores up to `TEXT_CAPACITY` bytes (256); backspace removes one
Unicode character.

### Confirmation

Confirmations default to no. `Toggle`, `y`/`Y`, or `n`/`N` changes the answer;
`Enter` accepts it and `Escape` cancels:

```rust
use wuu::{Prompt, PromptKey, PromptResult};

let mut prompt = Prompt::confirm("Delete this branch?");
prompt.handle_key(PromptKey::Character('y'));
prompt.handle_key(PromptKey::Enter);

if let Some(PromptResult::Confirm(confirmed)) = prompt.result() {
    if confirmed {
        // Delete the branch.
    }
}
```

### Selecting an option

```rust
use wuu::{Prompt, PromptKey, PromptResult};

let branches = ["main", "release", "feature"];
let mut prompt = Prompt::select("Switch to branch", &branches).unwrap();
prompt.handle_key(PromptKey::Down);
prompt.handle_key(PromptKey::Enter);

if let Some(PromptResult::Selected(index)) = prompt.result() {
    let branch = branches[index];
    // Switch to `branch`.
}
```

Use `Up` and `Down` to move through choices. Empty option lists return
`PromptError::NoChoices`.

### Selecting multiple options

Multi-select uses a caller-owned boolean slice, with one flag per option:

```rust
use wuu::{Prompt, PromptKey, PromptResult};

let actions = ["stage", "commit", "push"];
let mut selected = [false; 3];
let mut prompt =
    Prompt::multi_select("Choose actions", &actions, &mut selected).unwrap();

prompt.handle_key(PromptKey::Toggle); // Toggle "stage"
prompt.handle_key(PromptKey::Down);
prompt.handle_key(PromptKey::Toggle); // Toggle "commit"
prompt.handle_key(PromptKey::Enter);

if let Some(PromptResult::Multiple(choices)) = prompt.result() {
    // `choices` is [true, true, false].
}
```

`Toggle` or a space character toggles the highlighted option. The selection
slice must have the same length as the options.

### Selecting a Noun

[`Noun`](https://crates.io/crates/noun) is a 32-byte BLAKE3 content identifier.
`noun_select` lets users choose among identifiers your application already
has; it does not read files or compute their hashes. By default, each option
shows the first eight bytes (16 hex characters). Set the prefix length to show
more or fewer bytes:

```rust
use wuu::{Noun, Prompt, PromptKey, PromptResult};

// In a VCS, these might be identifiers computed from file contents.
let objects = [Noun::of(b"first file contents"), Noun::of(b"second file")];
let mut prompt = Prompt::noun_select("Choose an object", &objects)
    .unwrap()
    .noun_prefix_len(12);

prompt.handle_key(PromptKey::Down);
prompt.handle_key(PromptKey::Enter);

if let Some(PromptResult::Noun(noun)) = prompt.result() {
    // Use the selected content identifier.
}
```

Use `Escape` to cancel any prompt. Check `prompt.state()` for `Pending`,
`Accepted`, or `Cancelled`; `result()` returns a value only after acceptance.
