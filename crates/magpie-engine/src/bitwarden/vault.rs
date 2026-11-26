use std::{
    mem,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use zeroize::Zeroizing;

use super::{
    auth::{self, Grant, Method, PasswordGrant, Proof, Tokens},
    cipher_string::CipherString,
    client::{Client, EmailCode},
    device::DeviceId,
    error::Error,
    item::{Login, NewLogin, Sealed},
    kdf::Kdf,
    key::{MasterKey, SymmetricKey},
    server::Server,
    snapshot::Snapshot,
    sync,
    uri::Page,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    SignedOut,
    Locked,
    Unlocked,
}

pub struct Credentials<'a> {
    pub server: Server,
    pub email: &'a str,
    pub password: &'a str,
}

pub enum SignIn {
    Unlocked,
    Challenged(Challenge),
}

pub struct Challenge {
    methods: Vec<Method>,
    attempt: Box<Attempt>,
}

#[derive(Clone)]
struct Attempt {
    server: Server,
    email: String,
    kdf: Kdf,
    master_key: MasterKey,
    hash: Zeroizing<String>,
}

pub struct Pull {
    client: Client,
    server: Server,
    pass: Pass,
}

pub struct Fetched {
    renewed: Option<Tokens>,
    reply: sync::Reply,
}

enum Pass {
    Current(Zeroizing<String>),
    Renew(Zeroizing<String>),
}

struct Authorized {
    token: Zeroizing<String>,
    renewed: Option<Tokens>,
}

struct Access {
    token: Zeroizing<String>,
    expires: Instant,
}

struct Session {
    snapshot: Snapshot,
    key: SymmetricKey,
    logins: Vec<Login>,
    access: Option<Access>,
}

enum State {
    SignedOut,
    Locked(Snapshot),
    Unlocked(Session),
}

pub struct Vault {
    folder: PathBuf,
    device: DeviceId,
    client: Client,
    state: State,
}

const EXPIRY_MARGIN: Duration = Duration::from_secs(60);
const BAD_REQUEST: u16 = 400;
const UNAUTHORIZED: u16 = 401;

fn open_all(sealed: &[Sealed], key: &SymmetricKey) -> Result<Vec<Login>, Error> {
    sealed.iter().map(|item| item.open(key)).collect()
}

fn user_key(snapshot: &Snapshot, password: &str) -> Result<SymmetricKey, Error> {
    let master_key = snapshot.kdf.master_key(password, &snapshot.email)?;
    master_key
        .unseal(&snapshot.protected_key)
        .map_err(|error| match error {
            Error::Tampered => Error::WrongPassword,
            other => other,
        })
}

fn expired(error: Error) -> Error {
    match error {
        Error::Rejected {
            status: BAD_REQUEST | UNAUTHORIZED,
            ..
        } => Error::Expired,
        other => other,
    }
}

impl Challenge {
    pub fn methods(&self) -> &[Method] {
        &self.methods
    }
}

impl Access {
    fn new(tokens: &Tokens) -> Self {
        Self {
            token: tokens.access.clone(),
            expires: Instant::now() + Duration::from_secs(tokens.expires_secs),
        }
    }

    fn fresh(&self) -> bool {
        Instant::now() + EXPIRY_MARGIN < self.expires
    }
}

impl Pass {
    fn authorize(self, client: &Client, server: &Server) -> Result<Authorized, Error> {
        match self {
            Self::Current(token) => Ok(Authorized {
                token,
                renewed: None,
            }),
            Self::Renew(refresh_token) => {
                let reply = client
                    .token(server, &Grant::Refresh(&refresh_token))
                    .map_err(expired)?;
                let auth::Reply::Granted(tokens) = reply else {
                    return Err(Error::Expired);
                };
                Ok(Authorized {
                    token: tokens.access.clone(),
                    renewed: Some(tokens),
                })
            }
        }
    }
}

impl Pull {
    pub fn fetch(self) -> Result<Fetched, Error> {
        let authorized = self.pass.authorize(&self.client, &self.server)?;
        let reply = self.client.sync(&self.server, &authorized.token)?;
        Ok(Fetched {
            renewed: authorized.renewed,
            reply,
        })
    }
}

impl Session {
    fn pass(&self) -> Result<Pass, Error> {
        match &self.access {
            Some(access) if access.fresh() => Ok(Pass::Current(access.token.clone())),
            _ => {
                let refresh_token = self.snapshot.refresh_token.decrypt_text(&self.key)?;
                Ok(Pass::Renew(Zeroizing::new(refresh_token)))
            }
        }
    }

    fn renew(&mut self, renewed: Option<Tokens>, folder: &Path) -> Result<(), Error> {
        let Some(tokens) = renewed else {
            return Ok(());
        };
        self.access = Some(Access::new(&tokens));
        match &tokens.refresh {
            Some(rotated) => {
                self.snapshot.refresh_token = CipherString::encrypt(rotated.as_bytes(), &self.key)?;
                self.snapshot.save(folder)
            }
            None => Ok(()),
        }
    }

    fn access_token(&mut self, client: &Client, folder: &Path) -> Result<Zeroizing<String>, Error> {
        let authorized = self.pass()?.authorize(client, &self.snapshot.server)?;
        self.renew(authorized.renewed, folder)?;
        Ok(authorized.token)
    }

    fn absorb(&mut self, fetched: Fetched, folder: &Path) -> Result<(), Error> {
        self.renew(fetched.renewed, folder)?;
        self.logins = open_all(&fetched.reply.logins, &self.key)?;
        self.snapshot.logins = fetched.reply.logins;
        self.snapshot.save(folder)
    }

    fn create(&mut self, client: &Client, folder: &Path, login: &NewLogin) -> Result<(), Error> {
        let draft = login.seal(&self.key)?;
        let token = self.access_token(client, folder)?;
        let sealed = client.create(&self.snapshot.server, &token, &draft)?;
        self.logins.push(sealed.open(&self.key)?);
        self.snapshot.logins.push(sealed);
        self.snapshot.save(folder)
    }
}

impl State {
    fn session(&self) -> Result<&Session, Error> {
        match self {
            Self::Unlocked(session) => Ok(session),
            Self::Locked(_) => Err(Error::Locked),
            Self::SignedOut => Err(Error::SignedOut),
        }
    }

    fn session_mut(&mut self) -> Result<&mut Session, Error> {
        match self {
            Self::Unlocked(session) => Ok(session),
            Self::Locked(_) => Err(Error::Locked),
            Self::SignedOut => Err(Error::SignedOut),
        }
    }
}

impl Vault {
    pub fn open(folder: &Path) -> Result<Self, Error> {
        let device = DeviceId::load_or_create(folder)?;
        let state = Snapshot::load(folder)?.map_or(State::SignedOut, State::Locked);
        Ok(Self {
            folder: folder.to_owned(),
            device,
            client: Client::default(),
            state,
        })
    }

    pub fn status(&self) -> Status {
        match self.state {
            State::SignedOut => Status::SignedOut,
            State::Locked(_) => Status::Locked,
            State::Unlocked(_) => Status::Unlocked,
        }
    }

    fn snapshot(&self) -> Option<&Snapshot> {
        match &self.state {
            State::SignedOut => None,
            State::Locked(snapshot) => Some(snapshot),
            State::Unlocked(session) => Some(&session.snapshot),
        }
    }

    pub fn email(&self) -> Option<&str> {
        self.snapshot().map(|snapshot| snapshot.email.as_str())
    }

    pub fn server(&self) -> Option<&Server> {
        self.snapshot().map(|snapshot| &snapshot.server)
    }

    pub fn sign_in(&mut self, credentials: &Credentials) -> Result<SignIn, Error> {
        let email = credentials.email.trim().to_lowercase();
        let kdf = self.client.prelogin(&credentials.server, &email)?;
        let master_key = kdf.master_key(credentials.password, &email)?;
        let attempt = Attempt {
            server: credentials.server.clone(),
            hash: master_key.password_hash(credentials.password),
            email,
            kdf,
            master_key,
        };
        self.submit(attempt, None)
    }

    pub fn answer(&mut self, challenge: &Challenge, proof: &Proof) -> Result<SignIn, Error> {
        self.submit((*challenge.attempt).clone(), Some(proof))
    }

    pub fn send_email_code(&self, challenge: &Challenge) -> Result<(), Error> {
        let attempt = &challenge.attempt;
        let request = EmailCode {
            email: &attempt.email,
            hash: &attempt.hash,
            device: &self.device,
        };
        self.client.send_email_code(&attempt.server, &request)
    }

    fn submit(&mut self, attempt: Attempt, proof: Option<&Proof>) -> Result<SignIn, Error> {
        let grant = Grant::Password(PasswordGrant {
            email: &attempt.email,
            hash: &attempt.hash,
            device: &self.device,
            proof,
        });
        match self.client.token(&attempt.server, &grant)? {
            auth::Reply::Challenged(methods) => Ok(SignIn::Challenged(Challenge {
                methods,
                attempt: Box::new(attempt),
            })),
            auth::Reply::Granted(tokens) => {
                self.state = State::Unlocked(self.welcome(attempt, &tokens)?);
                Ok(SignIn::Unlocked)
            }
        }
    }

    fn welcome(&self, attempt: Attempt, tokens: &Tokens) -> Result<Session, Error> {
        let refresh_token = tokens
            .refresh
            .as_ref()
            .ok_or(Error::Malformed("token reply without a refresh token"))?;
        let reply = self.client.sync(&attempt.server, &tokens.access)?;
        let key = attempt.master_key.unseal(&reply.protected_key)?;
        let snapshot = Snapshot {
            server: attempt.server,
            email: attempt.email,
            kdf: attempt.kdf,
            protected_key: reply.protected_key,
            refresh_token: CipherString::encrypt(refresh_token.as_bytes(), &key)?,
            logins: reply.logins,
        };
        snapshot.save(&self.folder)?;
        Ok(Session {
            logins: open_all(&snapshot.logins, &key)?,
            snapshot,
            key,
            access: Some(Access::new(tokens)),
        })
    }

    pub fn unlock(&mut self, password: &str) -> Result<(), Error> {
        let snapshot = match &self.state {
            State::Locked(snapshot) => snapshot,
            State::Unlocked(_) => return Ok(()),
            State::SignedOut => return Err(Error::SignedOut),
        };
        let key = user_key(snapshot, password)?;
        let logins = open_all(&snapshot.logins, &key)?;
        if let State::Locked(snapshot) = mem::replace(&mut self.state, State::SignedOut) {
            self.state = State::Unlocked(Session {
                snapshot,
                key,
                logins,
                access: None,
            });
        }
        Ok(())
    }

    pub fn pull(&self) -> Result<Pull, Error> {
        let session = self.state.session()?;
        Ok(Pull {
            client: self.client.clone(),
            server: session.snapshot.server.clone(),
            pass: session.pass()?,
        })
    }

    pub fn absorb(&mut self, fetched: Fetched) -> Result<(), Error> {
        self.state.session_mut()?.absorb(fetched, &self.folder)
    }

    pub fn sync(&mut self) -> Result<(), Error> {
        let fetched = self.pull()?.fetch()?;
        self.absorb(fetched)
    }

    pub fn create_login(&mut self, login: &NewLogin) -> Result<(), Error> {
        self.state
            .session_mut()?
            .create(&self.client, &self.folder, login)
    }

    pub fn lock(&mut self) {
        self.state = match mem::replace(&mut self.state, State::SignedOut) {
            State::Unlocked(session) => State::Locked(session.snapshot),
            other => other,
        };
    }

    pub fn sign_out(&mut self) -> Result<(), Error> {
        self.state = State::SignedOut;
        Snapshot::delete(&self.folder)
    }

    pub fn logins(&self) -> &[Login] {
        match &self.state {
            State::Unlocked(session) => &session.logins,
            State::Locked(_) | State::SignedOut => &[],
        }
    }

    pub fn search(&self, query: &str) -> Vec<&Login> {
        let query = query.trim().to_lowercase();
        self.logins()
            .iter()
            .filter(|login| login.contains(&query))
            .collect()
    }

    pub fn matches_for_url(&self, address: &str) -> Vec<&Login> {
        match Page::parse(address) {
            Some(page) => self
                .logins()
                .iter()
                .filter(|login| login.matches(&page))
                .collect(),
            None => Vec::new(),
        }
    }
}
