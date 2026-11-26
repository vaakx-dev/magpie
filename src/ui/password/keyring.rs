use std::{mem, path::Path, sync::Arc};

use gpui::{AppContext, Context};
use zeroize::Zeroizing;

use magpie_engine::bitwarden::{
    Challenge, Credentials, Error, Fetched, Method, NewLogin, Proof, Server, SignIn, Status, Vault,
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

pub struct Keyring {
    slot: Slot,
    pending: Option<Pending>,
    problem: Option<Problem>,
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
    pub fn open(folder: &Path) -> Self {
        let slot = match Vault::open(folder) {
            Ok(vault) => Slot::Ready(Box::new(vault)),
            Err(error) => Slot::Unavailable(error.to_string()),
        };
        Self {
            slot,
            pending: None,
            problem: None,
        }
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
        self.run(job, |keyring, (), cx| keyring.sync_quietly(cx), cx);
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
        self.problem = vault
            .sign_out()
            .err()
            .map(|error| Problem::Failed(error.to_string()));
        self.pending = None;
        cx.notify();
    }
}
