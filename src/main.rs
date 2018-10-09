#[macro_use]
extern crate structopt;
#[macro_use]
extern crate log;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate flexi_logger;

extern crate treexml;

extern crate requests;

extern crate feed_rs;

extern crate chrono;

#[macro_use]
extern crate tera;

#[macro_use]
extern crate lazy_static;

use flexi_logger::Logger;
use std::path::PathBuf;
use structopt::StructOpt;

mod config;
mod export;
mod feed;
mod import;
mod store;
mod settings;

/// Application transforming rss feeds into email by directly pushing the entries into IMP folders.
/// This application is an adaption of the rss2imap Python script to Rust.
#[derive(Debug, StructOpt)]
#[structopt(name = "rrss2imap")]
enum RRSS2IMAP {
    /// Creates a new feedfile with the given email address
    #[structopt(name = "new")]
    New {
        /// email the notifications will be sent to
        email: String,
    },
    /// Changes email address used in feed file to be the given one
    #[structopt(name = "email")]
    Email { email: String },
    /// Run feed parsing and transformation
    #[structopt(name = "run")]
    Run,
    /// Adds a new feed given its url
    #[structopt(name = "add")]
    Add {
        /// Parameters used to add the feed. Expected parameters are
        ///
        /// - url of the feed. web page urls are not yet supported. Given as first parameters, mandatory
        ///
        /// - email address to use to forward feed content, optional
        ///
        /// - destination folder of feed content, optional
        ///
        /// Notice parameters have to be given in THIS order.
        parameters: Vec<String>,
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
        feed: u32,
    },
    /// Export subscriptions as opml file
    #[structopt(name = "export")]
    Export {
        /// Output file, stdout if not present
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    },
    /// import the given opml file into subscriptions
    #[structopt(name = "import")]
    Import {
        /// Output file, stdout if not present
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>,
    },
}

fn main() {
    // Configure logger
    Logger::with_env()
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let mut store = store::Store::load();
    let opt = RRSS2IMAP::from_args();
    match opt {
        RRSS2IMAP::New { email } => store.set_email(email),
        RRSS2IMAP::Email { email } => store.set_email(email),

        RRSS2IMAP::List => store.list(),

        RRSS2IMAP::Add { parameters } => store.add(parameters),
        RRSS2IMAP::Delete { feed } => store.delete(feed),

        RRSS2IMAP::Reset => store.reset(),

        RRSS2IMAP::Run => {
            store.run();
        }

        RRSS2IMAP::Export { output } => store.export(output),
        RRSS2IMAP::Import { input } => store.import(input),
    }
}
