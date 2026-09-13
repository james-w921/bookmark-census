//! The tree shape every bookmark export gets parsed into, regardless of
//! which browser produced the file.

/// A single saved link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
    /// Seconds since the Unix epoch, taken from the export's ADD_DATE
    /// attribute when present. Browsers don't always write one.
    pub add_date: Option<i64>,
    /// Seconds since the Unix epoch, from LAST_MODIFIED. Only set when the
    /// export actually recorded an edit; a fresh bookmark has none.
    pub last_modified: Option<i64>,
    /// The favicon, usually a base64 `data:` URI, from the ICON attribute.
    pub icon: Option<String>,
}

/// A folder, holding bookmarks and sub-folders in the order the export
/// listed them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Folder {
    pub title: String,
    pub bookmarks: Vec<Bookmark>,
    pub folders: Vec<Folder>,
    /// Seconds since the Unix epoch, from the folder's LAST_MODIFIED
    /// attribute, if the export recorded one.
    pub last_modified: Option<i64>,
}

impl Folder {
    pub fn new(title: impl Into<String>) -> Self {
        Folder {
            title: title.into(),
            bookmarks: Vec::new(),
            folders: Vec::new(),
        }
    }
}
