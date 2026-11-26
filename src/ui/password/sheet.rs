use std::time::Duration;

use gpui::{
    AnyElement, Context, Div, Entity, EventEmitter, Focusable, KeyBinding, MouseButton,
    SharedString, Stateful, Subscription, Task, Window, actions, div, prelude::*, px, rgba,
};
use url::Url;
use zeroize::Zeroizing;

use super::{
    clipboard, fill,
    keyring::{Keyring, Phase},
    look::{Emphasis, Look, MASKED_PASSWORD, login_name},
};
use crate::{
    locale::Text,
    ui::{field::Field, icon::Icon},
    webview::{WebView, autofill::fillable, window_handle},
};
use magpie_engine::{
    bitwarden::{CipherId, Login, Vault, password},
    blocker::site,
};

actions!(passwords, [Submit]);

pub enum Outcome {
    Close,
    Manage,
}

pub struct Options {
    pub keyring: Entity<Keyring>,
    pub page: Option<Entity<WebView>>,
    pub look: Look,
}

pub struct Sheet {
    keyring: Entity<Keyring>,
    page: Option<Entity<WebView>>,
    look: Look,
    password: Entity<Field>,
    generated: Option<Zeroizing<String>>,
    _keyring: Subscription,
    _clock: Task<()>,
}

const CONTEXT: &str = "Passwords";
const WIDTH_PX: f32 = 340.0;
const INPUT_HEIGHT_PX: f32 = 32.0;
const INPUT_CORNER_PX: f32 = 8.0;
const TICK: Duration = Duration::from_secs(1);

pub fn bindings() -> Vec<KeyBinding> {
    vec![KeyBinding::new("enter", Submit, Some(CONTEXT))]
}

fn clock(cx: &mut Context<Sheet>) -> Task<()> {
    cx.spawn(async move |sheet, cx| {
        loop {
            cx.background_executor().timer(TICK).await;
            if sheet.update(cx, |_, cx| cx.notify()).is_err() {
                break;
            }
        }
    })
}

fn grouped(digits: &str) -> String {
    let (left, right) = digits.split_at(digits.len() / 2);
    format!("{left} {right}")
}

impl EventEmitter<Outcome> for Sheet {}

impl Sheet {
    pub fn new(options: Options, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let look = options.look;
        let password = cx.new(|cx| Field::secret(look.text(Text::MasterPassword), cx));
        let keyring = cx.observe(&options.keyring, |_, _, cx| cx.notify());
        if matches!(options.keyring.read(cx).phase(), Phase::Locked(_)) {
            password.focus_handle(cx).focus(window);
        }
        Self {
            keyring: options.keyring,
            page: options.page,
            look,
            password,
            generated: None,
            _keyring: keyring,
            _clock: clock(cx),
        }
    }

    fn page_url(&self, cx: &Context<Self>) -> Option<Url> {
        let url = self.page.as_ref()?.read(cx).url()?;
        fillable(url).then(|| url.clone())
    }

    fn unlock(&mut self, cx: &mut Context<Self>) {
        if self.password.read(cx).text().is_empty() {
            return;
        }
        let password = self.password.update(cx, Field::take);
        self.keyring
            .update(cx, |keyring, cx| keyring.unlock(password, cx));
    }

    fn unlock_with_hello(&mut self, window: &Window, cx: &mut Context<Self>) {
        let Ok(handle) = window_handle(window) else {
            return;
        };
        let reason = self.look.text(Text::HelloReason).into();
        self.keyring.update(cx, |keyring, cx| {
            keyring.unlock_with_hello(handle, reason, cx)
        });
    }

    fn submit(&mut self, _: &Submit, _: &mut Window, cx: &mut Context<Self>) {
        self.unlock(cx);
    }

    fn fill(&mut self, id: &CipherId, cx: &mut Context<Self>) {
        let Some(view) = self.page.clone() else {
            return;
        };
        let keyring = self.keyring.read(cx);
        let Phase::Unlocked(vault) = keyring.phase() else {
            return;
        };
        let Some(login) = vault.login(id).cloned() else {
            return;
        };
        fill::sign_in(&view, &login, cx);
        cx.emit(Outcome::Close);
    }

    fn copy_code(&mut self, id: &CipherId, cx: &mut Context<Self>) {
        let keyring = self.keyring.read(cx);
        let Phase::Unlocked(vault) = keyring.phase() else {
            return;
        };
        let code = vault
            .login(id)
            .and_then(Login::authenticator)
            .map(|totp| Zeroizing::new(totp.code().digits));
        if let Some(code) = code {
            clipboard::copy_secret(code, cx);
        }
    }

    fn generate(&mut self, cx: &mut Context<Self>) {
        self.generated = password::generate().ok();
        cx.notify();
    }

    fn use_generated(&mut self, cx: &mut Context<Self>) {
        let (Some(view), Some(generated)) = (self.page.clone(), self.generated.take()) else {
            return;
        };
        fill::new_password(&view, &generated, cx);
        cx.emit(Outcome::Close);
    }

    fn header(&self, phase: &Phase, cx: &Context<Self>) -> Div {
        let look = self.look;
        let title = self
            .page_url(cx)
            .as_ref()
            .and_then(site)
            .unwrap_or_else(|| look.text(Text::Passwords).to_owned());
        let detail = match phase {
            Phase::Unlocked(vault) => match self.page_url(cx) {
                Some(page) => look.fill(
                    Text::SavedLogins,
                    &vault.matches_for_url(page.as_str()).len().to_string(),
                ),
                None => look.fill(Text::SavedLogins, &vault.logins().len().to_string()),
            },
            Phase::Locked(_) => look.text(Text::VaultLocked).to_owned(),
            Phase::Busy => look.text(Text::Working).to_owned(),
            Phase::SignedOut | Phase::Challenged(_) | Phase::Unavailable(_) => {
                look.text(Text::SignedOut).to_owned()
            }
        };
        div()
            .flex()
            .items_center()
            .gap_3()
            .pl_3p5()
            .pr_2()
            .py_1p5()
            .child(look.avatar(Icon::Globe.tinted(rgba(look.colors.text))))
            .child(look.titled(title, detail))
            .child(
                look.icon_button("close", Icon::X.tinted(rgba(look.colors.muted)))
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(Outcome::Close))),
            )
    }

    fn input(&self, cx: &Context<Self>) -> Div {
        let focus = self.password.focus_handle(cx);
        div()
            .flex_1()
            .min_w_0()
            .h(px(INPUT_HEIGHT_PX))
            .px_3()
            .flex()
            .items_center()
            .rounded(px(INPUT_CORNER_PX))
            .bg(rgba(self.look.colors.raised))
            .cursor_text()
            .on_mouse_down(MouseButton::Left, move |_, window, _| focus.focus(window))
            .child(self.password.clone())
    }

    fn hello_button(&self, cx: &Context<Self>) -> Option<Stateful<Div>> {
        let look = self.look;
        self.keyring.read(cx).hello_ready().then(|| {
            look.button("hello", look.text(Text::UnlockWithHello), Emphasis::Primary)
                .child(Icon::ScanFace.tinted(rgba(look.colors.on_text)))
                .flex_row_reverse()
                .on_click(cx.listener(|sheet, _, window, cx| sheet.unlock_with_hello(window, cx)))
        })
    }

    fn locked(&self, cx: &Context<Self>) -> Div {
        let look = self.look;
        let emphasis = match self.keyring.read(cx).hello_ready() {
            true => Emphasis::Secondary,
            false => Emphasis::Primary,
        };
        div()
            .px_3()
            .py_1()
            .flex()
            .flex_col()
            .gap_2()
            .children(self.hello_button(cx))
            .child(
                div().flex().gap_2().child(self.input(cx)).child(
                    look.button("unlock", look.text(Text::Unlock), emphasis)
                        .on_click(cx.listener(|sheet, _, _, cx| sheet.unlock(cx))),
                ),
            )
            .children(look.problem(self.keyring.read(cx).problem()))
    }

    fn signed_out(&self, cx: &Context<Self>) -> Stateful<Div> {
        let look = self.look;
        look.row("sign-in")
            .child(look.avatar(Icon::KeyRound.tinted(rgba(look.colors.text))))
            .child(look.titled(
                look.text(Text::SignInToBitwarden),
                look.text(Text::SignInHint),
            ))
            .child(Icon::ChevronRight.tinted(rgba(look.colors.muted)))
            .on_click(cx.listener(|_, _, _, cx| cx.emit(Outcome::Manage)))
    }

    fn login_row(&self, index: usize, login: &Login, cx: &Context<Self>) -> Stateful<Div> {
        let look = self.look;
        let id = login.id.clone();
        look.row(("login", index))
            .child(look.initial(login_name(login)))
            .child(look.titled(login_name(login).to_owned(), MASKED_PASSWORD))
            .child(look.button(("fill", index), look.text(Text::Fill), Emphasis::Primary))
            .on_click(cx.listener(move |sheet, _, _, cx| sheet.fill(&id, cx)))
    }

    fn code_row(&self, index: usize, login: &Login, cx: &Context<Self>) -> Option<Stateful<Div>> {
        let look = self.look;
        let code = login.authenticator()?.code();
        let id = login.id.clone();
        let left = look.fill(Text::SecondsLeft, &code.remaining_secs.to_string());
        let detail = format!("{} · {left}", look.text(Text::Authenticator));
        Some(
            look.row(("code", index))
                .child(look.avatar(Icon::Lock.tinted(rgba(look.colors.text))))
                .child(
                    look.titled(grouped(&code.digits), detail)
                        .font_family("Cascadia Mono"),
                )
                .child(
                    look.button(("copy", index), look.text(Text::Copy), Emphasis::Secondary)
                        .child(Icon::Copy.tinted(rgba(look.colors.muted)))
                        .flex_row_reverse(),
                )
                .on_click(cx.listener(move |sheet, _, _, cx| sheet.copy_code(&id, cx))),
        )
    }

    fn unlocked(&self, vault: &Vault, page: Option<&Url>, cx: &Context<Self>) -> Vec<AnyElement> {
        let found = page
            .map(|page| vault.matches_for_url(page.as_str()))
            .unwrap_or_default();
        if found.is_empty() {
            return vec![
                self.look
                    .note(self.look.text(Text::NoLoginsForSite))
                    .into_any_element(),
            ];
        }
        let logins = found
            .iter()
            .enumerate()
            .map(|(index, login)| self.login_row(index, login, cx).into_any_element());
        let codes = found
            .iter()
            .enumerate()
            .filter_map(|(index, login)| self.code_row(index, login, cx))
            .map(IntoElement::into_any_element);
        logins.chain(codes).collect()
    }

    fn body(&self, phase: &Phase, cx: &Context<Self>) -> Vec<AnyElement> {
        let look = self.look;
        match phase {
            Phase::Busy => vec![look.note(look.text(Text::Working)).into_any_element()],
            Phase::Unavailable(message) => {
                vec![
                    look.note(SharedString::from(message.to_string()))
                        .into_any_element(),
                ]
            }
            Phase::SignedOut | Phase::Challenged(_) => {
                vec![self.signed_out(cx).into_any_element()]
            }
            Phase::Locked(_) => vec![self.locked(cx).into_any_element()],
            Phase::Unlocked(vault) => self.unlocked(vault, self.page_url(cx).as_ref(), cx),
        }
    }

    fn generator(&self, cx: &Context<Self>) -> AnyElement {
        let look = self.look;
        let Some(generated) = &self.generated else {
            return look
                .row("generate")
                .child(look.avatar(Icon::Sparkle.tinted(rgba(look.colors.text))))
                .child(look.titled(
                    look.text(Text::GeneratePassword),
                    look.text(Text::SavedAfterSignIn),
                ))
                .on_click(cx.listener(|sheet, _, _, cx| sheet.generate(cx)))
                .into_any_element();
        };
        div()
            .mx_2p5()
            .my_1()
            .pl_3()
            .pr_1p5()
            .py_1p5()
            .flex()
            .items_center()
            .gap_1p5()
            .rounded(px(INPUT_CORNER_PX))
            .bg(rgba(look.colors.raised))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_xs()
                    .font_family("Cascadia Mono")
                    .child(generated.to_string()),
            )
            .child(
                look.icon_button("regenerate", Icon::RotateCw.tinted(rgba(look.colors.muted)))
                    .on_click(cx.listener(|sheet, _, _, cx| sheet.generate(cx))),
            )
            .child(
                look.button("use-generated", look.text(Text::Use), Emphasis::Primary)
                    .on_click(cx.listener(|sheet, _, _, cx| sheet.use_generated(cx))),
            )
            .into_any_element()
    }

    fn footer(&self, cx: &Context<Self>) -> Stateful<Div> {
        let colors = self.look.colors;
        let text = colors.text;
        div()
            .id("manage")
            .px_4()
            .pt_2()
            .pb_1()
            .flex()
            .items_center()
            .justify_between()
            .text_xs()
            .text_color(rgba(colors.muted))
            .cursor_pointer()
            .hover(move |footer| footer.text_color(rgba(text)))
            .child(self.look.text(Text::ManageAllLogins))
            .child(Icon::ChevronRight.tinted(rgba(colors.muted)))
            .on_click(cx.listener(|_, _, _, cx| cx.emit(Outcome::Manage)))
    }
}

impl Render for Sheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let keyring = self.keyring.read(cx);
        let phase = keyring.phase();
        let look = self.look;
        let generating = self.page_url(cx).is_some() && !matches!(phase, Phase::Busy);
        look.float()
            .w(px(WIDTH_PX))
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::submit))
            .child(self.header(&phase, cx))
            .child(look.separator())
            .children(self.body(&phase, cx))
            .when(generating, |sheet| {
                sheet.child(look.separator()).child(self.generator(cx))
            })
            .child(look.separator())
            .child(self.footer(cx))
    }
}
