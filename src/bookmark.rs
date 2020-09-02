use crossterm::style::Colorize;
use std::fmt;
pub struct Bookmark {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub date_added: String,
}

impl Bookmark {
    pub fn new(
        id: u32,
        title: String,
        url: String,
        notes: String,
        tags: Vec<String>,
        date_added: String,
    ) -> Self {
        Bookmark {
            id,
            title,
            url,
            notes,
            tags,
            date_added,
        }
    }
}
impl fmt::Display for Bookmark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}. {}\n   {} {}\n   {} {}\n   {} {}\n",
            (self.id).to_string().cyan(),
            self.title.clone().green(),
            ">".red(),
            self.url.clone().dark_yellow(),
            "+".red(),
            self.notes,
            "#".red(),
            self.tags.join(", ").dark_blue()
        )
    }
}
