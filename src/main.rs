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
//! Creates a new `config.json` file in the configuration directory
//! (`~/.config/rrss2imap/` on linux,
//! `~/Library/Preferences/org.Rrss2imap.rrss2imap` on macOS,
//! `AppData\Roaming\Rrss2imap\rrss2imap\` on Windows). At init time, the
//! config file will only contains `settings` element with the email address
//! set. You **have** to edit this file and set
//!
//! * the used imap server
//! ** with user login and password
//! ** and security settings (secure should contain `{"Yes": secure port}` for
//!    imap/s or `{"No": unsecure port}` for simple imap)
//! * the default config
//! ** folder will be the *full path to an imap folder* where entries will
//!    fall in (e.g., `INBOX.News`).  The exact syntax depends on your email provider.
//! ** email will be the recipient email address (which may not be yours for easier filtering)
//! ** Base64 image inlining
//!
//! `feeds` is the list of all rss feeds; use `rrss2imap add` to add a new feed.
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
extern crate rfc822_sanitizer;
extern crate unidecode;
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
extern crate reqwest;
extern crate rss;
extern crate xhtmlchardet;
extern crate url;
extern crate tree_magic;
extern crate emailmessage;
extern crate openssl_probe;
extern crate regex;
extern crate custom_error;
extern crate async_std;
extern crate tokio;
extern crate futures;
use flexi_logger::Logger;
use std::path::PathBuf;
use structopt::StructOpt;
use std::error::Error;

mod config;
mod export;
mod feed_errors;
mod feed_reader;
mod feed_utils;
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
#[structopt(author=env!("CARGO_PKG_AUTHORS"))]
struct RRSS2IMAP {
    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
    #[structopt(subcommand)]
    cmd: Command
}

#[derive(Debug, StructOpt)]
enum Command {
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
    /// Adds a new feed given its url.
    /// This option can use either named parameters or positional parameters.
    /// Although positional parameters may seems simpler to use, they're of a more weird usage
    /// (and may be sometimes buggy)
    #[structopt(name = "add")]
    Add {
        /// url of the feed
        #[structopt(short = "u", long = "url")]
        url:Option<String>,
        /// email address to use to forward feed content
        #[structopt(short = "e", long = "email")]
        email:Option<String>,
        /// destination folder of the email
        #[structopt(short = "d", long = "destination")]
        destination:Option<String>,
        /// inline image in this feed (useful only when default is to not include images)
        #[structopt(short = "i", long = "inline-mages")]
        inline_images:bool,
        /// Don't inline image in this feed (useful only when default is to include images)
        #[structopt(short = "x", long = "do-not-inline-mages")]
        do_not_inline_images:bool,
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !cfg!(debug_assertions) {
        setup_panic!();
    }
    let opt = RRSS2IMAP::from_args();

    // Configure logger
    Logger::with_env_or_str(
        match opt.verbose {
            0 => "warn",
            1 => "warn, rrss2imap = info",
            2 => "warn, rrss2imap = info",
            _ => "trace", })
        .format(match opt.verbose {
            0 => flexi_logger::colored_default_format,
            1 => flexi_logger::colored_default_format,
            2 => flexi_logger::colored_detailed_format,
            _ => flexi_logger::colored_with_thread, })
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    openssl_probe::init_ssl_cert_env_vars();

    let store_path = store::find_store();
    let store_result = store::Store::load(&store_path);
    match store_result {
        Ok(mut store) => {
            match opt.cmd {
                Command::New { email } => store.set_email(email),
                Command::Email { email } => store.set_email(email),

                Command::List => store.list(),

                Command::Add { url, email, destination, inline_images, do_not_inline_images, parameters } =>
                    store.add(url, email, destination, store.settings.config.inline(inline_images, do_not_inline_images), parameters),
                Command::Delete { feed } => store.delete(feed),

                Command::Reset => store.reset(),

                Command::Run => {
                    let handle = tokio::spawn(async move {
                        store.run().await
                    });

                    // Wait for the spawned task to finish
                    let _res = handle.await;
                }

                Command::Export { output } => store.export(output),
                Command::Import { input } => store.import(input),
            }
        },
        Err(e) => {
            error!("Impossible to open store {}\n{}", store_path.to_string_lossy(), e);
        }
    }
    Ok(())
}
