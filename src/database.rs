use crate::{bookmark, Error};
use bookmark::Bookmark;
use chrono::Local;
use crossterm::style::Colorize;
use crossterm::style::Styler;
use rusqlite::{params, Connection, NO_PARAMS};
use std::fs;
use std::path;
pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn open(in_memory: bool) -> DB {
        let conn = match in_memory {
            true => Connection::open_in_memory().unwrap(),
            false => {
                let home_dir = dirs::home_dir().unwrap();
                let bookie_dir = format!("{}/.bookie", home_dir.display());
                let path = path::Path::new(&bookie_dir);
                fs::create_dir_all(path).unwrap();
                let db_path = format!("{}/test_db.db", path.display());

                Connection::open(&db_path).unwrap()
            }
        };

        let db = DB { conn };
        match db.init() {
            Ok(_) => {}
            Err(err) => {
                println!("{:?}", err);
            }
        }

        db
    }
    fn init(&self) -> Result<(), Error> {
        self.conn.execute("PRAGMA foreign_keys = ON", params![])?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                    id              INTEGER PRIMARY KEY,
                    title           TEXT NOT NULL,
                    url             TEXT NOT NULL UNIQUE,
                    notes           TEXT,
                    date_added      DATETIME
                    )",
            params![],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                    id              INTEGER PRIMARY KEY,
                    name            TEXT NOT NULL UNIQUE
                )",
            params![],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags_to_bookmarks (
                    id              INTEGER PRIMARY KEY,
                    bookmark_id     INTEGER NOT NULL,
                    tag_id          INTEGER NOT NULL,

                    FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE,
                    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
                );
                CREATE UNIQUE  INDEX UIX_bookmark_id_tag_id
                ON tags_to_bookmarks
                ( bookmark_id ASC, tag_id ASC );",
            params![],
        )?;
        Ok(())
    }

    pub fn get_all_bookmarks(&self) -> Vec<Bookmark> {
        let query = "SELECT * FROM bookmarks;";
        self.vectorize(query, Vec::new())
    }

    pub fn get_selected_bookmark(&self, id: u32) -> Vec<Bookmark> {
        let query = "SELECT * FROM bookmarks where id = ?1";
        self.vectorize(query, vec![id.to_string()])
    }

    pub fn get_tags(&self, bookmark_id: u32) -> rusqlite::Result<Vec<String>> {
        let query = format!("SELECT name from tags where id in (SELECT tag_id FROM tags_to_bookmarks WHERE bookmark_id = {});", bookmark_id);
        let mut stmt = self.conn.prepare(query.as_str()).unwrap();
        let tag_iter = stmt.query_map(params![], |row| row.get(0))?;

        let mut tags: Vec<String> = Vec::new();
        for tag in tag_iter {
            tags.push(tag.unwrap());
        }
        Ok(tags)
    }

    pub fn delete_bookmark(&self, id: u32) {
        let query = "DELETE FROM bookmarks WHERE id = ?1";
        // let mut stmt = self.conn.prepare(&query).unwrap();
        self.conn.execute(query, params![&id]).unwrap();
    }

    fn vectorize(&self, query: &str, params: Vec<String>) -> Vec<Bookmark> {
        let mut stmt = self.conn.prepare(query).unwrap();

        let bookmark_iter = stmt
            .query_map(params, |row| {
                Ok(Bookmark {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    url: row.get(2)?,
                    notes: row.get(3)?,
                    date_added: row.get(4)?,
                    tags: self.get_tags(row.get(0)?).unwrap(),
                })
            })
            .unwrap();

        let mut bookmarks: Vec<Bookmark> = Vec::new();
        for bookmark in bookmark_iter {
            bookmarks.push(bookmark.unwrap());
        }

        bookmarks
    }

    pub fn add_bookmark(
        &self,
        title: &String,
        url: &String,
        notes: &String,
        tags: &Vec<String>,
    ) -> Result<(), rusqlite::Error> {
        let query = "INSERT INTO bookmarks (title, url, notes, date_added) VALUES (?1, ?2, ?3, ?4)";
        let date_added = Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .replace("\"", "");

        match self
            .conn
            .execute(query, params![title, url, notes, date_added])
        {
            Ok(_) => {
                self.add_tags(tags, url);
                Ok(())
            }
            Err(err) => match err {
                rusqlite::Error::SqliteFailure(_, msg) => {
                    println!(
                        "{} {}",
                        "The bookmark could not bet saved:\n".red(),
                        msg.unwrap().red().underlined()
                    );
                    Ok(())
                }
                _ => {
                    println!(
                        "Something went wrong! The bookmark could not be saved. Please try again."
                    );
                    Err(err)
                }
            },
        }
    }

    pub fn add_tags(&self, tags: &Vec<String>, url: &String) {
        for tag in tags {
            self.conn
                .execute(
                    "INSERT INTO tags(name)
                    SELECT ?1
                    WHERE NOT EXISTS(SELECT 1 from tags where name = ?1)
                   ",
                    params![tag],
                )
                .unwrap();

            self.conn
                .execute(
                    "INSERT OR IGNORE INTO tags_to_bookmarks
            (bookmark_id, tag_id) SELECT
            (SELECT id from bookmarks where url = ?1), (SELECT id from tags where name = ?2);",
                    params![url, tag],
                )
                .unwrap();
        }
    }

    pub fn display_bookmarks(&self) {
        let bookmark_iter = self.get_all_bookmarks();

        for bookmark in bookmark_iter {
            println!("{}", bookmark);
        }
    }

    pub fn display_selected_bookmark(&self, id: u32) {
        let bookmark_iter = self.get_selected_bookmark(id);

        for bookmark in bookmark_iter {
            println!("{}", bookmark);
        }
    }

    pub fn get_url_by_id(&self, id: u32) -> String {
        let query = "SELECT URL FROM bookmarks WHERE id = ?1;";
        let url: String = self
            .conn
            .query_row(query, params![id], |row| row.get(0))
            .unwrap_or(format!("Could not find URL for id {}", id));
        url
    }

    pub fn get_bookmark_count(&self) -> u32 {
        let query = "SELECT COUNT(*) FROM bookmarks;";
        let count = self
            .conn
            .query_row(query, rusqlite::NO_PARAMS, |row| row.get(0))
            .unwrap();
        count
    }

    pub fn bookmark_exists(&self, id: u32) -> bool {
        let query = "SELECT COUNT(*) FROM bookmarks WHERE id = ?1";
        let count: u8 = self
            .conn
            .query_row(query, params![id], |row| row.get(0))
            .unwrap();

        match count {
            1 => true,
            _ => false,
        }
    }

    pub fn search_by_tag(&self, mut tags: Vec<&str>) -> Vec<Bookmark> {
        let mut query = format!(
            "SELECT *
        FROM bookmarks b
        WHERE b.id in 
            (SELECT tb.bookmark_id FROM tags_to_bookmarks tb
                INNER JOIN tags t on tb.tag_id = t.id
                WHERE t.name LIKE \"%{}%\"",
            &tags.remove(0)
        );
        for tag in tags {
            query.push_str(format!("OR t.name LIKE \"%{}%\"", tag).as_str());
        }
        query.push_str(");");
        self.vectorize(query.as_str(), Vec::new())
    }

    pub fn search(&self, mut search_list: Vec<String>) -> Vec<Bookmark> {
        let mut query = format!(
            "SELECT *
        FROM bookmarks b
        WHERE b.id in 
            (SELECT tb.bookmark_id FROM tags_to_bookmarks tb 
                INNER JOIN tags t on tb.tag_id = t.id
                INNER JOIN bookmarks bm on tb.bookmark_id = bm.id
                WHERE (t.name || bm.title || bm.url) LIKE \"%{}%\"",
            &search_list.remove(0)
        );
        for keyword in search_list {
            query.push_str(
                format!("OR (t.name || bm.title || bm.url) LIKE \"%{}%\"", keyword).as_str(),
            );
        }
        query.push_str(");");
        self.vectorize(query.as_str(), Vec::new())
    }
}

#[cfg(test)]
mod tests {
    fn test_db_setup() -> DB {
        let db = DB::open(true);
        let bm1 = bookmark::Bookmark {
            id: 1,
            title: "Wikipedia".to_string(),
            url: "wikipedia.org".to_string(),
            notes: "".to_string(),
            tags: vec!["knowledge".to_string(), "encyclopedia".to_string()],
            date_added: "".to_string(),
        };
        let _ = db.add_bookmark(&bm1.title, &bm1.url, &bm1.notes, &bm1.tags);

        let bm2 = bookmark::Bookmark {
            id: 2,
            title: "GitHub".to_string(),
            url: "github.com".to_string(),
            notes: "Where code lives".to_string(),
            tags: vec!["programming".to_string(), "Coding".to_string()],
            date_added: "".to_string(),
        };
        let _ = db.add_bookmark(&bm2.title, &bm2.url, &bm2.notes, &bm2.tags);
        db
    }
    use super::*;

    #[test]
    fn test_get_url_by_id() {
        let db = test_db_setup();
        let b = db.get_url_by_id(1);
        assert_eq!("wikipedia.org", b);
    }

    #[test]
    fn test_bm_count() {
        let db = test_db_setup();
        assert_eq!(2, db.get_bookmark_count())
    }
    #[test]
    fn test_delete_bookmark() {
        let db = test_db_setup();
        db.delete_bookmark(1);
        assert_eq!(1, db.get_bookmark_count());
    }

    #[test]
    fn test_bookmark_exists() {
        let db = test_db_setup();
        assert!(db.bookmark_exists(1));
        assert!(!db.bookmark_exists(10));
    }
}
