use std::{mem, path::Path, sync::Arc};

use gpui::{AppContext, Context};
use windows::{Win32::Foundation::HWND, core::HSTRING};
use zeroize::Zeroizing;

use super::hello::{self, KeyFile};
use magpie_engine::bitwarden::{
    Challenge, CipherId, Credentials, Error, Fetched, Method, NewLogin, Proof, Server, SignIn,
    Status, SymmetricKey, Vault,
};

pub struct Account {
    pub server: Server,
    pub email: String,
    pub password: Zeroizing<String>,
}

pub struct Entry {
    pub name: String,
    pub username: String,
    pub password: Zeroizing<String>,
    pub uri: String,
}

pub struct Pending {
    challenge: Arc<Challenge>,
    email_sent: bool,
}

pub enum Problem {
    Failed(String),
    WrongCode,
    HelloExpired,
}

pub enum Phase<'a> {
    Busy,
    Unavailable(&'a str),
    SignedOut,
    Challenged(&'a Pending),
    Locked(&'a Vault),
    Unlocked(&'a Vault),
}

enum Slot {
    Ready(Box<Vault>),
    Busy,
    Unavailable(String),
}

struct Hello {
    file: KeyFile,
    available: bool,
    on: bool,
}

pub struct Keyring {
    slot: Slot,
    pending: Option<Pending>,
    problem: Option<Problem>,
    hello: Hello,
}

impl Pending {
    fn new(challenge: Challenge) -> Self {
        Self {
            challenge: Arc::new(challenge),
            email_sent: false,
        }
    }

    pub fn methods(&self) -> &[Method] {
        self.challenge.methods()
    }

    pub fn email_sent(&self) -> bool {
        self.email_sent
    }
}

impl Keyring {
    pub fn open(folder: &Path, cx: &mut Context<Self>) -> Self {
        let slot = match Vault::open(folder) {
            Ok(vault) => Slot::Ready(Box::new(vault)),
            Err(error) => Slot::Unavailable(error.to_string()),
        };
        let file = KeyFile::in_folder(folder);
        let hello = Hello {
            on: file.exists(),
            file,
            available: false,
        };
        Self::check_hello(cx);
        Self {
            slot,
            pending: None,
            problem: None,
            hello,
        }
    }

    fn check_hello(cx: &mut Context<Self>) {
        cx.spawn(async move |keyring, cx| {
            let available = hello::available().await;
            keyring
                .update(cx, |keyring, cx| {
                    keyring.hello.available = available;
                    cx.notify();
                })
                .ok();
        })
        .detach();
    }

    pub fn hello_available(&self) -> bool {
        self.hello.available
    }

    pub fn hello_on(&self) -> bool {
        self.hello.on
    }

    pub fn hello_ready(&self) -> bool {
        self.hello.available && self.hello.on
    }

    pub fn phase(&self) -> Phase<'_> {
        match (&self.slot, &self.pending) {
            (Slot::Busy, _) => Phase::Busy,
            (Slot::Unavailable(message), _) => Phase::Unavailable(message),
            (Slot::Ready(_), Some(pending)) => Phase::Challenged(pending),
            (Slot::Ready(vault), None) => match vault.status() {
                Status::SignedOut => Phase::SignedOut,
                Status::Locked => Phase::Locked(vault),
                Status::Unlocked => Phase::Unlocked(vault),
            },
        }
    }

    pub fn problem(&self) -> Option<&Problem> {
        self.problem.as_ref()
    }

    pub fn report(&mut self, error: &Error, cx: &mut Context<Self>) {
        self.problem = Some(Problem::Failed(error.to_string()));
        cx.notify();
    }

    fn run<T: Send + 'static>(
        &mut self,
        job: impl FnOnce(&mut Vault) -> Result<T, Error> + Send + 'static,
        done: impl FnOnce(&mut Self, T, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) {
        let slot = mem::replace(&mut self.slot, Slot::Busy);
        let Slot::Ready(mut vault) = slot else {
            self.slot = slot;
            return;
        };
        self.problem = None;
        let work = cx.background_spawn(async move {
            let result = job(&mut vault);
            (vault, result)
        });
        cx.spawn(async move |keyring, cx| {
            let (vault, result) = work.await;
            keyring
                .update(cx, |keyring, cx| {
                    keyring.slot = Slot::Ready(vault);
                    match result {
                        Ok(value) => done(keyring, value, cx),
                        Err(error) => keyring.problem = Some(Problem::Failed(error.to_string())),
                    }
                    cx.notify();
                })
                .ok();
        })
        .detach();
        cx.notify();
    }

    fn signed_in(&mut self, outcome: SignIn) {
        self.pending = match outcome {
            SignIn::Unlocked => None,
            SignIn::Challenged(challenge) => Some(Pending::new(challenge)),
        };
    }

    fn answered(&mut self, outcome: SignIn) {
        if matches!(outcome, SignIn::Challenged(_)) {
            self.problem = Some(Problem::WrongCode);
        }
        self.signed_in(outcome);
    }

    pub fn sign_in(&mut self, account: Account, cx: &mut Context<Self>) {
        let job = move |vault: &mut Vault| {
            vault.sign_in(&Credentials {
                server: account.server,
                email: &account.email,
                password: &account.password,
            })
        };
        self.run(job, |keyring, outcome, _| keyring.signed_in(outcome), cx);
    }

    pub fn answer(&mut self, proof: Proof, cx: &mut Context<Self>) {
        let Some(pending) = &self.pending else {
            return;
        };
        let challenge = Arc::clone(&pending.challenge);
        let job = move |vault: &mut Vault| vault.answer(&challenge, &proof);
        self.run(job, |keyring, outcome, _| keyring.answered(outcome), cx);
    }

    pub fn send_email_code(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = &self.pending else {
            return;
        };
        let challenge = Arc::clone(&pending.challenge);
        let job = move |vault: &mut Vault| vault.send_email_code(&challenge);
        let sent = |keyring: &mut Self, (), _: &mut Context<Self>| {
            if let Some(pending) = &mut keyring.pending {
                pending.email_sent = true;
            }
        };
        self.run(job, sent, cx);
    }

    pub fn cancel_sign_in(&mut self, cx: &mut Context<Self>) {
        self.pending = None;
        self.problem = None;
        cx.notify();
    }

    pub fn unlock(&mut self, password: Zeroizing<String>, cx: &mut Context<Self>) {
        let job = move |vault: &mut Vault| vault.unlock(&password);
        let unlocked = |keyring: &mut Self, (), cx: &mut Context<Self>| {
            keyring.refresh_hello(cx);
            keyring.sync_quietly(cx);
        };
        self.run(job, unlocked, cx);
    }

    pub fn unlock_with_hello(&mut self, window: HWND, reason: HSTRING, cx: &mut Context<Self>) {
        if !self.hello_ready() {
            return;
        }
        let verified = hello::verify(window, reason);
        cx.spawn(async move |keyring, cx| {
            if verified.await {
                keyring
                    .update(cx, |keyring, cx| keyring.open_with_stored_key(cx))
                    .ok();
            }
        })
        .detach();
    }

    fn open_with_stored_key(&mut self, cx: &mut Context<Self>) {
        let file = self.hello.file.clone();
        let job = move |vault: &mut Vault| {
            let key = SymmetricKey::from_bytes(&file.load()?)?;
            match vault.unlock_with_key(key) {
                Ok(()) => Ok(true),
                Err(Error::Tampered) => Ok(false),
                Err(error) => Err(error),
            }
        };
        let opened = |keyring: &mut Self, opened: bool, cx: &mut Context<Self>| match opened {
            true => keyring.sync_quietly(cx),
            false => keyring.problem = Some(Problem::HelloExpired),
        };
        self.run(job, opened, cx);
    }

    fn key(&self) -> Option<Zeroizing<Vec<u8>>> {
        let Slot::Ready(vault) = &self.slot else {
            return None;
        };
        vault.key().ok().map(SymmetricKey::bytes)
    }

    fn store_key(&mut self, key: Zeroizing<Vec<u8>>, cx: &mut Context<Self>) {
        let file = self.hello.file.clone();
        let stored = cx.background_spawn(async move { file.store(&key) });
        cx.spawn(async move |keyring, cx| {
            let result = stored.await;
            keyring
                .update(cx, |keyring, cx| {
                    match result {
                        Ok(()) => keyring.hello.on = true,
                        Err(error) => keyring.problem = Some(Problem::Failed(error.to_string())),
                    }
                    cx.notify();
                })
                .ok();
        })
        .detach();
    }

    fn refresh_hello(&mut self, cx: &mut Context<Self>) {
        if !self.hello.on {
            return;
        }
        if let Some(key) = self.key() {
            self.store_key(key, cx);
        }
    }

    pub fn turn_on_hello(&mut self, window: HWND, reason: HSTRING, cx: &mut Context<Self>) {
        let Some(key) = self.key().filter(|_| self.hello.available) else {
            return;
        };
        let verified = hello::verify(window, reason);
        cx.spawn(async move |keyring, cx| {
            if verified.await {
                keyring
                    .update(cx, |keyring, cx| keyring.store_key(key, cx))
                    .ok();
            }
        })
        .detach();
    }

    pub fn turn_off_hello(&mut self, cx: &mut Context<Self>) {
        self.problem = self
            .hello
            .file
            .forget()
            .err()
            .map(|error| Problem::Failed(error.to_string()));
        self.hello.on = false;
        cx.notify();
    }

    pub fn sync(&mut self, cx: &mut Context<Self>) {
        self.run(Vault::sync, |_, (), _| {}, cx);
    }

    fn sync_quietly(&mut self, cx: &mut Context<Self>) {
        let Slot::Ready(vault) = &self.slot else {
            return;
        };
        let Ok(pull) = vault.pull() else {
            return;
        };
        let work = cx.background_spawn(async move { pull.fetch() });
        cx.spawn(async move |keyring, cx| {
            let Ok(fetched) = work.await else {
                return;
            };
            keyring
                .update(cx, |keyring, cx| keyring.absorb(fetched, cx))
                .ok();
        })
        .detach();
    }

    fn absorb(&mut self, fetched: Fetched, cx: &mut Context<Self>) {
        let Slot::Ready(vault) = &mut self.slot else {
            return;
        };
        if vault.absorb(fetched).is_ok() {
            cx.notify();
        }
    }

    pub fn create_login(&mut self, entry: Entry, cx: &mut Context<Self>) {
        let job = move |vault: &mut Vault| {
            vault.create_login(&NewLogin {
                name: &entry.name,
                username: &entry.username,
                password: &entry.password,
                uri: &entry.uri,
            })
        };
        self.run(job, |_, (), _| {}, cx);
    }

    pub fn change_password(
        &mut self,
        id: CipherId,
        password: Zeroizing<String>,
        cx: &mut Context<Self>,
    ) {
        let job = move |vault: &mut Vault| vault.change_password(&id, &password);
        self.run(job, |_, (), _| {}, cx);
    }

    pub fn lock(&mut self, cx: &mut Context<Self>) {
        if let Slot::Ready(vault) = &mut self.slot {
            vault.lock();
        }
        self.problem = None;
        cx.notify();
    }

    pub fn sign_out(&mut self, cx: &mut Context<Self>) {
        let Slot::Ready(vault) = &mut self.slot else {
            return;
        };
        let signed_out = vault.sign_out().map_err(|error| error.to_string());
        let forgotten = self.hello.file.forget().map_err(|error| error.to_string());
        self.problem = signed_out.and(forgotten).err().map(Problem::Failed);
        self.hello.on = false;
        self.pending = None;
        cx.notify();
    }
}
