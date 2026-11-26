use gpui::{App, ClipboardItem, Context, WeakEntity};
use windows::{Win32::Foundation::HWND, core::HSTRING};
use zeroize::Zeroizing;

use super::{
    clipboard,
    keyring::{Account, Keyring, Phase, Problem},
};
use crate::{
    internal::settings::{
        Challenge, Hello, Host, LoginRow, Method as MethodChoice, Part, PasswordAction, Passwords,
        Reply, Secret,
    },
    locale::{Locale, Text},
    ui::{browser::Browser, window},
    webview::{WebView, window_handle},
};
use magpie_engine::bitwarden::{CipherId, Login, Method, Proof, Server};

const RESULT_LIMIT: usize = 50;
const YUBIKEY: &str = "YubiKey";

fn method_value(method: Method) -> &'static str {
    match method {
        Method::Authenticator => "authenticator",
        Method::Email => "email",
        Method::NewDevice => "new_device",
        Method::YubiKey => "yubikey",
    }
}

fn method_name(method: Method, locale: Locale) -> &'static str {
    match method {
        Method::Authenticator => locale.text(Text::Authenticator),
        Method::Email | Method::NewDevice => locale.text(Text::Email),
        Method::YubiKey => YUBIKEY,
    }
}

fn problem_text(problem: Option<&Problem>, locale: Locale) -> Option<String> {
    Some(match problem? {
        Problem::Failed(message) => message.clone(),
        Problem::WrongCode => locale.text(Text::WrongCode).to_owned(),
        Problem::HelloExpired => locale.text(Text::HelloExpired).to_owned(),
    })
}

fn hello(keyring: &Keyring) -> Hello {
    Hello {
        available: keyring.hello_available(),
        on: keyring.hello_on(),
    }
}

pub fn passwords_state(keyring: &Keyring, locale: Locale) -> Passwords {
    let problem = problem_text(keyring.problem(), locale);
    match keyring.phase() {
        Phase::Busy => Passwords::Working,
        Phase::Unavailable(reason) => Passwords::Unavailable {
            reason: reason.to_owned(),
        },
        Phase::SignedOut => Passwords::SignedOut { problem },
        Phase::Challenged(pending) => Passwords::Challenged {
            challenge: Challenge {
                methods: pending
                    .methods()
                    .iter()
                    .map(|method| MethodChoice {
                        value: method_value(*method),
                        name: method_name(*method, locale),
                    })
                    .collect(),
                email_sent: pending.email_sent(),
            },
            problem,
        },
        Phase::Locked(vault) => Passwords::Locked {
            email: vault.email().map(str::to_owned),
            hello: hello(keyring),
            problem,
        },
        Phase::Unlocked(vault) => Passwords::Unlocked {
            email: vault.email().map(str::to_owned),
            logins: vault.logins().len(),
            hello: hello(keyring),
            problem,
        },
    }
}

fn server(host: Host, address: &str) -> Result<Server, magpie_engine::bitwarden::Error> {
    match host {
        Host::UnitedStates => Ok(Server::UnitedStates),
        Host::Europe => Ok(Server::Europe),
        Host::Custom => Server::parse(address),
    }
}

fn row(login: &Login) -> LoginRow {
    let site = login
        .uris
        .first()
        .map(|uri| uri.uri.clone())
        .unwrap_or_default();
    LoginRow {
        id: login.id.clone(),
        name: login.name.clone(),
        username: login.username.clone().unwrap_or_default(),
        site,
        password: login.password.is_some(),
        code: login.totp.is_some(),
    }
}

fn part(login: &Login, part: Part) -> Option<Zeroizing<String>> {
    match part {
        Part::Username => login.username.clone().map(Zeroizing::new),
        Part::Password => login.password.clone().map(Zeroizing::new),
        Part::Code => login
            .authenticator()
            .map(|totp| Zeroizing::new(totp.code().digits)),
    }
}

fn sign_in(host: Host, address: &str, email: &str, password: Secret, cx: &mut App) {
    let email = email.trim().to_owned();
    if email.is_empty() || password.0.is_empty() {
        return;
    }
    let server = server(host, address);
    window::keyring(cx).update(cx, |keyring, cx| match server {
        Ok(server) => keyring.sign_in(
            Account {
                server,
                email,
                password: password.0,
            },
            cx,
        ),
        Err(error) => keyring.report(&error, cx),
    });
}

fn answer(method: &str, code: Secret, cx: &mut App) {
    let keyring = window::keyring(cx);
    let chosen = match keyring.read(cx).phase() {
        Phase::Challenged(pending) => pending
            .methods()
            .iter()
            .copied()
            .find(|candidate| method_value(*candidate) == method),
        _ => None,
    };
    let Some(method) = chosen.filter(|_| !code.0.trim().is_empty()) else {
        return;
    };
    let proof = Proof {
        method,
        code: code.0,
    };
    keyring.update(cx, |keyring, cx| keyring.answer(proof, cx));
}

fn copy(id: &CipherId, wanted: Part, cx: &mut App) {
    let keyring = window::keyring(cx);
    let value = match keyring.read(cx).phase() {
        Phase::Unlocked(vault) => vault.login(id).and_then(|login| part(login, wanted)),
        _ => None,
    };
    let Some(value) = value else {
        return;
    };
    match wanted {
        Part::Username => cx.write_to_clipboard(ClipboardItem::new_string(value.to_string())),
        Part::Password | Part::Code => clipboard::copy_secret(value, cx),
    }
}

impl Browser {
    fn with_window_handle(
        &self,
        then: impl FnOnce(HWND, HSTRING, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let window = self.runtime.window();
        let reason = HSTRING::from(self.text(Text::HelloReason));
        cx.defer(move |cx| {
            let handle = window
                .update(cx, |_, window, _| window_handle(window))
                .ok()
                .and_then(Result::ok);
            if let Some(handle) = handle {
                then(handle, reason, cx);
            }
        });
    }

    fn reply_logins(&self, view: &WeakEntity<WebView>, query: &str, cx: &mut Context<Self>) {
        let keyring = window::keyring(cx);
        let rows = match keyring.read(cx).phase() {
            Phase::Unlocked(vault) => vault
                .search(query)
                .into_iter()
                .take(RESULT_LIMIT)
                .map(row)
                .collect(),
            _ => Vec::new(),
        };
        let json = Reply::logins(rows).json();
        if let Some(view) = view.upgrade() {
            view.read(cx).post(&json);
        }
    }

    pub(in crate::ui) fn answer_passwords(
        &mut self,
        view: WeakEntity<WebView>,
        action: PasswordAction,
        cx: &mut Context<Self>,
    ) {
        let keyring = window::keyring(cx);
        match action {
            PasswordAction::SignIn {
                host,
                address,
                email,
                password,
            } => sign_in(host, &address, &email, password, cx),
            PasswordAction::AnswerChallenge { method, code } => answer(&method, code, cx),
            PasswordAction::SendEmailCode => keyring.update(cx, Keyring::send_email_code),
            PasswordAction::CancelSignIn => keyring.update(cx, Keyring::cancel_sign_in),
            PasswordAction::Unlock { password } => {
                keyring.update(cx, |keyring, cx| keyring.unlock(password.0, cx))
            }
            PasswordAction::UnlockWithHello => self.with_window_handle(
                move |handle, reason, cx| {
                    keyring.update(cx, |keyring, cx| {
                        keyring.unlock_with_hello(handle, reason, cx)
                    })
                },
                cx,
            ),
            PasswordAction::Lock => keyring.update(cx, Keyring::lock),
            PasswordAction::Sync => keyring.update(cx, Keyring::sync),
            PasswordAction::SignOut => keyring.update(cx, Keyring::sign_out),
            PasswordAction::SetHello { on: true } => self.with_window_handle(
                move |handle, reason, cx| {
                    keyring.update(cx, |keyring, cx| keyring.turn_on_hello(handle, reason, cx))
                },
                cx,
            ),
            PasswordAction::SetHello { on: false } => keyring.update(cx, Keyring::turn_off_hello),
            PasswordAction::Search { query } => self.reply_logins(&view, &query, cx),
            PasswordAction::Copy { id, part } => copy(&id, part, cx),
        }
    }
}
