use crate::bookmark;
use bookmark::Bookmark;
use chrono::Local;
use rusqlite::{params, Connection};
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
                println!("{}", path.exists());
                let db_path = format!("{}/test_db.db", path.display());

                Connection::open(&db_path).unwrap()
            }
        };

        let db = DB { conn };
        db.init();
        db
    }
    fn init(&self) {
        self.conn
            .execute("PRAGMA foreign_keys = ON", params![])
            .unwrap();

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS bookmarks (
                    id              INTEGER PRIMARY KEY,
                    title           TEXT NOT NULL,
                    url             TEXT NOT NULL UNIQUE,
                    notes           TEXT,
                    date_added      DATETIME
                    )",
                params![],
            )
            .unwrap();

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS tags (
                    id              INTEGER PRIMARY KEY,
                    name            TEXT NOT NULL UNIQUE
                )",
                params![],
            )
            .unwrap();

        self.conn
            .execute(
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
            )
            .unwrap();
    }

    pub fn get_all_bookmarks(&self) -> Vec<Bookmark> {
        let query = "SELECT * FROM bookmarks;";
        self.vectorize(query)
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

    pub fn delete_bookmark(&self, url: String) {
        let query = "DELETE FROM bookmarks WHERE url = ?1";
        // let mut stmt = self.conn.prepare(&query).unwrap();
        self.conn.execute(query, params![&url]).unwrap();
    }

    fn vectorize(&self, query: &str) -> Vec<Bookmark> {
        let mut stmt = self.conn.prepare(query).unwrap();

        let bookmark_iter = stmt
            .query_map(params![], |row| {
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
                    println!("The bookmark could not bet saved. {}", msg.unwrap());
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_delete_cascade() {
        // let conn = db.open(false);

        // TODO: check if entries for tags_to_bookmarks exist for values 1,1.
        // then delete bookmark with id 1 and check if it is deleted in tags_to_bookmarks as well
        assert_eq!(1, 1);
    }
}
