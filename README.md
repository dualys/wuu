# wuu

Allocation-free terminal helpers for status messages and small tables:

```rust
use wuu::{MessageLevel, Table, print_message};

let headers = ["CHANGE", "STATUS", "PATH"];
let modified = ["M", "modified", "src/lib.rs"];
let untracked = ["?", "untracked", "README.md"];
let rows: [&[&str]; 2] = [&modified, &untracked];

Table::new(&headers, &rows).print();
print_message(MessageLevel::Info, "Working tree summary ready");
```

`Table::write_to` and `write_message` can also write to any `core::fmt::Write`
destination. The `print` methods write to `TERMINAL` when it has been configured.