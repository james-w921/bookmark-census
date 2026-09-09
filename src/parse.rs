//! Parser for the Netscape bookmark file format: the HTML export produced by
//! "Export Bookmarks" in Chrome, Firefox, and Safari alike. The format
//! predates XHTML and is not well-formed markup (DT and P tags are never
//! closed), so a general HTML parser is the wrong tool here. This walks the
//! tag stream by hand and only understands the handful of tags the format
//! actually uses: H1, H3, A, DL, DT.
use crate::model::{Bookmark, Folder};
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    /// A DL close tag had no matching open, or the document ended with
    /// folders still open.
    UnbalancedTags,
    /// An H3 or A tag was never closed, so its title text is ambiguous.
    UnclosedTag(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnbalancedTags => write!(f, "unbalanced <DL> tags in bookmark export"),
            ParseError::UnclosedTag(tag) => write!(f, "unclosed <{}> tag in bookmark export", tag),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse a Netscape bookmark export into a tree. The returned `Folder` is
/// the synthetic root; its title comes from the document's H1 if one is
/// present, otherwise it stays "Bookmarks".
pub fn parse(input: &str) -> Result<Folder, ParseError> {
    let mut stack: Vec<Folder> = vec![Folder::new("Bookmarks")];
    let mut pending_folder: Option<Folder> = None;
    let mut cursor = 0usize;

    while let Some(rel) = input[cursor..].find('<') {
        let tag_start = cursor + rel + 1;
        let tag_end = match input[tag_start..].find('>') {
            Some(i) => tag_start + i,
            None => break,
        };
        let tag_content = &input[tag_start..tag_end];
        cursor = tag_end + 1;

        let (name, attrs, closing) = split_tag(tag_content);

        match name.to_ascii_lowercase().as_str() {
            "h1" if !closing => {
                if let Some((text_end, after)) = find_close(input, cursor, "h1") {
                    stack[0].title = decode_entities(&input[cursor..text_end]);
                    cursor = after;
                }
            }
            "h3" if !closing => {
                let (text_end, after) =
                    find_close(input, cursor, "h3").ok_or(ParseError::UnclosedTag("h3"))?;
                pending_folder = Some(Folder::new(decode_entities(&input[cursor..text_end])));
                cursor = after;
            }
            "a" if !closing => {
                let (text_end, after) =
                    find_close(input, cursor, "a").ok_or(ParseError::UnclosedTag("a"))?;
                let title = decode_entities(&input[cursor..text_end]);
                let url = get_attr(attrs, "href").unwrap_or_default();
                let add_date = get_attr(attrs, "add_date").and_then(|v| v.parse().ok());
                cursor = after;

                let top = stack.last_mut().ok_or(ParseError::UnbalancedTags)?;
                top.bookmarks.push(Bookmark {
                    title,
                    url,
                    add_date,
                });
            }
            "dl" => {
                if closing {
                    let finished = stack.pop().ok_or(ParseError::UnbalancedTags)?;
                    match stack.last_mut() {
                        Some(parent) => parent.folders.push(finished),
                        // this was the root's own DL; put it back so parsing
                        // can finish and hand it back to the caller below
                        None => stack.push(finished),
                    }
                } else if let Some(folder) = pending_folder.take() {
                    stack.push(folder);
                }
                // an opening DL with no pending folder is the root list,
                // which is already on the stack
            }
            _ => {}
        }
    }

    if stack.len() != 1 {
        return Err(ParseError::UnbalancedTags);
    }
    Ok(stack.pop().unwrap())
}

/// Splits `<name attr="val" ...>` content (without the angle brackets) into
/// the tag name, the raw attribute string, and whether it's a close tag.
fn split_tag(content: &str) -> (&str, &str, bool) {
    let content = content.trim();
    let closing = content.starts_with('/');
    let content = content.strip_prefix('/').unwrap_or(content).trim_start();
    match content.find(char::is_whitespace) {
        Some(idx) => (&content[..idx], content[idx..].trim(), closing),
        None => (content, "", closing),
    }
}

/// Scans forward from `from` for the next `</tag>` and returns the byte
/// offset where its text content ends, plus the cursor position just past
/// the close tag.
fn find_close(input: &str, from: usize, tag: &str) -> Option<(usize, usize)> {
    let mut i = from;
    loop {
        let rel = input[i..].find('<')?;
        let pos = i + rel;
        let bytes = input.as_bytes();
        if pos + 1 < bytes.len() && bytes[pos + 1] == b'/' {
            let name_start = pos + 2;
            let gt_rel = input[name_start..].find('>')?;
            let gt = name_start + gt_rel;
            if input[name_start..gt].trim().eq_ignore_ascii_case(tag) {
                return Some((pos, gt + 1));
            }
        }
        i = pos + 1;
    }
}

/// Looks up `key="value"` (or `key='value'`) in a raw attribute string.
/// Matching is case-insensitive on the key, as the format itself is
/// inconsistent about attribute casing across browsers.
fn get_attr(attrs: &str, key: &str) -> Option<String> {
    let mut rest = attrs;
    loop {
        let eq = rest.find('=')?;
        let name = rest[..eq].trim();
        let after_eq = rest[eq + 1..].trim_start();
        let mut chars = after_eq.chars();
        let quote = chars.next()?;
        let (value, tail) = if quote == '"' || quote == '\'' {
            let body = &after_eq[quote.len_utf8()..];
            let end = body.find(quote)?;
            (&body[..end], &body[end + quote.len_utf8()..])
        } else {
            let end = after_eq
                .find(char::is_whitespace)
                .unwrap_or(after_eq.len());
            (&after_eq[..end], &after_eq[end..])
        };
        if name.eq_ignore_ascii_case(key) {
            return Some(value.to_string());
        }
        rest = tail;
    }
}

/// Decodes the handful of entities that actually show up in bookmark
/// titles. Unrecognized entities are passed through unchanged.
fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }
        let mut entity = String::new();
        let mut found_semi = false;
        while let Some(&nc) = chars.peek() {
            if nc == ';' {
                chars.next();
                found_semi = true;
                break;
            }
            if entity.len() > 8 {
                break;
            }
            entity.push(nc);
            chars.next();
        }
        if !found_semi {
            out.push('&');
            out.push_str(&entity);
            continue;
        }
        match entity.as_str() {
            "amp" => out.push('&'),
            "lt" => out.push('<'),
            "gt" => out.push('>'),
            "quot" => out.push('"'),
            "apos" | "#39" => out.push('\''),
            _ => {
                out.push('&');
                out.push_str(&entity);
                out.push(';');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_flat_list_of_bookmarks() {
        let doc = r#"
            <DL><p>
                <DT><A HREF="https://a.example/">A</A>
                <DT><A HREF="https://b.example/">B</A>
            </DL><p>
        "#;
        let root = parse(doc).unwrap();
        assert_eq!(root.bookmarks.len(), 2);
        assert_eq!(root.bookmarks[0].url, "https://a.example/");
        assert_eq!(root.bookmarks[1].url, "https://b.example/");
        assert!(root.folders.is_empty());
    }

    #[test]
    fn parses_nested_folders_in_document_order() {
        let doc = r#"
            <H1>My Bookmarks</H1>
            <DL><p>
                <DT><H3 ADD_DATE="1600000000">Folder A</H3>
                <DL><p>
                    <DT><A HREF="https://example.com/">Example</A>
                    <DT><H3>Nested</H3>
                    <DL><p>
                        <DT><A HREF="https://nested.example.com/">Nested Link</A>
                    </DL><p>
                </DL><p>
                <DT><A HREF="https://top.example.com/">Top Link</A>
            </DL><p>
        "#;
        let root = parse(doc).unwrap();
        assert_eq!(root.title, "My Bookmarks");
        assert_eq!(root.bookmarks.len(), 1);
        assert_eq!(root.bookmarks[0].url, "https://top.example.com/");
        assert_eq!(root.folders.len(), 1);

        let folder_a = &root.folders[0];
        assert_eq!(folder_a.title, "Folder A");
        assert_eq!(folder_a.bookmarks.len(), 1);
        assert_eq!(folder_a.folders.len(), 1);

        let nested = &folder_a.folders[0];
        assert_eq!(nested.title, "Nested");
        assert_eq!(nested.bookmarks.len(), 1);
        assert_eq!(nested.bookmarks[0].url, "https://nested.example.com/");
    }

    #[test]
    fn root_title_defaults_when_no_h1_present() {
        let doc = "<DL><p></DL><p>";
        let root = parse(doc).unwrap();
        assert_eq!(root.title, "Bookmarks");
    }

    #[test]
    fn add_date_is_parsed_when_present_and_none_otherwise() {
        let doc = r#"
            <DL><p>
                <DT><A HREF="https://dated.example/" ADD_DATE="1700000000">Dated</A>
                <DT><A HREF="https://undated.example/">Undated</A>
            </DL><p>
        "#;
        let root = parse(doc).unwrap();
        assert_eq!(root.bookmarks[0].add_date, Some(1700000000));
        assert_eq!(root.bookmarks[1].add_date, None);
    }

    #[test]
    fn attribute_quoting_and_casing_are_tolerated() {
        let doc = r#"<DL><p><DT><a href='https://single.example/'>Single</a></DL><p>"#;
        let root = parse(doc).unwrap();
        assert_eq!(root.bookmarks[0].url, "https://single.example/");
    }

    #[test]
    fn decodes_common_entities_in_titles() {
        let doc = r#"<DL><p><DT><H3>Q&amp;A &lt;Archive&gt;</H3><DL><p></DL><p></DL><p>"#;
        let root = parse(doc).unwrap();
        assert_eq!(root.folders[0].title, "Q&A <Archive>");
    }

    #[test]
    fn unrecognized_and_unterminated_entities_pass_through() {
        assert_eq!(decode_entities("A &frobnicate; B"), "A &frobnicate; B");
        assert_eq!(decode_entities("A & B"), "A & B");
    }

    #[test]
    fn unclosed_h3_tag_is_an_error() {
        let doc = "<DL><p><DT><H3>Folder with no close<DL><p></DL><p></DL><p>";
        match parse(doc) {
            Err(ParseError::UnclosedTag("h3")) => {}
            other => panic!("expected UnclosedTag(\"h3\"), got {:?}", other),
        }
    }

    #[test]
    fn unclosed_a_tag_is_an_error() {
        let doc = r#"<DL><p><DT><A HREF="https://example.com/">Example</DL><p>"#;
        match parse(doc) {
            Err(ParseError::UnclosedTag("a")) => {}
            other => panic!("expected UnclosedTag(\"a\"), got {:?}", other),
        }
    }

    #[test]
    fn folders_left_open_at_end_of_document_are_unbalanced() {
        let doc = r#"
            <DL><p>
                <DT><H3>Never closed</H3>
                <DL><p>
                    <DT><A HREF="https://example.com/">Example</A>
        "#;
        match parse(doc) {
            Err(ParseError::UnbalancedTags) => {}
            other => panic!("expected UnbalancedTags, got {:?}", other),
        }
    }
}
