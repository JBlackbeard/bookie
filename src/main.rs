use database::DB;
use parser::Bookie;
use rusqlite::Result;
mod bookmark;
mod database;
mod parser;
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

    // let bm = bookmark::Bookmark {
    //     id: 1,
    //     title: "Wikipedia".to_string(),
    //     url: "wikipedia.org".to_string(),
    //     notes: "".to_string(),
    //     tags: vec!["Knowledge".to_string(), "encyclopedia".to_string()],
    //     date_added: "".to_string(),
    // };
    // match db.add_bookmark(&bm.title, &bm.url, &bm.notes, &bm.tags) {
    //     Ok(a) => a,
    //     Err(a) => println!("{}", a),
    // }

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
