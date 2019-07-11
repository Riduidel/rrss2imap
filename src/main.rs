//! Application transforming rss feeds into email by directly pushing the entries into IMP folders.
//! This application is an adaption of the rss2imap Python script to Rust.
//!
//! #### How to use ?
//!
//! The simplest way to understand what to do is just to run `rrss2imap --help`
//!
//! It should output something like
//!
//!     FLAGS:
//!         -h, --help       Prints help information
//!         -V, --version    Prints version information
//!     
//!     SUBCOMMANDS:
//!         add       Adds a new feed given its url
//!         delete    Delete the given feed
//!         email     Changes email address used in feed file to be the given one
//!         export    Export subscriptions as opml file
//!         help      Prints this message or the help of the given subcommand(s)
//!         import    import the given opml file into subscriptions
//!         list      List all feeds configured
//!         new       Creates a new feedfile with the given email address
//!         reset     Reset feedfile (in other words, remove everything)
//!         run       Run feed parsing and transformation
//!
//! Which give you a glimpse of what will happen
//!
//! Each of these commands also provide some help, when run with the same `--help` flag.
//!
//! The important operations to memorize are obviously
//!
//! #### `rrss2imap new`
//!
//! Creates a new `config.json` file. At init time, the config file will only contains `settings` element
//! with the email address set. You **have** to set
//!
//! * the used imap server
//! ** with user login and password
//! ** and security settings (secure should contain `{"Yes": secure port}` for imap/s
//! or `{"No": unsecure port}` for simple imap)
//! * the default config
//! ** folder will be the full path to an imap folder where entries will fall in
//! ** email will be the recipient email address (which may not be yours for easier filtering)
//! ** Base64 image inlining
//! * feeds is the list of all rss feeds that can be added
//!
//! #### `rrss2imap add`
//!
//! This command will add a new feed to your config. You can directly set here the email recipient as well as the folder
//! (but not the base64 image inlining parameter)
//!
//! #### `rrss2imap run`
//!
//! THis is the main command. It will
//!
//! 1. get all rss/atom feed contents
//! 2. List all new entries in these feeds
//! 3. Transform these entries into valid email messages
//! 4. Push these mail messages directly on IMAP server
//!
//! #### `rrss2imap list`
//!
//! Displays a list of the rss feeds. Here is an example
//!
//! ```
//! 0 : http://tontof.net/?rss (to: Nicolas Delsaux <nicolas.delsaux@gmx.fr> (default)) RSS/rrss2imap (default)
//! 1 : https://www.brothers-brick.com/feed/ (to: Nicolas Delsaux <nicolas.delsaux@gmx.fr> (default)) RSS/rrss2imap (default)
//! 2 : https://nicolas-delsaux.hd.free.fr/rss-bridge/?action=display&bridge=LesJoiesDuCode&format=AtomFormat (to: Nicolas Delsaux <nicolas.delsaux@gmx.fr> (default)) RSS/rrss2imap (default)
//! ```
//!
//! Please notice that each entry has an associated number, which is the one to enter when running `rrss2imap delete <NUMBER>`
//!
//! #### `config.json` format
//!
//! A typical feedfile will look like this
//!
//! ```json
//!     {
//!       "settings": {
//!         "email": {
//!           "server": "the imap server of your mail provider",
//!           "user": "your imap user name",
//!           "password": "your imap user password",
//!           "secure": {
//!             "Yes": 993 // Set to "Yes": port for imaps or "No": port for unsecure imap
//!           }
//!         },
//!         // This config is to be used for all feeds
//!         "config": {
//!             // This is the email address written in each mail sent. It can be different from the email user
//!             "email": "Nicolas Delsaux <nicolas.delsaux@gmx.fr>",
//!             // This is the imap folder in which mails will be written
//!             "folder": "RSS/rrss2imap"
//!             // Setting this to true will force rrss2imap to transform all images into
//!             // base64. This prevents images from beind downloaded (and is really cool when reading feeds from a smartphone)
//!             // But largely increase each mail size (which can be quite bothering)
//!             "inline_image_as_data": true
//!         }
//!       },
//!       "feeds": [
//!         {
//!           "url": "http://tontof.net/?rss",
//!           // This last updated is updated for each entry and should be enough to have rss items correctly read
//!           "last_updated": "2019-05-04T16:53:15",
//!           "config": {
//!               // each config element can be overwritten at the feed level
//!           }
//!         },
//! ```
//!     

extern crate structopt;
#[macro_use]
extern crate log;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate flexi_logger;

extern crate treexml;

extern crate chrono;

#[macro_use]
extern crate tera;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate human_panic;

extern crate kuchiki;

extern crate imap;
extern crate native_tls;

extern crate base64;

extern crate atom_syndication;
extern crate requests;
extern crate rss;

extern crate xhtmlchardet;

extern crate url;

extern crate tree_magic;

extern crate quoted_printable;

extern crate emailmessage;

use flexi_logger::Logger;
use std::path::PathBuf;
use structopt::StructOpt;

mod config;
mod export;
mod feed;
mod image_to_data;
mod import;
mod message;
mod settings;
mod store;
mod syndication;

///
/// rrss2imap is a script used to transform rss feed entries into mail messages that are directly dropped
/// into your mailbox by the grace of imap protocol
///
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
        /// - url of the feed. web page urls are not yet supported. Given as first parameters, **mandatory**
        ///
        /// - email address to use to forward feed content, **optional**
        ///
        /// - destination folder of feed content, **optional**
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

/// Main function simply load the RRSS2IMAP struct from the command-line arguments
fn main() {
    if !cfg!(debug_assertions) {
        setup_panic!();
    }
    // Configure logger

    Logger::with_str("warn, rrss2imap = info")
        .format(flexi_logger::colored_detailed_format)
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
