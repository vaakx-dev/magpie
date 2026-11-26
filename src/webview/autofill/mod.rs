use gpui::{Bounds, Pixels, point, px, size};
use serde::{Deserialize, Serialize};
use url::Url;
use webview2_com::{Microsoft::Web::WebView2::Win32::*, WebMessageReceivedEventHandler};
use windows::core::{HSTRING, Result};
use zeroize::Zeroizing;

use super::{
    link,
    native::report,
    script,
    signal::{Signal, Signals, address, text},
};

const FORM: &str = include_str!("form.js");
const DETECT: &str = include_str!("detect.js");
const FILL: &str = include_str!("fill.js");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Username,
    Password,
    NewPassword,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    SignIn,
    NewPassword,
}

#[derive(Serialize)]
pub struct Fill<'a> {
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub purpose: Purpose,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct Spot {
    role: Role,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Field {
    pub role: Role,
    pub bounds: Bounds<Pixels>,
}

#[derive(Clone)]
pub struct Submission {
    pub page: Url,
    pub username: String,
    pub password: Zeroizing<String>,
}

pub enum Sighting {
    Focused(Spot),
    Left,
    Found,
}

#[derive(Default)]
pub struct Login {
    spot: Option<Spot>,
    form: bool,
    filled: bool,
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum Message {
    #[serde(rename = "login_field")]
    Field(Spot),
    #[serde(rename = "login_field_left")]
    Left,
    #[serde(rename = "login_form")]
    Form,
    #[serde(rename = "login_submitted")]
    Submitted { username: String, password: String },
}

#[derive(Serialize)]
struct Request<'a> {
    origin: String,
    #[serde(flatten)]
    fill: &'a Fill<'a>,
}

impl Spot {
    fn field(self, zoom: f32) -> Field {
        Field {
            role: self.role,
            bounds: Bounds::new(
                point(px(self.x * zoom), px(self.y * zoom)),
                size(px(self.width * zoom), px(self.height * zoom)),
            ),
        }
    }
}

impl Login {
    pub fn field(&self, zoom: f32) -> Option<Field> {
        self.spot.map(|spot| spot.field(zoom))
    }

    pub fn form(&self) -> bool {
        self.form
    }

    pub fn focused(&self) -> bool {
        self.spot.is_some()
    }

    pub fn see(&mut self, sighting: Sighting) {
        match sighting {
            Sighting::Focused(_) if self.filled => {}
            Sighting::Focused(spot) => {
                self.spot = Some(spot);
                self.form = true;
            }
            Sighting::Left => {
                self.spot = None;
                self.filled = false;
            }
            Sighting::Found => self.form = true,
        }
    }

    pub fn filled(&mut self) {
        self.spot = None;
        self.filled = true;
    }
}

pub fn fillable(page: &Url) -> bool {
    link::web(page)
}

fn signal(message: Message, page: Url) -> Signal {
    match message {
        Message::Field(spot) => Signal::Login(Sighting::Focused(spot)),
        Message::Left => Signal::Login(Sighting::Left),
        Message::Form => Signal::Login(Sighting::Found),
        Message::Submitted { username, password } => Signal::Submitted(Submission {
            page,
            username,
            password: Zeroizing::new(password),
        }),
    }
}

fn receive(args: &ICoreWebView2WebMessageReceivedEventArgs) -> Result<Option<Signal>> {
    let Some(page) = address(|value| unsafe { args.Source(value) })?.filter(fillable) else {
        return Ok(None);
    };
    let json = Zeroizing::new(text(|value| unsafe { args.WebMessageAsJson(value) })?);
    let message = serde_json::from_str::<Message>(&json).ok();
    Ok(message.map(|message| signal(message, page)))
}

pub fn wire(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    script::install(webview, &format!("{{\n{FORM}\n{DETECT}\n}}"))?;
    let handler = WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
        let Some(args) = args else {
            return Ok(());
        };
        if let Some(signal) = receive(&args)? {
            signals.send(signal);
        }
        Ok(())
    }));
    unsafe { webview.add_WebMessageReceived(&handler, &mut 0) }
}

pub fn fill(webview: &ICoreWebView2, page: &Url, fill: &Fill) {
    if !fillable(page) {
        return;
    }
    let request = Request {
        origin: page.origin().ascii_serialization(),
        fill,
    };
    let json = Zeroizing::new(
        serde_json::to_string(&request).expect("an autofill request always serializes to JSON"),
    );
    let script = Zeroizing::new(format!("{{\n{FORM}\n({FILL})({})\n}}", json.as_str()));
    report(unsafe { webview.ExecuteScript(&HSTRING::from(script.as_str()), None) });
}
