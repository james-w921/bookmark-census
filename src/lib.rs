//! Reads the Netscape-format bookmark file that Chrome, Firefox, and Safari
//! all produce from "Export Bookmarks", and reports on what's actually in
//! it: how many bookmarks, how deep the folder tree goes, which URLs got
//! saved more than once, and which folders are dead weight.
//!
//! This is a library, not a CLI. Wire `parse`, `analyze`, and `render`
//! together in your own small program; see the README for a full example.

pub mod format;
pub mod model;
pub mod parse;
pub mod report;

pub use format::{render, OutputFormat};
pub use model::{Bookmark, Folder};
pub use parse::{parse, ParseError};
pub use report::{analyze, Report};
