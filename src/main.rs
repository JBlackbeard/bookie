use database::DB;
use parser::Bookie;
use rusqlite::Result;
mod bookmark;
mod database;
mod parser;
use std::fmt;
use structopt::StructOpt;
use Bookie::{Add, Delete, Update};
fn main() -> Result<(), Error> {
    let db = DB::open(false);

    let input = Bookie::from_args();

    match input {
        Add {
            url,
            title,
            notes,
            tags,
        } => {
            db.add_bookmark(&title, &url, &notes, &tags)?;
        }
        Delete {} => println!("Delete functionality not yet implemented"),
        Update {} => println!("Update functionality not yet implemented"),
    };

    db.display_bookmarks();

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
