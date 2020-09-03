use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Bookie",
    about = "All possible usages for bookmark manager Bookie"
)]
pub enum Bookie {
    Display {},

    Add {
        /// Activate debug mode
        // short and long flags (-d, --debug) will be deduced from the field's name
        // #[structopt(short, long)]
        // debug: bool,

        /// URL
        #[structopt()]
        url: String,

        /// Title
        #[structopt(long = "title", default_value = "")]
        title: String,

        /// Notes for URL
        #[structopt(short = "n", long = "notes", default_value = "")]
        notes: String,

        /// Date Added
        // #[structopt(skip = format!("{:?}", Local::now().format("%Y-%m-%d %H:%M:%S").to_string().replace("\"", "") ))]
        // date_added: String,

        /// Tags
        #[structopt(short = "t", long = "tags", default_value = "")]
        tags: Vec<String>,
    },
    Delete {
        /// ID
        #[structopt()]
        id: u32,
    },
    Update {},
}
