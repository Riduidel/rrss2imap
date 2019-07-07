use imap::error::Result;
use imap::Session;

use super::config::Config;

/// Secured connection or not ?
/// Whichever is chosen, user has to give the port as parameter
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Secure {
    No(u16),
    Yes(u16),
}
/// mail config
/// I SHOULD allow a kind of Keepass access.
/// But as code isn't expected to run on any kind of UI-aware machine (but on a headless Raspbian),
/// I can't connect it to Keepass.
/// So I should implement a kind of secure storage
#[derive(Debug, Deserialize, Serialize)]
pub struct Email {
    /// imap server we want to connect to
    server: String,
    /// username used to connect to that server
    user: String,
    /// password used to connect to that server.
    /// **WARNING** THis password is in **no way** encrypted, which makes rrss2imap a "not-so-secured" software
    password: String,
    /// secured connection state
    #[serde(default = "Email::default_secure")]
    secure: Secure,
    #[serde(default = "Email::default_retry_max_count")]
    retry_max_count:u8,
    #[serde(default = "Email::default_retry_delay")]
    retry_delay:u64
}

/// Imap effective connection type (ie once connection has been established).
/// This enum presents a simple interface allowing seamless access for (un)secured servers.
#[derive(Debug)]
pub enum Imap {
    Secured(Session<native_tls::TlsStream<std::net::TcpStream>>),
    Insecured(Session<std::net::TcpStream>),
}

impl Imap {
    /// Appends a new message to the given server.
    pub fn append<S: AsRef<str>, B: AsRef<[u8]>>(&mut self, mailbox: S, content: B) -> Result<()> {
        match self {
            Imap::Secured(ref mut session) => session.append(mailbox, content),
            Imap::Insecured(ref mut session) => session.append(mailbox, content),
        }
    }
}

impl Email {
    /// Appends a new message to the given server.
    /// This method decorates the Imap::append method by adding retry ability.
    pub fn append<S: AsRef<str>, B: AsRef<[u8]>>(&self, mailbox: &S, content: &B) -> Result<()> {
        let mut count = 0;
        loop {
            count = count+1;
            let mut imap = self.start();
            let result = imap.append(mailbox, content);
            if result.is_err() {
                if count>self.retry_max_count {
                    return result;
                } else {
                    error!("Previous append attempt failed with {}. Retrying ({}/{})in {} s.!", 
                        result.unwrap_err(), 
                        count,
                        self.retry_max_count,
                        self.retry_delay);
                    // TODO maybe remove that once code is parallel
                    thread::sleep(time::Duration::from_secs(self.retry_delay));
                }
            } else {
                return result;
            }
        }
    }

    /// default secure port, used by serde
    pub fn default_secure() -> Secure {
        Secure::Yes(993)
    }
    /// default max retries number, used by serde
    pub fn default_retry_max_count() -> u8 {
        3
    }
    /// default retry delay, used by serde
    pub fn default_retry_delay() -> u64 {
        1
    }
    /// Constructs a default email config, used in Settings by serde
    pub fn default() -> Email {
        Email {
            server: "Set your email server address here".to_owned(),
            user: "Set your imap server user name (it may be your email address or not)".to_owned(),
            password: "Set your imap server password (yup, in clear, this is very bad)".to_owned(),
            secure: Email::default_secure(),
            retry_max_count: Email::default_retry_max_count(),
            retry_delay: Email::default_retry_delay()
        }
    }

    /// starts connection to selected imap server, whatever it is
    pub fn start(&self) -> Imap {
        match self.secure {
            Secure::Yes(port) => self.start_secure(port),
            Secure::No(port) => self.start_insecure(port),
        }
    }

    fn start_insecure(&self, port: u16) -> Imap {
        // we pass in the domain twice to check that the server's TLS
        // certificate is valid for the domain we're connecting to.
        let client = imap::connect_insecure((self.server.as_str(), port))
            .unwrap_or_else(|_| panic!("Couldn't connect to {}:{}", self.server, port));

        // the client we have here is unauthenticated.
        // to do anything useful with the e-mails, we need to log in
        let imap_session = client
            .login(&self.user, &self.password)
            .unwrap_or_else(|_| {
                panic!(
                    "Couldn't isnecurely connect to {}:{} for login {}",
                    self.server, port, self.user
                )
            });

        debug!(
            "Successfully connected to INSECURE imap server {}",
            self.server
        );
        Imap::Insecured(imap_session)
    }

    fn start_secure(&self, port: u16) -> Imap {
        let tls = native_tls::TlsConnector::builder()
            .build()
            .expect("Couldn't create TLS connector");

        // we pass in the domain twice to check that the server's TLS
        // certificate is valid for the domain we're connecting to.
        let client = imap::connect((self.server.as_str(), port), &self.server, &tls)
            .unwrap_or_else(|_| panic!("Couldn't connect to {}:{}", self.server, port));

        // the client we have here is unauthenticated.
        // to do anything useful with the e-mails, we need to log in
        let imap_session = client
            .login(&self.user, &self.password)
            .unwrap_or_else(|_| {
                panic!(
                    "Couldn't securely connect to {}:{} for login {}",
                    self.server, port, self.user
                )
            });

        debug!(
            "Successfully connected to SECURE imap server {}",
            self.server
        );
        Imap::Secured(imap_session)
    }
}

/// Store-level config
#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    /// when set to true, no reading statis will be persisted.
    /// As a consequence, messages may be read more than once
    #[serde(
        skip_serializing_if = "Settings::is_false",
        default = "Settings::default_false"
    )]
    pub do_not_save: bool,
    /// inline all images as base64 data
/*    #[serde(
        skip_serializing_if = "Settings::is_false",
        default = "Settings::default_false"
    )]
    pub inline_image_as_data: bool,
*/    #[serde(default = "Email::default")]
    pub email: Email,
    #[serde(default = )]
    pub config: Config,
}

impl Settings {
    pub fn is_false(value: &bool) -> bool {
        !value
    }
    pub fn default_false() -> bool {
        false
    }
/*
    pub fn is_true(value: &bool) -> bool {
        !!value
    }
    pub fn default_true() -> bool {
        true
    }
*/
    pub fn default() -> Settings {
        Settings {
            do_not_save: false,
            email: Email::default(),
            config: Config::new(),
        }
    }
}
