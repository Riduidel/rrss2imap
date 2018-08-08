#[macro_use]
extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rrss2imap",
	about = "
Application transforming rss feeds into email by directly pushing the entries into IMP folders.
This application is an adaption of the rss2imap Python script to Rust.")]
enum RRSS2IMAP {
    #[structopt(name = "new", about = "Creates a new feedfile with the given email address")]
    New {
        #[structopt(short = "e")]
        email: String
    },
    #[structopt(name = "email", about = "Changes email address used in feed file to be the given one")]
    Email {
        email: String
    },
    #[structopt(name = "run", about = "Run feed parsing and transformation")]
    Run {
    },
    #[structopt(name = "add", about = "Adds a new feed given its url")]
    Add {
        // url of the feed. web page urls are not yet supported.
        url: String,
        // email address to use to forward feed content
        email: String,
        // destination folder of feed content
        folder: String
    },
    #[structopt(name = "list", about = "List all feeds configured")]
    List {
    },
    #[structopt(name = "reset", about = "Reset feedfile (in other words, remove everything)")]
    Reset {
    },
    #[structopt(name = "delete", about = "Delete the given feed")]
    Delete {
        // index of the feed to delete
        feed: u32
    },
    #[structopt(name = "export", about = "Export subscriptions as opml file")]
    Export {
        /// Output file, stdout if not present
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>
    },
    #[structopt(name = "import", about = "import the given opml file into subscriptions")]
    Import {
        /// Output file, stdout if not present
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>
    }

}

fn main() {
    let opt = RRSS2IMAP::from_args();
    println!("{:?}", opt);
}