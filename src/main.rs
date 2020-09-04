use database::DB;
use parser::Bookie;
use rusqlite::Result;
mod bookmark;
mod database;
mod parser;
use std::{fmt, io};
use structopt::StructOpt;
use Bookie::{Add, Delete, Display, Update};
fn main() -> Result<(), Error> {
    let db = DB::open(false);

    let input = Bookie::from_args();

    match input {
        Display {} => {
            db.display_bookmarks();
        }

        Add {
            url,
            title,
            notes,
            tags,
        } => {
            db.add_bookmark(&title, &url, &notes, &tags)?;
        }
        Delete { id } => {
            let mut input = String::new();

            if db.bookmark_exists(id) {
                println!("Are you sure you want to delete the following bookmark?");
                db.display_selected_bookmark(id);
                println!("y/n");
                io::stdin()
                    .read_line(&mut input)
                    .expect("Couldn't read line");

                match input.as_str() {
                    "y\n" => {
                        db.delete_bookmark(id);
                        println!("Deleted bookmark with id {}", id)
                    }
                    "n\n" => println!("Bookmarks was NOT deleted."),
                    _ => println!("Bookmark was NOT deleted."),
                }
            } else {
                println!("The bookmark with id {} does not exist!", id);
            }
        }
        Update {} => println!("Update functionality not yet implemented"),
    };
    let bookies = db.search_by_tag(vec!["programm", "basi"]);
    for book in bookies {
        println!("Result: {}", book);
    }

    Ok(())
}

#[derive(Debug)]
enum Error {
    RusqliteError(rusqlite::Error),
    IoError(std::io::Error),
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Error {
        Error::RusqliteError(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::RusqliteError(err) => write!(f, "{}", err),
            Error::IoError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::RusqliteError(err) => Some(err),
            Error::IoError(err) => Some(err),
        }
    }
}
