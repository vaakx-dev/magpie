use std::{
    fmt, io,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, OnceLock},
    thread,
};

use magpie_engine::bitwarden::{
    Challenge, Credentials, Error, Login, Method, Proof, Server, SignIn, Status, SymmetricKey,
    Totp, Vault,
};
use serde::Serialize;
use zeroize::Zeroizing;

#[derive(Serialize)]
#[serde(untagged)]
enum Reply<T> {
    Done(T),
    Failed { error: String },
}

#[derive(Serialize)]
struct Account<'a> {
    state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<String>,
    methods: &'a [&'static str],
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct Entry<'a> {
    id: &'a str,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    totp: Option<&'a str>,
}

#[derive(Serialize)]
struct Code {
    code: String,
    remaining: u64,
}

struct Keyring {
    vault: Vault,
    challenge: Option<Pending>,
}

struct Pending {
    challenge: Challenge,
    methods: Vec<&'static str>,
}

enum Problem {
    Failed(Error),
    WrongCode,
}

const WRONG_CODE: &str = "That code didn't work. Check it and try again.";
const SIGNED_OUT: Account<'static> = Account {
    state: "signedOut",
    email: None,
    server: None,
    methods: &[],
    error: None,
};
const NOT_STARTED: &str = "Magpie's core wasn't started";

static FOLDER: OnceLock<PathBuf> = OnceLock::new();
static KEYRING: Mutex<Option<Keyring>> = Mutex::new(None);

fn slot() -> MutexGuard<'static, Option<Keyring>> {
    KEYRING.lock().expect("vault lock poisoned")
}

fn folder() -> Result<&'static Path, Error> {
    FOLDER
        .get()
        .map(PathBuf::as_path)
        .ok_or_else(|| Error::Storage(io::Error::other(NOT_STARTED)))
}

fn opened(slot: &mut Option<Keyring>) -> Result<&mut Keyring, Error> {
    let keyring = match slot.take() {
        Some(keyring) => keyring,
        None => Keyring {
            vault: Vault::open(folder()?)?,
            challenge: None,
        },
    };
    Ok(slot.insert(keyring))
}

fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).expect("replies always serialize")
}

fn reply<T: Serialize>(result: Result<T, Error>) -> String {
    json(&match result {
        Ok(value) => Reply::Done(value),
        Err(error) => Reply::Failed {
            error: error.to_string(),
        },
    })
}

fn state_name(status: Status) -> &'static str {
    match status {
        Status::SignedOut => "signedOut",
        Status::Locked => "locked",
        Status::Unlocked => "unlocked",
    }
}

fn method_name(method: &Method) -> &'static str {
    match method {
        Method::Authenticator => "authenticator",
        Method::Email => "email",
        Method::YubiKey => "yubiKey",
        Method::NewDevice => "newDevice",
    }
}

fn method_named(name: &str) -> Result<Method, Error> {
    match name {
        "authenticator" => Ok(Method::Authenticator),
        "email" => Ok(Method::Email),
        "yubiKey" => Ok(Method::YubiKey),
        "newDevice" => Ok(Method::NewDevice),
        _ => Err(Error::Unsupported(format!("two-step method {name}"))),
    }
}

fn challenge(pending: &Option<Pending>) -> Result<&Challenge, Error> {
    pending
        .as_ref()
        .map(|pending| &pending.challenge)
        .ok_or(Error::SignedOut)
}

fn pending(challenge: Challenge) -> Pending {
    Pending {
        methods: challenge.methods().iter().map(method_name).collect(),
        challenge,
    }
}

impl From<Error> for Problem {
    fn from(error: Error) -> Self {
        Self::Failed(error)
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Failed(error) => error.fmt(formatter),
            Self::WrongCode => formatter.write_str(WRONG_CODE),
        }
    }
}

impl Keyring {
    fn account(&self, error: Option<String>) -> String {
        let (state, methods) = match &self.challenge {
            Some(pending) => ("challenged", pending.methods.as_slice()),
            None => (state_name(self.vault.status()), &[][..]),
        };
        json(&Account {
            state,
            email: self.vault.email(),
            server: self.vault.server().map(Server::address),
            methods,
            error,
        })
    }

    fn settle(&mut self, result: Result<SignIn, Error>) -> Result<(), Error> {
        self.challenge = match result? {
            SignIn::Unlocked => None,
            SignIn::Challenged(challenge) => Some(pending(challenge)),
        };
        Ok(())
    }

    fn sign_in(&mut self, server: &str, email: &str, password: &str) -> Result<(), Error> {
        let server = Server::parse(server)?;
        let result = self.vault.sign_in(&Credentials {
            server,
            email,
            password,
        });
        self.settle(result)
    }

    fn answer(&mut self, method: &str, code: &str) -> Result<(), Problem> {
        let proof = Proof {
            method: method_named(method)?,
            code: Zeroizing::new(code.trim().to_owned()),
        };
        let result = self.vault.answer(challenge(&self.challenge)?, &proof);
        self.settle(result)?;
        match self.challenge {
            Some(_) => Err(Problem::WrongCode),
            None => Ok(()),
        }
    }

    fn send_email_code(&self) -> Result<(), Error> {
        self.vault.send_email_code(challenge(&self.challenge)?)
    }

    fn unlock(&mut self, password: &str) -> Result<(), Error> {
        self.vault.unlock(password)?;
        sync_quietly();
        Ok(())
    }

    fn unlock_with_key(&mut self, key: &[u8]) -> Result<(), Error> {
        self.vault.unlock_with_key(SymmetricKey::from_bytes(key)?)?;
        sync_quietly();
        Ok(())
    }

    fn lock(&mut self) {
        self.challenge = None;
        self.vault.lock();
    }

    fn sign_out(&mut self) -> Result<(), Error> {
        self.challenge = None;
        self.vault.sign_out()
    }
}

fn account_after<P: fmt::Display>(action: impl FnOnce(&mut Keyring) -> Result<(), P>) -> String {
    let mut slot = slot();
    match opened(&mut slot) {
        Ok(keyring) => {
            let problem = action(keyring).err().map(|problem| problem.to_string());
            keyring.account(problem)
        }
        Err(error) => json(&Account {
            error: Some(error.to_string()),
            ..SIGNED_OUT
        }),
    }
}

fn entry(login: &Login) -> Entry<'_> {
    Entry {
        id: &login.id.0,
        name: &login.name,
        username: login.username.as_deref(),
        password: login.password.as_deref(),
        totp: login.totp.as_deref(),
    }
}

fn entries(find: impl Fn(&Vault) -> Vec<&Login>) -> String {
    let slot = slot();
    let logins = slot
        .as_ref()
        .map(|keyring| find(&keyring.vault))
        .unwrap_or_default();
    let listed: Vec<Entry> = logins.into_iter().map(entry).collect();
    json(&listed)
}

fn sync_quietly() {
    thread::spawn(|| {
        let Some(Ok(pull)) = slot().as_ref().map(|keyring| keyring.vault.pull()) else {
            return;
        };
        let Ok(fetched) = pull.fetch() else {
            return;
        };
        if let Some(keyring) = slot().as_mut() {
            keyring.vault.absorb(fetched).ok();
        }
    });
}

pub fn locate(folder: PathBuf) {
    FOLDER.set(folder).ok();
}

pub fn status() -> String {
    account_after(|_| Ok::<(), Error>(()))
}

pub fn sign_in(server: &str, email: &str, password: &str) -> String {
    account_after(|keyring| keyring.sign_in(server, email, password))
}

pub fn answer(method: &str, code: &str) -> String {
    account_after(|keyring| keyring.answer(method, code))
}

pub fn send_email_code() -> String {
    account_after(|keyring| keyring.send_email_code())
}

pub fn unlock(password: &str) -> String {
    account_after(|keyring| keyring.unlock(password))
}

pub fn unlock_with_key(key: &[u8]) -> String {
    account_after(|keyring| keyring.unlock_with_key(key))
}

pub fn key() -> Option<Zeroizing<Vec<u8>>> {
    let slot = slot();
    let vault = &slot.as_ref()?.vault;
    vault.key().ok().map(SymmetricKey::bytes)
}

pub fn sign_out() -> String {
    account_after(Keyring::sign_out)
}

pub fn lock() -> String {
    account_after(|keyring| {
        keyring.lock();
        Ok::<(), Error>(())
    })
}

pub fn matches(address: &str) -> String {
    entries(|vault| vault.matches_for_url(address))
}

pub fn search(query: &str) -> String {
    entries(|vault| vault.search(query))
}

pub fn totp(secret: &str) -> String {
    reply(Totp::parse(secret).map(|totp| {
        let code = totp.code();
        Code {
            code: code.digits,
            remaining: code.remaining_secs,
        }
    }))
}
