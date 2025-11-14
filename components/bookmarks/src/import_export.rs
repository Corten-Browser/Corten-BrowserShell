use anyhow::Result;
use crate::types::Bookmark;

/// Parse HTML bookmarks (Netscape format)
pub fn parse_html(html: &str) -> Result<Vec<Bookmark>> {
    let mut bookmarks = Vec::new();
    let mut folder_stack: Vec<String> = Vec::new();

    for line in html.lines() {
        let line = line.trim();

        // Check for folder start
        if line.starts_with("<DT><H3>") {
            if let Some(folder_name) = extract_folder_name(line) {
                folder_stack.push(folder_name);
            }
        }
        // Check for folder end
        else if line.starts_with("</DL>") {
            if !folder_stack.is_empty() {
                folder_stack.pop();
            }
        }
        // Check for bookmark
        else if line.starts_with("<DT><A HREF=") {
            if let Some(bookmark) = parse_bookmark_line(line, &folder_stack) {
                bookmarks.push(bookmark);
            }
        }
    }

    Ok(bookmarks)
}

fn extract_folder_name(line: &str) -> Option<String> {
    // Extract text between <H3> and </H3>
    let start = line.find("<H3>")?;
    let end = line.find("</H3>")?;
    Some(line[start + 4..end].to_string())
}

fn parse_bookmark_line(line: &str, folder_stack: &[String]) -> Option<Bookmark> {
    // Extract URL
    let href_start = line.find("HREF=\"")? + 6;
    let href_end = line[href_start..].find('"')?;
    let url = line[href_start..href_start + href_end].to_string();

    // Extract creation date
    let created_at = if let Some(date_start) = line.find("ADD_DATE=\"") {
        let date_start = date_start + 10;
        if let Some(date_end) = line[date_start..].find('"') {
            line[date_start..date_start + date_end]
                .parse::<i64>()
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    // Extract title (text between last > and </A>)
    let title_end = line.find("</A>")?;
    let before_title = &line[..title_end];
    let title_start = before_title.rfind('>')? + 1;
    let title = line[title_start..title_end].to_string();

    // Build folder path
    let folder = if folder_stack.is_empty() {
        None
    } else {
        Some(folder_stack.join("/"))
    };

    Some(Bookmark {
        id: uuid::Uuid::new_v4().to_string(),
        url,
        title,
        folder,
        tags: Vec::new(),
        favicon: None,
        created_at,
        updated_at: created_at,
    })
}

/// Generate HTML bookmarks (Netscape format)
pub fn generate_html(bookmarks: &[Bookmark]) -> Result<String> {
    let mut html = String::new();

    html.push_str("<!DOCTYPE NETSCAPE-Bookmark-file-1>\n");
    html.push_str("<HTML>\n");
    html.push_str("<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n");
    html.push_str("<TITLE>Bookmarks</TITLE>\n");
    html.push_str("<H1>Bookmarks</H1>\n");
    html.push_str("<DL><p>\n");

    // Group bookmarks by folder
    let mut folder_map: std::collections::HashMap<Option<String>, Vec<&Bookmark>> =
        std::collections::HashMap::new();

    for bookmark in bookmarks {
        folder_map
            .entry(bookmark.folder.clone())
            .or_insert_with(Vec::new)
            .push(bookmark);
    }

    // Generate root-level bookmarks
    if let Some(root_bookmarks) = folder_map.get(&None) {
        for bookmark in root_bookmarks {
            html.push_str(&format!(
                "    <DT><A HREF=\"{}\" ADD_DATE=\"{}\">{}</A>\n",
                bookmark.url, bookmark.created_at, bookmark.title
            ));
        }
    }

    // Generate folders and their bookmarks
    let mut folders: Vec<_> = folder_map
        .keys()
        .filter_map(|k| k.as_ref())
        .collect();
    folders.sort();

    for folder_path in folders {
        generate_folder(&mut html, folder_path, &folder_map, 1);
    }

    html.push_str("</DL><p>\n");
    html.push_str("</HTML>\n");

    Ok(html)
}

fn generate_folder(
    html: &mut String,
    folder_path: &str,
    folder_map: &std::collections::HashMap<Option<String>, Vec<&Bookmark>>,
    indent_level: usize,
) {
    let indent = "    ".repeat(indent_level);
    let parts: Vec<&str> = folder_path.split('/').collect();

    // Generate folder hierarchy
    for (i, part) in parts.iter().enumerate() {
        let current_indent = "    ".repeat(indent_level + i);
        html.push_str(&format!("{}<DT><H3>{}</H3>\n", current_indent, part));
        html.push_str(&format!("{}<DL><p>\n", current_indent));
    }

    // Generate bookmarks in this folder
    if let Some(bookmarks) = folder_map.get(&Some(folder_path.to_string())) {
        let bookmark_indent = "    ".repeat(indent_level + parts.len());
        for bookmark in bookmarks {
            html.push_str(&format!(
                "{}<DT><A HREF=\"{}\" ADD_DATE=\"{}\">{}</A>\n",
                bookmark_indent, bookmark.url, bookmark.created_at, bookmark.title
            ));
        }
    }

    // Close folder hierarchy
    for i in (0..parts.len()).rev() {
        let current_indent = "    ".repeat(indent_level + i);
        html.push_str(&format!("{}</DL><p>\n", current_indent));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<HTML>
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<TITLE>Bookmarks</TITLE>
<H1>Bookmarks</H1>
<DL><p>
    <DT><A HREF="https://example.com" ADD_DATE="1234567890">Example</A>
</DL><p>"#;

        let bookmarks = parse_html(html).unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].url, "https://example.com");
        assert_eq!(bookmarks[0].title, "Example");
        assert_eq!(bookmarks[0].created_at, 1234567890);
    }

    #[test]
    fn test_parse_html_with_folders() {
        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<HTML>
<DL><p>
    <DT><H3>Programming</H3>
    <DL><p>
        <DT><A HREF="https://rust-lang.org" ADD_DATE="1234567890">Rust</A>
    </DL><p>
</DL><p>"#;

        let bookmarks = parse_html(html).unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].url, "https://rust-lang.org");
        assert_eq!(bookmarks[0].folder, Some("Programming".to_string()));
    }

    #[test]
    fn test_parse_html_with_nested_folders() {
        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<HTML>
<DL><p>
    <DT><H3>Programming</H3>
    <DL><p>
        <DT><H3>Rust</H3>
        <DL><p>
            <DT><A HREF="https://rust-lang.org" ADD_DATE="1234567890">Rust Lang</A>
        </DL><p>
    </DL><p>
</DL><p>"#;

        let bookmarks = parse_html(html).unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].folder, Some("Programming/Rust".to_string()));
    }

    #[test]
    fn test_parse_html_multiple_bookmarks() {
        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<HTML>
<DL><p>
    <DT><A HREF="https://example1.com" ADD_DATE="1234567890">Example 1</A>
    <DT><A HREF="https://example2.com" ADD_DATE="1234567891">Example 2</A>
</DL><p>"#;

        let bookmarks = parse_html(html).unwrap();
        assert_eq!(bookmarks.len(), 2);
        assert_eq!(bookmarks[0].url, "https://example1.com");
        assert_eq!(bookmarks[1].url, "https://example2.com");
    }

    #[test]
    fn test_parse_html_empty() {
        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<HTML>
<DL><p>
</DL><p>"#;

        let bookmarks = parse_html(html).unwrap();
        assert_eq!(bookmarks.len(), 0);
    }

    #[test]
    fn test_generate_simple_html() {
        let bookmarks = vec![
            Bookmark {
                id: "1".to_string(),
                url: "https://example.com".to_string(),
                title: "Example".to_string(),
                folder: None,
                tags: vec![],
                favicon: None,
                created_at: 1234567890,
                updated_at: 1234567890,
            },
        ];

        let html = generate_html(&bookmarks).unwrap();
        assert!(html.contains("<!DOCTYPE NETSCAPE-Bookmark-file-1>"));
        assert!(html.contains(r#"<A HREF="https://example.com""#));
        assert!(html.contains("Example</A>"));
        assert!(html.contains("ADD_DATE=\"1234567890\""));
    }

    #[test]
    fn test_generate_html_with_folders() {
        let bookmarks = vec![
            Bookmark {
                id: "1".to_string(),
                url: "https://rust-lang.org".to_string(),
                title: "Rust".to_string(),
                folder: Some("Programming".to_string()),
                tags: vec![],
                favicon: None,
                created_at: 1234567890,
                updated_at: 1234567890,
            },
        ];

        let html = generate_html(&bookmarks).unwrap();
        assert!(html.contains("<H3>Programming</H3>"));
        assert!(html.contains("https://rust-lang.org"));
    }

    #[test]
    fn test_generate_html_with_nested_folders() {
        let bookmarks = vec![
            Bookmark {
                id: "1".to_string(),
                url: "https://rust-lang.org".to_string(),
                title: "Rust".to_string(),
                folder: Some("Programming/Rust".to_string()),
                tags: vec![],
                favicon: None,
                created_at: 1234567890,
                updated_at: 1234567890,
            },
        ];

        let html = generate_html(&bookmarks).unwrap();
        assert!(html.contains("<H3>Programming</H3>"));
        assert!(html.contains("<H3>Rust</H3>"));
    }

    #[test]
    fn test_roundtrip() {
        let original_bookmarks = vec![
            Bookmark {
                id: "1".to_string(),
                url: "https://example.com".to_string(),
                title: "Example".to_string(),
                folder: Some("Test".to_string()),
                tags: vec![],
                favicon: None,
                created_at: 1234567890,
                updated_at: 1234567890,
            },
        ];

        let html = generate_html(&original_bookmarks).unwrap();
        let parsed_bookmarks = parse_html(&html).unwrap();

        assert_eq!(parsed_bookmarks.len(), 1);
        assert_eq!(parsed_bookmarks[0].url, original_bookmarks[0].url);
        assert_eq!(parsed_bookmarks[0].title, original_bookmarks[0].title);
        assert_eq!(parsed_bookmarks[0].folder, original_bookmarks[0].folder);
        assert_eq!(parsed_bookmarks[0].created_at, original_bookmarks[0].created_at);
    }
}
