# bookmark-census

Every browser will happily export your bookmarks as HTML, and none of them
will tell you that a third of that file is dead folders and the same login
page saved four times under different names. This crate parses that export
and produces a report: how many bookmarks, how deep the folder tree goes,
which URLs show up more than once, and which folders are empty.

It's a library, not a program. There is no `bookmark-census` binary to
install. You call three functions from your own code (or a test, or a
five-line `main.rs` you write yourself) and get back either a plain-text
report or a JSON document, depending on what you're doing with the result.

## Where the input comes from

Chrome, Firefox, and Safari all have an "Export Bookmarks" option that
writes a file in the old Netscape bookmark format: an HTML file with `H3`
tags for folders and `A` tags for links, nested in `DL` lists. That's the
only input format this crate reads. It is not a bookmarks *manager* - it
doesn't write that file back out, and it doesn't touch your browser's
profile directly.

## Usage

```rust
use bookmark_census::{analyze, parse, render, OutputFormat};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = fs::read_to_string("bookmarks.html")?;
    let tree = parse(&html)?;
    let report = analyze(&tree);

    println!("{}", render(&report, OutputFormat::Human));
    Ok(())
}
```

Human-readable output looks like this:

```
folders:       14
bookmarks:     231
max depth:     4
duplicates:    3
  2x  https://news.ycombinator.com/
  2x  https://en.wikipedia.org/wiki/Rust_(programming_language)
  3x  https://github.com/
empty folders: 2
  Bookmarks/Imported/2019 archive
  Bookmarks/Work/old team wiki
```

Ask for `OutputFormat::Json` instead and the same report comes back as one
JSON object, meant for a script rather than a terminal:

```json
{"folder_count":14,"bookmark_count":231,"max_depth":4,"duplicate_urls":[{"url":"https://github.com/","count":3},{"url":"https://news.ycombinator.com/","count":2},{"url":"https://en.wikipedia.org/wiki/Rust_(programming_language)","count":2}],"empty_folders":["Bookmarks/Imported/2019 archive","Bookmarks/Work/old team wiki"]}
```

`parse` also hands back the full tree (`Folder`, with nested `folders` and
`bookmarks`) if you want to do something `analyze` doesn't cover, like
walking it yourself or writing your own report.

## Dependencies

None. Everything here, including the JSON output, is written against the
standard library. `cargo build` doesn't reach the network.

## Status

Early. The parser handles the export format as written by current Chrome,
Firefox, and Safari; other tools that claim to speak this format sometimes
take small liberties with it, and not all of those are covered yet. See
below for what's next.
