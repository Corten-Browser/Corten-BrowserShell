use anyhow::Result;
use parking_lot::RwLock;
use rusqlite::{Connection, params};
use std::sync::Arc;

use crate::types::{Bookmark, BookmarkFolder, BookmarkId};

/// SQLite storage for bookmarks
pub struct BookmarkStorage {
    conn: Arc<RwLock<Connection>>,
}

impl BookmarkStorage {
    /// Create new storage with database file
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Self::init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(RwLock::new(conn)),
        })
    }

    /// Initialize database schema
    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                folder TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                favicon BLOB
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmark_tags (
                bookmark_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (bookmark_id, tag),
                FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmark_folders (
                path TEXT PRIMARY KEY,
                parent TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmarks_folder ON bookmarks(folder)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmarks_url ON bookmarks(url)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmarks_title ON bookmarks(title)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmark_tags_tag ON bookmark_tags(tag)",
            [],
        )?;

        Ok(())
    }

    /// Add bookmark to storage
    pub fn add_bookmark(&self, bookmark: &Bookmark) -> Result<()> {
        let conn = self.conn.write();

        conn.execute(
            "INSERT INTO bookmarks (id, url, title, folder, created_at, updated_at, favicon)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                bookmark.id,
                bookmark.url,
                bookmark.title,
                bookmark.folder,
                bookmark.created_at,
                bookmark.updated_at,
                bookmark.favicon,
            ],
        )?;

        // Insert tags
        for tag in &bookmark.tags {
            conn.execute(
                "INSERT INTO bookmark_tags (bookmark_id, tag) VALUES (?1, ?2)",
                params![bookmark.id, tag],
            )?;
        }

        Ok(())
    }

    /// Get bookmark by ID
    pub fn get_bookmark(&self, id: &BookmarkId) -> Result<Option<Bookmark>> {
        let conn = self.conn.read();

        let mut stmt = conn.prepare(
            "SELECT id, url, title, folder, created_at, updated_at, favicon
             FROM bookmarks WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let bookmark_id: String = row.get(0)?;
            let url: String = row.get(1)?;
            let title: String = row.get(2)?;
            let folder: Option<String> = row.get(3)?;
            let created_at: i64 = row.get(4)?;
            let updated_at: i64 = row.get(5)?;
            let favicon: Option<Vec<u8>> = row.get(6)?;

            // Get tags
            let mut tag_stmt = conn.prepare(
                "SELECT tag FROM bookmark_tags WHERE bookmark_id = ?1"
            )?;
            let tags: Vec<String> = tag_stmt
                .query_map(params![bookmark_id], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Some(Bookmark {
                id: bookmark_id,
                url,
                title,
                folder,
                tags,
                favicon,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update bookmark
    pub fn update_bookmark(&self, bookmark: &Bookmark) -> Result<()> {
        let conn = self.conn.write();

        conn.execute(
            "UPDATE bookmarks SET url = ?2, title = ?3, folder = ?4, updated_at = ?5, favicon = ?6
             WHERE id = ?1",
            params![
                bookmark.id,
                bookmark.url,
                bookmark.title,
                bookmark.folder,
                bookmark.updated_at,
                bookmark.favicon,
            ],
        )?;

        // Update tags: delete old ones and insert new ones
        conn.execute(
            "DELETE FROM bookmark_tags WHERE bookmark_id = ?1",
            params![bookmark.id],
        )?;

        for tag in &bookmark.tags {
            conn.execute(
                "INSERT INTO bookmark_tags (bookmark_id, tag) VALUES (?1, ?2)",
                params![bookmark.id, tag],
            )?;
        }

        Ok(())
    }

    /// Delete bookmark
    pub fn delete_bookmark(&self, id: &BookmarkId) -> Result<()> {
        let conn = self.conn.write();
        conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// List all bookmarks
    pub fn list_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let conn = self.conn.read();

        let mut stmt = conn.prepare(
            "SELECT id, url, title, folder, created_at, updated_at, favicon FROM bookmarks"
        )?;

        let bookmarks: Vec<Bookmark> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(id, url, title, folder, created_at, updated_at, favicon)| {
                // Get tags for this bookmark
                let mut tag_stmt = conn.prepare(
                    "SELECT tag FROM bookmark_tags WHERE bookmark_id = ?1"
                ).unwrap();
                let tags: Vec<String> = tag_stmt
                    .query_map(params![&id], |row| row.get(0))
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                Bookmark {
                    id,
                    url,
                    title,
                    folder,
                    tags,
                    favicon,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(bookmarks)
    }

    /// List bookmarks in folder
    pub fn list_bookmarks_in_folder(&self, folder: &str) -> Result<Vec<Bookmark>> {
        let conn = self.conn.read();

        let mut stmt = conn.prepare(
            "SELECT id, url, title, folder, created_at, updated_at, favicon
             FROM bookmarks WHERE folder = ?1"
        )?;

        let bookmarks: Vec<Bookmark> = stmt
            .query_map(params![folder], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(id, url, title, folder, created_at, updated_at, favicon)| {
                // Get tags
                let mut tag_stmt = conn.prepare(
                    "SELECT tag FROM bookmark_tags WHERE bookmark_id = ?1"
                ).unwrap();
                let tags: Vec<String> = tag_stmt
                    .query_map(params![&id], |row| row.get(0))
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                Bookmark {
                    id,
                    url,
                    title,
                    folder,
                    tags,
                    favicon,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(bookmarks)
    }

    /// Search bookmarks by query
    pub fn search_bookmarks(&self, query: &str) -> Result<Vec<Bookmark>> {
        let conn = self.conn.read();
        let search_pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT id, url, title, folder, created_at, updated_at, favicon
             FROM bookmarks WHERE title LIKE ?1 OR url LIKE ?1"
        )?;

        let bookmarks: Vec<Bookmark> = stmt
            .query_map(params![search_pattern], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(id, url, title, folder, created_at, updated_at, favicon)| {
                // Get tags
                let mut tag_stmt = conn.prepare(
                    "SELECT tag FROM bookmark_tags WHERE bookmark_id = ?1"
                ).unwrap();
                let tags: Vec<String> = tag_stmt
                    .query_map(params![&id], |row| row.get(0))
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                Bookmark {
                    id,
                    url,
                    title,
                    folder,
                    tags,
                    favicon,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(bookmarks)
    }

    /// Find bookmarks by tag
    pub fn find_by_tag(&self, tag: &str) -> Result<Vec<Bookmark>> {
        let conn = self.conn.read();

        let mut stmt = conn.prepare(
            "SELECT b.id, b.url, b.title, b.folder, b.created_at, b.updated_at, b.favicon
             FROM bookmarks b
             INNER JOIN bookmark_tags t ON b.id = t.bookmark_id
             WHERE t.tag = ?1"
        )?;

        let bookmarks: Vec<Bookmark> = stmt
            .query_map(params![tag], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(id, url, title, folder, created_at, updated_at, favicon)| {
                // Get all tags
                let mut tag_stmt = conn.prepare(
                    "SELECT tag FROM bookmark_tags WHERE bookmark_id = ?1"
                ).unwrap();
                let tags: Vec<String> = tag_stmt
                    .query_map(params![&id], |row| row.get(0))
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                Bookmark {
                    id,
                    url,
                    title,
                    folder,
                    tags,
                    favicon,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(bookmarks)
    }

    /// Create folder
    pub fn create_folder(&self, path: &str) -> Result<()> {
        let conn = self.conn.write();
        let folder = BookmarkFolder::new(path.to_string());
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO bookmark_folders (path, parent, created_at) VALUES (?1, ?2, ?3)",
            params![folder.path, folder.parent, now],
        )?;

        Ok(())
    }

    /// Delete folder
    pub fn delete_folder(&self, path: &str) -> Result<()> {
        let conn = self.conn.write();
        conn.execute("DELETE FROM bookmark_folders WHERE path = ?1", params![path])?;
        Ok(())
    }

    /// List all folders
    pub fn list_folders(&self) -> Result<Vec<BookmarkFolder>> {
        let conn = self.conn.read();

        let mut stmt = conn.prepare(
            "SELECT path, parent FROM bookmark_folders"
        )?;

        let folders: Vec<BookmarkFolder> = stmt
            .query_map([], |row| {
                let path: String = row.get(0)?;
                let parent: Option<String> = row.get(1)?;

                // Count bookmarks in this folder
                let count: usize = conn
                    .query_row(
                        "SELECT COUNT(*) FROM bookmarks WHERE folder = ?1",
                        params![&path],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                Ok(BookmarkFolder {
                    path,
                    parent,
                    children: Vec::new(),
                    bookmark_count: count,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(folders)
    }

    /// Get bookmark count
    pub fn count(&self) -> Result<usize> {
        let conn = self.conn.read();
        let count: usize = conn.query_row(
            "SELECT COUNT(*) FROM bookmarks",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Clear all bookmarks
    pub fn clear(&self) -> Result<()> {
        let conn = self.conn.write();
        conn.execute("DELETE FROM bookmarks", [])?;
        conn.execute("DELETE FROM bookmark_folders", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_storage() -> BookmarkStorage {
        BookmarkStorage::new(":memory:").expect("Failed to create test storage")
    }

    fn create_test_bookmark(url: &str, title: &str) -> Bookmark {
        let mut bookmark = Bookmark::new(url.to_string(), title.to_string());
        bookmark.id = "test-id-1".to_string();
        bookmark
    }

    #[test]
    fn test_storage_creation() {
        let storage = create_test_storage();
        assert_eq!(storage.count().unwrap(), 0);
    }

    #[test]
    fn test_add_bookmark() {
        let storage = create_test_storage();
        let bookmark = create_test_bookmark("https://example.com", "Example");

        storage.add_bookmark(&bookmark).unwrap();
        assert_eq!(storage.count().unwrap(), 1);
    }

    #[test]
    fn test_get_bookmark() {
        let storage = create_test_storage();
        let bookmark = create_test_bookmark("https://example.com", "Example");

        storage.add_bookmark(&bookmark).unwrap();
        let retrieved = storage.get_bookmark(&bookmark.id).unwrap();

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, bookmark.id);
        assert_eq!(retrieved.url, bookmark.url);
        assert_eq!(retrieved.title, bookmark.title);
    }

    #[test]
    fn test_get_nonexistent_bookmark() {
        let storage = create_test_storage();
        let result = storage.get_bookmark(&"nonexistent".to_string()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_bookmark() {
        let storage = create_test_storage();
        let mut bookmark = create_test_bookmark("https://example.com", "Example");
        storage.add_bookmark(&bookmark).unwrap();

        bookmark.title = "Updated Title".to_string();
        storage.update_bookmark(&bookmark).unwrap();

        let retrieved = storage.get_bookmark(&bookmark.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Title");
    }

    #[test]
    fn test_delete_bookmark() {
        let storage = create_test_storage();
        let bookmark = create_test_bookmark("https://example.com", "Example");
        storage.add_bookmark(&bookmark).unwrap();

        storage.delete_bookmark(&bookmark.id).unwrap();
        assert_eq!(storage.count().unwrap(), 0);
        assert!(storage.get_bookmark(&bookmark.id).unwrap().is_none());
    }

    #[test]
    fn test_list_bookmarks() {
        let storage = create_test_storage();
        let bookmark1 = create_test_bookmark("https://example1.com", "Example 1");
        let mut bookmark2 = create_test_bookmark("https://example2.com", "Example 2");
        bookmark2.id = "test-id-2".to_string();

        storage.add_bookmark(&bookmark1).unwrap();
        storage.add_bookmark(&bookmark2).unwrap();

        let bookmarks = storage.list_bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 2);
    }

    #[test]
    fn test_bookmark_with_tags() {
        let storage = create_test_storage();
        let mut bookmark = create_test_bookmark("https://example.com", "Example");
        bookmark.tags = vec!["rust".to_string(), "programming".to_string()];
        storage.add_bookmark(&bookmark).unwrap();

        let retrieved = storage.get_bookmark(&bookmark.id).unwrap().unwrap();
        assert_eq!(retrieved.tags.len(), 2);
        assert!(retrieved.tags.contains(&"rust".to_string()));
        assert!(retrieved.tags.contains(&"programming".to_string()));
    }

    #[test]
    fn test_find_by_tag() {
        let storage = create_test_storage();
        let mut bookmark1 = create_test_bookmark("https://rust-lang.org", "Rust");
        bookmark1.tags = vec!["rust".to_string()];
        let mut bookmark2 = create_test_bookmark("https://python.org", "Python");
        bookmark2.id = "test-id-2".to_string();
        bookmark2.tags = vec!["python".to_string()];

        storage.add_bookmark(&bookmark1).unwrap();
        storage.add_bookmark(&bookmark2).unwrap();

        let rust_bookmarks = storage.find_by_tag("rust").unwrap();
        assert_eq!(rust_bookmarks.len(), 1);
        assert_eq!(rust_bookmarks[0].url, "https://rust-lang.org");
    }

    #[test]
    fn test_bookmark_with_folder() {
        let storage = create_test_storage();
        let mut bookmark = create_test_bookmark("https://example.com", "Example");
        bookmark.folder = Some("Programming/Rust".to_string());
        storage.add_bookmark(&bookmark).unwrap();

        let retrieved = storage.get_bookmark(&bookmark.id).unwrap().unwrap();
        assert_eq!(retrieved.folder, Some("Programming/Rust".to_string()));
    }

    #[test]
    fn test_list_bookmarks_in_folder() {
        let storage = create_test_storage();
        let mut bookmark1 = create_test_bookmark("https://rust-lang.org", "Rust");
        bookmark1.folder = Some("Programming/Rust".to_string());
        let mut bookmark2 = create_test_bookmark("https://python.org", "Python");
        bookmark2.id = "test-id-2".to_string();
        bookmark2.folder = Some("Programming/Python".to_string());

        storage.add_bookmark(&bookmark1).unwrap();
        storage.add_bookmark(&bookmark2).unwrap();

        let rust_bookmarks = storage.list_bookmarks_in_folder("Programming/Rust").unwrap();
        assert_eq!(rust_bookmarks.len(), 1);
        assert_eq!(rust_bookmarks[0].url, "https://rust-lang.org");
    }

    #[test]
    fn test_search_bookmarks_by_title() {
        let storage = create_test_storage();
        let bookmark1 = create_test_bookmark("https://rust-lang.org", "Rust Programming");
        let mut bookmark2 = create_test_bookmark("https://python.org", "Python Language");
        bookmark2.id = "test-id-2".to_string();

        storage.add_bookmark(&bookmark1).unwrap();
        storage.add_bookmark(&bookmark2).unwrap();

        let results = storage.search_bookmarks("Rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");
    }

    #[test]
    fn test_search_bookmarks_by_url() {
        let storage = create_test_storage();
        let bookmark1 = create_test_bookmark("https://rust-lang.org", "Rust");
        let mut bookmark2 = create_test_bookmark("https://python.org", "Python");
        bookmark2.id = "test-id-2".to_string();

        storage.add_bookmark(&bookmark1).unwrap();
        storage.add_bookmark(&bookmark2).unwrap();

        let results = storage.search_bookmarks("rust-lang").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://rust-lang.org");
    }

    #[test]
    fn test_create_folder() {
        let storage = create_test_storage();
        storage.create_folder("Programming").unwrap();

        let folders = storage.list_folders().unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].path, "Programming");
    }

    #[test]
    fn test_create_nested_folder() {
        let storage = create_test_storage();
        storage.create_folder("Programming/Rust").unwrap();

        let folders = storage.list_folders().unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].path, "Programming/Rust");
    }

    #[test]
    fn test_delete_folder() {
        let storage = create_test_storage();
        storage.create_folder("Programming").unwrap();
        storage.delete_folder("Programming").unwrap();

        let folders = storage.list_folders().unwrap();
        assert_eq!(folders.len(), 0);
    }

    #[test]
    fn test_clear() {
        let storage = create_test_storage();
        let bookmark1 = create_test_bookmark("https://example1.com", "Example 1");
        let mut bookmark2 = create_test_bookmark("https://example2.com", "Example 2");
        bookmark2.id = "test-id-2".to_string();

        storage.add_bookmark(&bookmark1).unwrap();
        storage.add_bookmark(&bookmark2).unwrap();
        storage.create_folder("Programming").unwrap();

        storage.clear().unwrap();

        assert_eq!(storage.count().unwrap(), 0);
        assert_eq!(storage.list_folders().unwrap().len(), 0);
    }
}
