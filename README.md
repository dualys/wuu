# wuu

Allocation-free terminal helpers for status messages, diffs, tables, and progress bars:

```rust
use wuu::{ColorMode, Diff, MessageLevel, ProgressBar, Table, print_styled_message};

let headers = ["CHANGE", "STATUS", "PATH"];
let modified = ["M", "modified", "src/lib.rs"];
let untracked = ["?", "untracked", "README.md"];
let rows: [&[&str]; 2] = [&modified, &untracked];

Table::new(&headers, &rows).print_auto();
print_styled_message(
    MessageLevel::Info,
    "Working tree summary ready",
    ColorMode::Ansi,
);
Diff::new("--- a/file\n+++ b/file\n-old\n+new\n").print(ColorMode::Ansi);

let mut progress = ProgressBar::with_message(100, "Fetching");
progress.set_position(42);
progress.print();
progress.set_position(100);
progress.finish();
```

Prompts are event-driven: feed keyboard events from your application and redraw
the prompt. Noun prompts show an eight-byte hexadecimal prefix by default:

```rust
use wuu::{Noun, Prompt, PromptKey, PromptResult};

let object = Noun::of(b"file contents");
let objects = [object];
let mut prompt = Prompt::noun_select("Choose an object", &objects)?;
prompt.print();
prompt.handle_key(PromptKey::Enter);
if let Some(PromptResult::Noun(selected)) = prompt.result() {
    // Use the selected content identifier.
}
# Ok::<(), wuu::PromptError>(())
```

Use `ColorMode::Plain` when ANSI colors are unsupported. `Table::print_auto`
uses the configured terminal width; `print_width` and `write_to_width` accept
an explicit width. Long paths are shortened from the left to preserve their
ending, while other cells are shortened from the right.
`Table::write_to`, `Diff::write_to`, and the message writers accept any
`core::fmt::Write` destination. Terminal print helpers write only when `TERMINAL`
has been configured.