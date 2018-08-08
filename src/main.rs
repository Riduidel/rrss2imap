#[macro_use]
extern crate structopt;
extern crate log;

extern crate flexi_logger;

use std::path::PathBuf;
use structopt::StructOpt;
use flexi_logger::Logger;

mod feedstore;

/// Application transforming rss feeds into email by directly pushing the entries into IMP folders.
/// This application is an adaption of the rss2imap Python script to Rust.
#[derive(Debug, StructOpt)]
#[structopt(name = "rrss2imap")]
enum RRSS2IMAP {
    /// Creates a new feedfile with the given email address
    #[structopt(name = "new")]
    New {
        /// email the notifications will be sent to
        email: String
    },
    /// Changes email address used in feed file to be the given one
    #[structopt(name = "email")]
    Email {
        email: String
    },
    /// Run feed parsing and transformation
    #[structopt(name = "run")]
    Run,
    /// Adds a new feed given its url
    #[structopt(name = "add")]
    Add {
        /// url of the feed. web page urls are not yet supported.
        url: String,
        /// email address to use to forward feed content
        email: String,
        /// destination folder of feed content
        folder: String
    },
    /// List all feeds configured
    #[structopt(name = "list")]
    List,
    /// Reset feedfile (in other words, remove everything)
    #[structopt(name = "reset")]
    Reset,
    /// Delete the given feed
    #[structopt(name = "delete")]
    Delete {
        // index of the feed to delete
        feed: u32
    },
    /// Export subscriptions as opml file
    #[structopt(name = "export")]
    Export {
        /// Output file, stdout if not present
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>
    },
    /// import the given opml file into subscriptions
    #[structopt(name = "import")]
    Import {
        /// Output file, stdout if not present
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>
    }

}

fn main() {
    // Configure logger
    Logger::with_env()
                .start()
                .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
    
    let mut store = feedstore::FeedStore::load();
    let opt = RRSS2IMAP::from_args();
    match opt {
        RRSS2IMAP::New { email } => store.set_email(email),
        RRSS2IMAP::Email { email } => store.set_email(email),

        RRSS2IMAP::List => store.list(),

        RRSS2IMAP::Add { url, email, folder } => store.add(url, email, folder),
        RRSS2IMAP::Delete { feed } => store.delete(feed),

        RRSS2IMAP::Reset => store.reset(),

        RRSS2IMAP::Run => store.run(),

        RRSS2IMAP::Export { output } => store.export(output),
        RRSS2IMAP::Import { input } => store.import(input)
    }
} 