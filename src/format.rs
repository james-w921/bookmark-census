//! Rendering a `Report` for either a person or another program to read.
//! Every caller picks one `OutputFormat`; there's no mixing of the two in a
//! single call, so a script piping this into `jq` never has to guess which
//! lines are data and which are labels.
use crate::report::Report;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
}

pub fn render(report: &Report, format: OutputFormat) -> String {
    match format {
        OutputFormat::Human => render_human(report),
        OutputFormat::Json => render_json(report),
    }
}

fn render_human(report: &Report) -> String {
    let mut out = String::new();
    out.push_str(&format!("folders:       {}\n", report.folder_count));
    out.push_str(&format!("bookmarks:     {}\n", report.bookmark_count));
    out.push_str(&format!("max depth:     {}\n", report.max_depth));

    if report.duplicate_urls.is_empty() {
        out.push_str("duplicates:    none\n");
    } else {
        out.push_str(&format!("duplicates:    {}\n", report.duplicate_urls.len()));
        for (url, count) in &report.duplicate_urls {
            out.push_str(&format!("  {}x  {}\n", count, url));
        }
    }

    if report.empty_folders.is_empty() {
        out.push_str("empty folders: none\n");
    } else {
        out.push_str(&format!(
            "empty folders: {}\n",
            report.empty_folders.len()
        ));
        for path in &report.empty_folders {
            out.push_str(&format!("  {}\n", path));
        }
    }

    out
}

fn render_json(report: &Report) -> String {
    let mut out = String::new();
    out.push('{');
    out.push_str(&format!("\"folder_count\":{},", report.folder_count));
    out.push_str(&format!("\"bookmark_count\":{},", report.bookmark_count));
    out.push_str(&format!("\"max_depth\":{},", report.max_depth));

    out.push_str("\"duplicate_urls\":[");
    for (i, (url, count)) in report.duplicate_urls.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"url\":{},\"count\":{}}}",
            json_string(url),
            count
        ));
    }
    out.push_str("],");

    out.push_str("\"empty_folders\":[");
    for (i, path) in report.empty_folders.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&json_string(path));
    }
    out.push(']');
    out.push('}');
    out
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
