use std::{mem, time::Duration};

use gpui::{
    ClipboardItem, Context, Div, ElementId, Entity, EventEmitter, Focusable, KeyBinding,
    MouseButton, SharedString, Stateful, Subscription, Task, Window, actions, div, prelude::*, px,
    rgba,
};
use url::Url;
use zeroize::Zeroizing;

use super::{
    clipboard,
    keyring::{Account, Entry, Keyring, Pending, Phase, Problem},
};
use crate::{
    locale::{Locale, Text},
    ui::{
        field::Field,
        icon::Icon,
        menu::heading,
        surface::{Colors, panel, tint},
    },
    webview::{Fill, WebView, fillable},
};
use magpie_engine::bitwarden::{
    CipherId, Error, Login, Method, Proof, Server, Totp, Vault, password,
};

actions!(passwords, [Submit, NextField, PreviousField]);

pub struct Done;

pub struct Look {
    pub colors: &'static Colors,
    pub locale: Locale,
}

pub struct Options {
    pub keyring: Entity<Keyring>,
    pub page: Option<Entity<WebView>>,
    pub look: Look,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Host {
    UnitedStates,
    Europe,
    Custom,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Busy,
    Unavailable,
    SignedOut,
    Challenged,
    Locked,
    Unlocked,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Username,
    Password,
    Code,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Emphasis {
    Primary,
    Secondary,
}

struct Fields {
    address: Entity<Field>,
    email: Entity<Field>,
    password: Entity<Field>,
    code: Entity<Field>,
    search: Entity<Field>,
}

struct Form {
    name: Entity<Field>,
    website: Entity<Field>,
    username: Entity<Field>,
    password: Entity<Field>,
    saving: bool,
}

pub struct Panel {
    keyring: Entity<Keyring>,
    page: Option<Entity<WebView>>,
    look: Look,
    host: Host,
    method: Option<Method>,
    stage: Stage,
    fields: Fields,
    form: Option<Form>,
    _keyring: Subscription,
    _search: Subscription,
    _clock: Task<()>,
}

const CONTEXT: &str = "Passwords";
const WIDTH_PX: f32 = 340.0;
const INPUT_HEIGHT_PX: f32 = 30.0;
const BUTTON_HEIGHT_PX: f32 = 28.0;
const CHIP_HEIGHT_PX: f32 = 22.0;
const SEGMENT_HEIGHT_PX: f32 = 26.0;
const PROBLEM_BACKGROUND: u32 = 0xd9304226;
const PRIMARY_HOVER_OPACITY: f32 = 0.9;
const SEGMENT_ALPHA: u8 = 0x2e;
const LIST_MAX_HEIGHT_PX: f32 = 240.0;
const RESULT_LIMIT: usize = 50;
const TICK: Duration = Duration::from_secs(1);
const HOSTS: [Host; 3] = [Host::UnitedStates, Host::Europe, Host::Custom];
const SERVER_PLACEHOLDER: &str = "https://vault.example.com";
const YUBIKEY: &str = "YubiKey";

pub fn bindings() -> Vec<KeyBinding> {
    let context = Some(CONTEXT);
    vec![
        KeyBinding::new("enter", Submit, context),
        KeyBinding::new("tab", NextField, context),
        KeyBinding::new("shift-tab", PreviousField, context),
    ]
}

fn stage(phase: &Phase) -> Stage {
    match phase {
        Phase::Busy => Stage::Busy,
        Phase::Unavailable(_) => Stage::Unavailable,
        Phase::SignedOut => Stage::SignedOut,
        Phase::Challenged(_) => Stage::Challenged,
        Phase::Locked(_) => Stage::Locked,
        Phase::Unlocked(_) => Stage::Unlocked,
    }
}

fn chosen(methods: &[Method], picked: Option<Method>) -> Option<Method> {
    picked
        .filter(|method| methods.contains(method))
        .or_else(|| methods.first().copied())
}

fn tab_stop(field: Entity<Field>, cx: &mut Context<Panel>) -> Entity<Field> {
    field.focus_handle(cx).tab_stop(true);
    field
}

fn filled_field(placeholder: &str, content: &str, cx: &mut Context<Panel>) -> Entity<Field> {
    let field = cx.new(|cx| Field::new(placeholder, content, cx));
    tab_stop(field, cx)
}

fn plain_field(placeholder: &str, cx: &mut Context<Panel>) -> Entity<Field> {
    filled_field(placeholder, "", cx)
}

fn secret_field(placeholder: &str, cx: &mut Context<Panel>) -> Entity<Field> {
    let field = cx.new(|cx| Field::secret(placeholder, cx));
    tab_stop(field, cx)
}

fn clock(cx: &mut Context<Panel>) -> Task<()> {
    cx.spawn(async move |panel, cx| {
        loop {
            cx.background_executor().timer(TICK).await;
            if panel.update(cx, |_, cx| cx.notify()).is_err() {
                break;
            }
        }
    })
}

fn grouped(digits: &str) -> String {
    let (left, right) = digits.split_at(digits.len() / 2);
    format!("{left} {right}")
}

fn code(login: &Login) -> Option<Zeroizing<String>> {
    let totp = Totp::parse(login.totp.as_deref()?).ok()?;
    Some(Zeroizing::new(totp.code().digits))
}

fn part(login: &Login, part: Part) -> Option<Zeroizing<String>> {
    match part {
        Part::Username => login.username.clone().map(Zeroizing::new),
        Part::Password => login.password.clone().map(Zeroizing::new),
        Part::Code => code(login),
    }
}

fn login<'a>(vault: &'a Vault, id: &CipherId) -> Option<&'a Login> {
    vault.logins().iter().find(|login| &login.id == id)
}

fn input(field: &Entity<Field>, colors: &Colors, cx: &Context<Panel>) -> Div {
    let focus = field.focus_handle(cx);
    div()
        .h(px(INPUT_HEIGHT_PX))
        .px_2()
        .flex()
        .items_center()
        .rounded_md()
        .bg(rgba(colors.raised))
        .border_1()
        .border_color(rgba(colors.line))
        .cursor_text()
        .on_mouse_down(MouseButton::Left, move |_, window, _| focus.focus(window))
        .child(field.clone())
}

fn note(text: impl Into<SharedString>, colors: &Colors) -> Div {
    div()
        .px_2()
        .py_1()
        .text_xs()
        .text_color(rgba(colors.muted))
        .child(text.into())
}

impl Look {
    fn text(&self, text: Text) -> &'static str {
        self.locale.text(text)
    }

    fn button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        emphasis: Emphasis,
    ) -> Stateful<Div> {
        let colors = self.colors;
        let hover = colors.overlay;
        let button = div()
            .id(id)
            .h(px(BUTTON_HEIGHT_PX))
            .px_3()
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .child(label.into());
        match emphasis {
            Emphasis::Primary => button
                .bg(rgba(colors.text))
                .text_color(rgba(colors.on_text))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .hover(|button| button.opacity(PRIMARY_HOVER_OPACITY)),
            Emphasis::Secondary => button
                .bg(rgba(colors.raised))
                .hover(move |button| button.bg(rgba(hover))),
        }
    }

    fn chip(&self, id: &'static str, icon: Icon, label: impl Into<SharedString>) -> Stateful<Div> {
        let colors = self.colors;
        let hover = colors.overlay;
        div()
            .id(id)
            .h(px(CHIP_HEIGHT_PX))
            .px_1p5()
            .flex()
            .items_center()
            .gap_1()
            .rounded_md()
            .text_xs()
            .bg(rgba(colors.raised))
            .cursor_pointer()
            .hover(move |chip| chip.bg(rgba(hover)))
            .child(icon.tinted(rgba(colors.muted)))
            .child(label.into())
    }

    fn segment(
        &self,
        id: impl Into<ElementId>,
        label: &'static str,
        emphasis: Emphasis,
    ) -> Stateful<Div> {
        let hover = self.colors.overlay;
        let segment = div()
            .id(id)
            .flex_1()
            .h(px(SEGMENT_HEIGHT_PX))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .text_xs()
            .cursor_pointer()
            .child(label);
        match emphasis {
            Emphasis::Primary => segment
                .bg(tint(self.colors.text, SEGMENT_ALPHA))
                .font_weight(gpui::FontWeight::SEMIBOLD),
            Emphasis::Secondary => segment.hover(move |segment| segment.bg(rgba(hover))),
        }
    }

    fn host(&self, host: Host) -> &'static str {
        match host {
            Host::UnitedStates => "Bitwarden US",
            Host::Europe => "Bitwarden EU",
            Host::Custom => self.text(Text::SelfHosted),
        }
    }

    fn method(&self, method: Method) -> &'static str {
        match method {
            Method::Authenticator => self.text(Text::Authenticator),
            Method::Email | Method::NewDevice => self.text(Text::Email),
            Method::YubiKey => YUBIKEY,
        }
    }

    fn problem(&self, problem: Option<&Problem>) -> Option<Div> {
        let message = match problem? {
            Problem::Failed(message) => SharedString::from(message.clone()),
            Problem::WrongCode => self.text(Text::WrongCode).into(),
        };
        Some(
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .text_xs()
                .bg(rgba(PROBLEM_BACKGROUND))
                .child(message),
        )
    }
}

impl EventEmitter<Done> for Panel {}

impl Panel {
    pub fn new(options: Options, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let look = options.look;
        let fields = Fields {
            address: plain_field(SERVER_PLACEHOLDER, cx),
            email: plain_field(look.text(Text::Email), cx),
            password: secret_field(look.text(Text::MasterPassword), cx),
            code: plain_field(look.text(Text::VerificationCode), cx),
            search: plain_field(look.text(Text::SearchVault), cx),
        };
        let keyring = cx.observe_in(&options.keyring, window, |panel, _, window, cx| {
            panel.follow(window, cx)
        });
        let search = cx.observe(&fields.search, |_, _, cx| cx.notify());
        let stage = stage(&options.keyring.read(cx).phase());
        let panel = Self {
            keyring: options.keyring,
            page: options.page,
            look,
            host: Host::UnitedStates,
            method: None,
            stage,
            fields,
            form: None,
            _keyring: keyring,
            _search: search,
            _clock: clock(cx),
        };
        panel.focus_first(window, cx);
        panel
    }

    fn follow(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let stage = stage(&self.keyring.read(cx).phase());
        let changed = mem::replace(&mut self.stage, stage) != stage;
        self.settle_form(cx);
        if changed {
            self.focus_first(window, cx);
        }
        cx.notify();
    }

    fn settle_form(&mut self, cx: &Context<Self>) {
        let failed = self.keyring.read(cx).problem().is_some();
        let Some(form) = &mut self.form else {
            return;
        };
        match self.stage {
            Stage::Busy => {}
            Stage::Unlocked if !form.saving => {}
            Stage::Unlocked if failed => form.saving = false,
            _ => self.form = None,
        }
    }

    fn first_field(&self) -> Option<&Entity<Field>> {
        match (self.stage, &self.form) {
            (Stage::SignedOut, _) => Some(&self.fields.email),
            (Stage::Challenged, _) => Some(&self.fields.code),
            (Stage::Locked, _) => Some(&self.fields.password),
            (Stage::Unlocked, Some(form)) => Some(&form.username),
            (Stage::Unlocked, None) => Some(&self.fields.search),
            (Stage::Busy | Stage::Unavailable, _) => None,
        }
    }

    fn focus_first(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(field) = self.first_field() {
            field.focus_handle(cx).focus(window);
        }
    }

    fn fillable_page(&self, cx: &Context<Self>) -> Option<Url> {
        let url = self.page.as_ref()?.read(cx).url()?;
        fillable(url).then(|| url.clone())
    }

    fn server(&self, cx: &Context<Self>) -> Result<Server, Error> {
        match self.host {
            Host::UnitedStates => Ok(Server::UnitedStates),
            Host::Europe => Ok(Server::Europe),
            Host::Custom => Server::parse(self.fields.address.read(cx).text()),
        }
    }

    fn sign_in(&mut self, cx: &mut Context<Self>) {
        let email = self.fields.email.read(cx).text().trim().to_owned();
        if email.is_empty() || self.fields.password.read(cx).text().is_empty() {
            return;
        }
        let server = self.server(cx);
        let password = self.fields.password.update(cx, Field::take);
        self.keyring.update(cx, |keyring, cx| match server {
            Ok(server) => keyring.sign_in(
                Account {
                    server,
                    email,
                    password,
                },
                cx,
            ),
            Err(error) => keyring.report(&error, cx),
        });
    }

    fn chosen_method(&self, cx: &Context<Self>) -> Option<Method> {
        match self.keyring.read(cx).phase() {
            Phase::Challenged(pending) => chosen(pending.methods(), self.method),
            _ => None,
        }
    }

    fn answer(&mut self, cx: &mut Context<Self>) {
        let Some(method) = self.chosen_method(cx) else {
            return;
        };
        if self.fields.code.read(cx).text().trim().is_empty() {
            return;
        }
        let code = self.fields.code.update(cx, Field::take);
        let proof = Proof { method, code };
        self.keyring
            .update(cx, |keyring, cx| keyring.answer(proof, cx));
    }

    fn unlock(&mut self, cx: &mut Context<Self>) {
        if self.fields.password.read(cx).text().is_empty() {
            return;
        }
        let password = self.fields.password.update(cx, Field::take);
        self.keyring
            .update(cx, |keyring, cx| keyring.unlock(password, cx));
    }

    fn open_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (title, address) = match (self.fillable_page(cx), &self.page) {
            (Some(url), Some(view)) => (view.read(cx).title(), url.to_string()),
            _ => (SharedString::default(), String::new()),
        };
        let form = Form {
            name: filled_field(self.look.text(Text::Name), &title, cx),
            website: filled_field(self.look.text(Text::Website), &address, cx),
            username: plain_field(self.look.text(Text::Username), cx),
            password: secret_field(self.look.text(Text::Password), cx),
            saving: false,
        };
        form.username.focus_handle(cx).focus(window);
        self.form = Some(form);
        cx.notify();
    }

    fn close_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.form = None;
        self.focus_first(window, cx);
        cx.notify();
    }

    fn generate(&mut self, cx: &mut Context<Self>) {
        let Some(form) = &self.form else {
            return;
        };
        match password::generate() {
            Ok(generated) => form
                .password
                .update(cx, |field, cx| field.set(&generated, cx)),
            Err(error) => self
                .keyring
                .update(cx, |keyring, cx| keyring.report(&error, cx)),
        }
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        let Some(form) = &mut self.form else {
            return;
        };
        let text = |field: &Entity<Field>| field.read(cx).text().to_owned();
        let entry = Entry {
            name: text(&form.name),
            username: text(&form.username),
            password: Zeroizing::new(text(&form.password)),
            uri: text(&form.website),
        };
        if entry.name.trim().is_empty() {
            return;
        }
        form.saving = true;
        self.keyring
            .update(cx, |keyring, cx| keyring.create_login(entry, cx));
    }

    fn submit(&mut self, _: &Submit, _: &mut Window, cx: &mut Context<Self>) {
        match (self.stage, &self.form) {
            (Stage::SignedOut, _) => self.sign_in(cx),
            (Stage::Challenged, _) => self.answer(cx),
            (Stage::Locked, _) => self.unlock(cx),
            (Stage::Unlocked, Some(_)) => self.save(cx),
            (Stage::Unlocked, None) | (Stage::Busy | Stage::Unavailable, _) => {}
        }
    }

    fn next_field(&mut self, _: &NextField, window: &mut Window, _: &mut Context<Self>) {
        window.focus_next();
    }

    fn previous_field(&mut self, _: &PreviousField, window: &mut Window, _: &mut Context<Self>) {
        window.focus_prev();
    }

    fn secret(&self, id: &CipherId, wanted: Part, cx: &Context<Self>) -> Option<Zeroizing<String>> {
        let Phase::Unlocked(vault) = self.keyring.read(cx).phase() else {
            return None;
        };
        part(login(vault, id)?, wanted)
    }

    fn copy(&mut self, id: &CipherId, wanted: Part, cx: &mut Context<Self>) {
        let Some(value) = self.secret(id, wanted, cx) else {
            return;
        };
        match wanted {
            Part::Username => cx.write_to_clipboard(ClipboardItem::new_string(value.to_string())),
            Part::Password | Part::Code => clipboard::copy_secret(value, cx),
        }
        cx.emit(Done);
    }

    fn fill(&mut self, id: &CipherId, cx: &mut Context<Self>) {
        let Some(view) = self.page.clone() else {
            return;
        };
        let username = self.secret(id, Part::Username, cx);
        let password = self.secret(id, Part::Password, cx);
        let fill = Fill {
            username: username.as_deref().map(String::as_str),
            password: password.as_deref().map(String::as_str),
        };
        view.read(cx).autofill(&fill);
        cx.emit(Done);
    }

    fn title(&self, text: Text) -> Div {
        div()
            .px_2()
            .pt_1()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .child(self.look.text(text))
    }

    fn hosts(&self, cx: &Context<Self>) -> Div {
        let segments = HOSTS.iter().map(|host| {
            let host = *host;
            let emphasis = match host == self.host {
                true => Emphasis::Primary,
                false => Emphasis::Secondary,
            };
            self.look
                .segment(("host", host as usize), self.look.host(host), emphasis)
                .on_click(cx.listener(move |panel, _, _, cx| {
                    panel.host = host;
                    cx.notify();
                }))
        });
        div().flex().gap_1().children(segments)
    }

    fn signed_out(&self, cx: &Context<Self>) -> Div {
        let colors = self.look.colors;
        let custom = self.host == Host::Custom;
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.title(Text::SignInToBitwarden))
            .child(self.hosts(cx))
            .when(custom, |form| {
                form.child(input(&self.fields.address, colors, cx))
            })
            .child(input(&self.fields.email, colors, cx))
            .child(input(&self.fields.password, colors, cx))
            .children(self.look.problem(self.keyring.read(cx).problem()))
            .child(
                self.look
                    .button("sign-in", self.look.text(Text::SignIn), Emphasis::Primary)
                    .on_click(cx.listener(|panel, _, _, cx| panel.sign_in(cx))),
            )
    }

    fn methods(&self, methods: &[Method], chosen: Method, cx: &Context<Self>) -> Option<Div> {
        if methods.len() < 2 {
            return None;
        }
        let segments = methods.iter().enumerate().map(|(index, method)| {
            let method = *method;
            let emphasis = match method == chosen {
                true => Emphasis::Primary,
                false => Emphasis::Secondary,
            };
            self.look
                .segment(("method", index), self.look.method(method), emphasis)
                .on_click(cx.listener(move |panel, _, _, cx| {
                    panel.method = Some(method);
                    cx.notify();
                }))
        });
        Some(div().flex().gap_1().children(segments))
    }

    fn email_button(&self, pending: &Pending, cx: &Context<Self>) -> Stateful<Div> {
        let label = match pending.email_sent() {
            true => Text::EmailCodeSent,
            false => Text::SendEmailCode,
        };
        self.look
            .button(
                "send-email-code",
                self.look.text(label),
                Emphasis::Secondary,
            )
            .on_click(cx.listener(|panel, _, _, cx| {
                panel
                    .keyring
                    .update(cx, |keyring, cx| keyring.send_email_code(cx))
            }))
    }

    fn challenged(&self, pending: &Pending, cx: &Context<Self>) -> Div {
        let colors = self.look.colors;
        let methods = pending.methods();
        let Some(method) = chosen(methods, self.method) else {
            return div();
        };
        let actions = div()
            .flex()
            .gap_2()
            .justify_end()
            .child(
                self.look
                    .button(
                        "cancel-sign-in",
                        self.look.text(Text::Cancel),
                        Emphasis::Secondary,
                    )
                    .on_click(cx.listener(|panel, _, _, cx| {
                        panel
                            .keyring
                            .update(cx, |keyring, cx| keyring.cancel_sign_in(cx))
                    })),
            )
            .child(
                self.look
                    .button("verify", self.look.text(Text::Continue), Emphasis::Primary)
                    .on_click(cx.listener(|panel, _, _, cx| panel.answer(cx))),
            );
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.title(Text::TwoStepLogin))
            .children(self.methods(methods, method, cx))
            .child(input(&self.fields.code, colors, cx))
            .when(method == Method::Email, |form| {
                form.child(self.email_button(pending, cx))
            })
            .children(self.look.problem(self.keyring.read(cx).problem()))
            .child(actions)
    }

    fn locked(&self, vault: &Vault, cx: &Context<Self>) -> Div {
        let colors = self.look.colors;
        let actions = div()
            .flex()
            .gap_2()
            .justify_end()
            .child(
                self.look
                    .button(
                        "sign-out",
                        self.look.text(Text::SignOut),
                        Emphasis::Secondary,
                    )
                    .on_click(cx.listener(|panel, _, _, cx| {
                        panel.keyring.update(cx, |keyring, cx| keyring.sign_out(cx))
                    })),
            )
            .child(
                self.look
                    .button("unlock", self.look.text(Text::Unlock), Emphasis::Primary)
                    .on_click(cx.listener(|panel, _, _, cx| panel.unlock(cx))),
            );
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.title(Text::VaultLocked))
            .children(vault.email().map(|email| note(email.to_owned(), colors)))
            .child(input(&self.fields.password, colors, cx))
            .children(self.look.problem(self.keyring.read(cx).problem()))
            .child(actions)
    }

    fn row(&self, index: usize, login: &Login, cx: &Context<Self>) -> Stateful<Div> {
        let colors = self.look.colors;
        let id = login.id.clone();
        let listen = |wanted: Part| {
            let id = id.clone();
            cx.listener(move |panel, _, _, cx| panel.copy(&id, wanted, cx))
        };
        let fill = self.fillable_page(cx).map(|_| {
            let filled = id.clone();
            self.look
                .chip("fill", Icon::Check, self.look.text(Text::Fill))
                .on_click(cx.listener(move |panel, _, _, cx| panel.fill(&filled, cx)))
        });
        let username = login.username.as_ref().map(|_| {
            self.look
                .chip("username", Icon::Copy, self.look.text(Text::Username))
                .on_click(listen(Part::Username))
        });
        let password = login.password.as_ref().map(|_| {
            self.look
                .chip("password", Icon::Copy, self.look.text(Text::Password))
                .on_click(listen(Part::Password))
        });
        let totp = login
            .totp
            .as_deref()
            .and_then(|secret| Totp::parse(secret).ok())
            .map(|totp| {
                let code = totp.code();
                let label = format!("{} · {}s", grouped(&code.digits), code.remaining_secs);
                self.look
                    .chip("code", Icon::Lock, label)
                    .on_click(listen(Part::Code))
            });
        div()
            .id(index)
            .px_2()
            .py_1p5()
            .flex()
            .flex_col()
            .gap_1()
            .rounded_md()
            .child(div().truncate().child(login.name.clone()))
            .children(login.username.as_ref().map(|username| {
                div()
                    .truncate()
                    .text_xs()
                    .text_color(rgba(colors.muted))
                    .child(username.clone())
            }))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1()
                    .children(fill)
                    .children(username)
                    .children(password)
                    .children(totp),
            )
    }

    fn list(
        &self,
        id: &'static str,
        logins: &[&Login],
        empty: Text,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let colors = self.look.colors;
        let rows = logins
            .iter()
            .take(RESULT_LIMIT)
            .enumerate()
            .map(|(index, login)| self.row(index, login, cx));
        div()
            .id(id)
            .max_h(px(LIST_MAX_HEIGHT_PX))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .children(rows)
            .when(logins.is_empty(), |list| {
                list.child(note(self.look.text(empty), colors))
            })
    }

    fn header(&self, vault: &Vault, cx: &Context<Self>) -> Div {
        let colors = self.look.colors;
        div()
            .flex()
            .items_center()
            .gap_1()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .px_2()
                    .truncate()
                    .text_xs()
                    .text_color(rgba(colors.muted))
                    .child(vault.email().unwrap_or_default().to_owned()),
            )
            .child(
                self.look
                    .chip("add", Icon::Plus, self.look.text(Text::Add))
                    .on_click(cx.listener(|panel, _, window, cx| panel.open_form(window, cx))),
            )
            .child(
                self.look
                    .chip("sync", Icon::RotateCw, self.look.text(Text::Sync))
                    .on_click(cx.listener(|panel, _, _, cx| {
                        panel.keyring.update(cx, |keyring, cx| keyring.sync(cx))
                    })),
            )
            .child(
                self.look
                    .chip("lock", Icon::Lock, self.look.text(Text::Lock))
                    .on_click(cx.listener(|panel, _, _, cx| {
                        panel.keyring.update(cx, |keyring, cx| keyring.lock(cx))
                    })),
            )
    }

    fn form(&self, form: &Form, cx: &Context<Self>) -> Div {
        let colors = self.look.colors;
        let password = div()
            .flex()
            .gap_1()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(input(&form.password, colors, cx)),
            )
            .child(
                self.look
                    .button(
                        "generate",
                        self.look.text(Text::Generate),
                        Emphasis::Secondary,
                    )
                    .on_click(cx.listener(|panel, _, _, cx| panel.generate(cx))),
            );
        let actions = div()
            .flex()
            .gap_2()
            .justify_end()
            .child(
                self.look
                    .button(
                        "cancel-add",
                        self.look.text(Text::Cancel),
                        Emphasis::Secondary,
                    )
                    .on_click(cx.listener(|panel, _, window, cx| panel.close_form(window, cx))),
            )
            .child(
                self.look
                    .button("save", self.look.text(Text::Save), Emphasis::Primary)
                    .on_click(cx.listener(|panel, _, _, cx| panel.save(cx))),
            );
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.title(Text::NewLogin))
            .child(input(&form.name, colors, cx))
            .child(input(&form.website, colors, cx))
            .child(input(&form.username, colors, cx))
            .child(password)
            .children(self.look.problem(self.keyring.read(cx).problem()))
            .child(actions)
    }

    fn unlocked(&self, vault: &Vault, cx: &Context<Self>) -> Div {
        match &self.form {
            Some(form) => self.form(form, cx),
            None => self.browse(vault, cx),
        }
    }

    fn browse(&self, vault: &Vault, cx: &Context<Self>) -> Div {
        let colors = self.look.colors;
        let matches = self.fillable_page(cx).map(|page| {
            let found = vault.matches_for_url(page.as_str());
            div()
                .child(heading(self.look.text(Text::ThisSite), colors))
                .child(self.list("matches", &found, Text::NoLoginsForSite, cx))
        });
        let query = self.fields.search.read(cx).text().trim().to_owned();
        let results = (!query.is_empty()).then(|| {
            let found = vault.search(&query);
            self.list("results", &found, Text::NoMatchingLogins, cx)
        });
        let search = div()
            .flex()
            .items_center()
            .gap_2()
            .child(Icon::Search.tinted(rgba(colors.muted)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(input(&self.fields.search, colors, cx)),
            );
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.header(vault, cx))
            .children(self.look.problem(self.keyring.read(cx).problem()))
            .children(matches)
            .child(search)
            .children(results)
    }

    fn body(&self, cx: &Context<Self>) -> Div {
        let colors = self.look.colors;
        match self.keyring.read(cx).phase() {
            Phase::Busy => note(self.look.text(Text::Working), colors),
            Phase::Unavailable(message) => note(message.to_owned(), colors),
            Phase::SignedOut => self.signed_out(cx),
            Phase::Challenged(pending) => self.challenged(pending, cx),
            Phase::Locked(vault) => self.locked(vault, cx),
            Phase::Unlocked(vault) => self.unlocked(vault, cx),
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        panel(self.look.colors)
            .w(px(WIDTH_PX))
            .p_2()
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::next_field))
            .on_action(cx.listener(Self::previous_field))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .pb_1()
                    .child(Icon::KeyRound.tinted(rgba(self.look.colors.muted)))
                    .child(self.look.text(Text::Passwords)),
            )
            .child(self.body(cx))
    }
}
