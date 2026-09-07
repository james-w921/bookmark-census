//! Turns a parsed bookmark tree into the numbers someone actually wants to
//! see: how big it is, how deep it goes, and where the cruft is.
use crate::model::Folder;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Includes the synthetic root folder.
    pub folder_count: usize,
    pub bookmark_count: usize,
    /// 1 for the root, 2 for a folder directly under it, and so on.
    pub max_depth: usize,
    /// URLs saved more than once, sorted, with how many times each appears.
    pub duplicate_urls: Vec<(String, usize)>,
    /// Slash-joined paths of folders with no bookmarks and no sub-folders.
    pub empty_folders: Vec<String>,
}

pub fn analyze(root: &Folder) -> Report {
    let mut acc = Accumulator::default();
    walk(root, 1, root.title.clone(), &mut acc);

    let mut duplicate_urls: Vec<(String, usize)> = acc
        .url_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .collect();
    duplicate_urls.sort();

    Report {
        folder_count: acc.folder_count,
        bookmark_count: acc.bookmark_count,
        max_depth: acc.max_depth,
        duplicate_urls,
        empty_folders: acc.empty_folders,
    }
}

#[derive(Default)]
struct Accumulator {
    folder_count: usize,
    bookmark_count: usize,
    max_depth: usize,
    url_counts: HashMap<String, usize>,
    empty_folders: Vec<String>,
}

fn walk(folder: &Folder, depth: usize, path: String, acc: &mut Accumulator) {
    acc.folder_count += 1;
    if depth > acc.max_depth {
        acc.max_depth = depth;
    }
    if folder.bookmarks.is_empty() && folder.folders.is_empty() {
        acc.empty_folders.push(path.clone());
    }
    for bookmark in &folder.bookmarks {
        acc.bookmark_count += 1;
        *acc.url_counts.entry(bookmark.url.clone()).or_insert(0) += 1;
    }
    for child in &folder.folders {
        let child_path = format!("{}/{}", path, child.title);
        walk(child, depth + 1, child_path, acc);
    }
}
