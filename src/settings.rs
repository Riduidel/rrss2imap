use imap::Session;
use imap::error::Result;

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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Email {
    server: String,
    user: String,
    password: String,
    #[serde(default = "Email::default_secure")]
    secure: Secure,
}

pub enum Imap {
    Secured(Session<native_tls::TlsStream<std::net::TcpStream>>),
    Insecured(Session<std::net::TcpStream>)
}

impl Imap {
    pub fn append<S: AsRef<str>, B: AsRef<[u8]>>(&mut self, mailbox: S, content: B) -> Result<()> {
        match self {
            Imap::Secured(ref mut session) => session.append(mailbox, content),
            Imap::Insecured(ref mut session) => session.append(mailbox, content),
        }
    }
}

impl Email {
    pub fn default_secure() -> Secure {
        Secure::Yes(993)
    }
    pub fn default() -> Email {
        Email {
            server: "Set your email server address here".to_owned(),
            user: "Set your imap server user name (it may be your email address or not)".to_owned(),
            password: "Set your imap server password (yup, in clear, this is very bad)".to_owned(),
            secure: Secure::Yes(993),
        }
    }

    pub fn start(&self) -> Imap {
        match self.secure {
            Secure::Yes(port) => self.start_secure(port),
            Secure::No(port) => self.start_insecure(port),
        }
    }

    pub fn start_insecure(&self, port: u16) -> Imap {
        // we pass in the domain twice to check that the server's TLS
        // certificate is valid for the domain we're connecting to.
        let client = imap::connect_insecure((self.server.as_str(), port))
            .unwrap_or_else(|_| panic!("Couldn't connect to {}:{}", self.server, port));

        // the client we have here is unauthenticated.
        // to do anything useful with the e-mails, we need to log in
        let imap_session = client
            .login(&self.user, &self.password)
            .unwrap_or_else(|_| panic!("Couldn't isnecurely connect to {}:{} for login {}", self.server, port, self.user));
        
        info!("Successfully connected to INSECURE imap server {}", self.server);
        Imap::Insecured(imap_session)
    }

    pub fn start_secure(&self, port: u16) -> Imap {
        let tls = native_tls::TlsConnector::builder().build()
            .expect("Couldn't create TLS connector");

        // we pass in the domain twice to check that the server's TLS
        // certificate is valid for the domain we're connecting to.
        let client = imap::connect((self.server.as_str(), port), &self.server, &tls)
            .unwrap_or_else(|_| panic!("Couldn't connect to {}:{}", self.server, port));

        // the client we have here is unauthenticated.
        // to do anything useful with the e-mails, we need to log in
        let imap_session = client
            .login(&self.user, &self.password)
            .unwrap_or_else(|_| panic!("Couldn't securely connect to {}:{} for login {}", self.server, port, self.user));
        
        info!("Successfully connected to SECURE imap server {}", self.server);
        Imap::Secured(imap_session)
    }
}

/// Store-level config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    #[serde(
        skip_serializing_if = "Settings::is_false",
        default = "Settings::default_false"
    )]
    pub do_not_save: bool,
    #[serde(
        skip_serializing_if = "Settings::is_false",
        default = "Settings::default_false"
    )]
    pub inline_image_as_data: bool,
    #[serde(default = "Email::default")]
    pub email: Email,
}

impl Settings {
    pub fn is_false(value: &bool) -> bool {
        !value
    }
    pub fn default_false() -> bool {
        false
    }

    pub fn is_true(value: &bool) -> bool {
        !!value
    }
    pub fn default_true() -> bool {
        true
    }

    pub fn default() -> Settings {
        Settings {
            do_not_save: false,
            inline_image_as_data: false,
            email: Email::default(),
        }
    }

    pub fn connect(&self) -> Imap {
        self.email.start()
    }
}
