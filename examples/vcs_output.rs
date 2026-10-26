use core::fmt::Write as _;

use wuu::{
    ColorMode, Diff, MessageLevel, ProgressBar, Table, write_ko_custom, write_message, write_ok,
    write_ok_custom, write_styled_message,
};

fn main() {
    let mut output = String::new();

    write_message(&mut output, MessageLevel::Info, "Scanning working tree").unwrap();
    write_styled_message(
        &mut output,
        MessageLevel::Success,
        "Commit a1b2c3d created",
        ColorMode::Plain,
    )
    .unwrap();
    write_ok(&mut output, "Working tree scanned", 48, ColorMode::Plain).unwrap();
    write_ko_custom(
        &mut output,
        "Remote connection",
        "offline",
        48,
        ColorMode::Plain,
    )
    .unwrap();
    write_ok_custom(
        &mut output,
        "Objects fetched",
        "complete",
        48,
        ColorMode::Plain,
    )
    .unwrap();

    let headers = ["CHANGE", "STATUS", "PATH"];
    let staged = ["A", "added", "src/commit.rs"];
    let modified = ["M", "modified", "src/lib.rs"];
    let rows: [&[&str]; 2] = [&staged, &modified];
    Table::new(&headers, &rows)
        .write_to_width(&mut output, 48)
        .unwrap();

    let patch = "\
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,2 +1,2 @@
-let old_name = \"wuu\";
+let new_name = \"wuu\";
";
    Diff::new(patch)
        .write_to(&mut output, ColorMode::Plain)
        .unwrap();

    let mut progress = ProgressBar::with_message(4, "Fetching");
    progress.set_width(12);
    for _ in 0..4 {
        progress.inc();
    }
    progress.write_finish(&mut output).unwrap();

    let formatted = wuu::format_stack!("branch={} status={}", "main", "clean");
    writeln!(output, "{}", formatted.as_str()).unwrap();

    println!("{output}");
}
