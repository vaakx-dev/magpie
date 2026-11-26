use std::process::Command;

use gpui::{
    ClickEvent, Context, Div, ElementId, FontWeight, MouseButton, SharedString, Stateful, Task,
    Window, div, img, prelude::*, px, rgba,
};
use magpie_engine::focus::{Focus, Rule};

use super::{
    browser::Browser,
    command::{Mode, Target},
    favicon,
};
use crate::{
    config::{Choosable, Engine, Scheme, Settings, Side, Switch, Theme},
    import::{Profile, Scope, detect},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Welcome,
    Look,
    Search,
    Bring,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Progress {
    Ready,
    Importing,
    Imported,
}

struct Source {
    profile: Profile,
    progress: Progress,
}

pub(super) struct Tour {
    step: Step,
    sources: Option<Vec<Source>>,
    _detection: Task<()>,
}

struct Tones {
    backdrop: u32,
    card: u32,
    control: u32,
    text: u32,
    muted: u32,
}

const DARK: Tones = Tones {
    backdrop: 0x14141499,
    card: 0x1f1f1fff,
    control: 0x2a2a2aff,
    text: 0xf2f2f2ff,
    muted: 0xf2f2f299,
};

const LIGHT: Tones = Tones {
    backdrop: 0xf3f3f399,
    card: 0xffffffff,
    control: 0xeaeaeaff,
    text: 0x141414ff,
    muted: 0x14141499,
};

const STEPS: [Step; 4] = [Step::Welcome, Step::Look, Step::Search, Step::Bring];
const THEMES: [Theme; 3] = [Theme::System, Theme::Dark, Theme::Light];
const SIDES: [Side; 2] = [Side::Left, Side::Right];
const SWITCHES: [Switch; 2] = [Switch::Off, Switch::On];
const DEFAULT_APPS_SETTINGS: &str = "ms-settings:defaultapps";

const CARD_WIDTH_PX: f32 = 440.0;
const CARD_PADDING_PX: f32 = 24.0;
const CARD_CORNER_PX: f32 = 12.0;
const CONTROL_CORNER_PX: f32 = 8.0;
const SEGMENT_CORNER_PX: f32 = 6.0;
const CONTROL_HEIGHT_PX: f32 = 32.0;
const SEGMENT_INSET_PX: f32 = 2.0;
const ROW_HEIGHT_PX: f32 = 36.0;
const DOT_PX: f32 = 6.0;
const CURRENT_DOT_WIDTH_PX: f32 = 16.0;
const LOGO_PX: f32 = 56.0;
const HOVER_OPACITY: f32 = 0.85;

fn tones(scheme: Scheme) -> &'static Tones {
    match scheme {
        Scheme::Dark => &DARK,
        Scheme::Light => &LIGHT,
    }
}

fn built_in_engines() -> impl Iterator<Item = Engine> {
    Engine::ALL
        .iter()
        .copied()
        .filter(|engine| *engine != Engine::Custom)
}

fn distractions() -> impl Iterator<Item = Rule> {
    Rule::ALL
        .into_iter()
        .filter(|rule| rule.blocked_by_default())
}

fn distractions_blocked(focus: &Focus) -> Switch {
    Switch::from(distractions().all(|rule| focus.blocks(rule)))
}

fn block_distractions(focus: &mut Focus, switch: Switch) {
    for rule in distractions() {
        match switch {
            Switch::On => focus.block(rule),
            Switch::Off => focus.allow(rule),
        }
    }
}

fn open_default_apps_settings() {
    if let Err(error) = Command::new("explorer").arg(DEFAULT_APPS_SETTINGS).spawn() {
        eprintln!("magpie: could not open the default apps settings: {error}");
    }
}

fn sources(profiles: Vec<Profile>) -> Vec<Source> {
    profiles
        .into_iter()
        .map(|profile| Source {
            profile,
            progress: Progress::Ready,
        })
        .collect()
}

fn progress_label(progress: Progress) -> &'static str {
    match progress {
        Progress::Ready => "Import",
        Progress::Importing => "Importing…",
        Progress::Imported => "Imported",
    }
}

fn detection(cx: &mut Context<Browser>) -> Task<()> {
    let profiles = cx.background_spawn(async { detect() });
    cx.spawn(async move |browser, cx| {
        let found = sources(profiles.await);
        browser
            .update(cx, |browser, cx| browser.found_sources(found, cx))
            .ok();
    })
}

impl Step {
    fn index(self) -> usize {
        STEPS
            .iter()
            .position(|step| *step == self)
            .expect("every step is in STEPS")
    }

    fn next(self) -> Option<Self> {
        STEPS.get(self.index() + 1).copied()
    }
}

impl Tones {
    fn button(&self, id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
        div()
            .id(id)
            .h(px(CONTROL_HEIGHT_PX))
            .px_4()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(px(CONTROL_CORNER_PX))
            .font_weight(FontWeight::SEMIBOLD)
            .cursor_pointer()
            .hover(|button| button.opacity(HOVER_OPACITY))
            .child(label.into())
    }

    fn primary(&self, id: &'static str, label: &'static str) -> Stateful<Div> {
        self.button(id, label)
            .bg(rgba(self.text))
            .text_color(rgba(self.card))
    }

    fn secondary(&self, id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
        self.button(id, label)
            .bg(rgba(self.control))
            .text_color(rgba(self.text))
    }

    fn quiet(&self, id: &'static str, label: &'static str) -> Stateful<Div> {
        div()
            .id(id)
            .px_2()
            .py_1()
            .rounded(px(CONTROL_CORNER_PX))
            .text_color(rgba(self.muted))
            .cursor_pointer()
            .hover(|link| link.text_color(rgba(self.text)))
            .child(label)
    }

    fn dots(&self, current: Step) -> Div {
        div()
            .flex()
            .items_center()
            .gap_1()
            .children(STEPS.iter().map(|step| {
                let reached = *step == current;
                div()
                    .h(px(DOT_PX))
                    .w(px(if reached {
                        CURRENT_DOT_WIDTH_PX
                    } else {
                        DOT_PX
                    }))
                    .rounded_full()
                    .bg(rgba(if reached { self.text } else { self.control }))
            }))
    }

    fn heading(&self, title: &'static str) -> Div {
        div()
            .text_lg()
            .font_weight(FontWeight::SEMIBOLD)
            .child(title)
    }

    fn label(&self, text: impl Into<SharedString>) -> Div {
        div()
            .text_xs()
            .text_color(rgba(self.muted))
            .child(text.into())
    }

    fn field(&self, label: &'static str, control: impl IntoElement) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_1p5()
            .child(self.label(label))
            .child(control)
    }

    fn row(&self, label: impl Into<SharedString>, control: impl IntoElement) -> Div {
        div()
            .h(px(ROW_HEIGHT_PX))
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .child(div().min_w_0().truncate().child(label.into()))
            .child(control)
    }
}

impl Browser {
    pub(super) fn offer_onboarding(&mut self, cx: &mut Context<Self>) {
        if self.settings().onboarding_done.on() {
            return;
        }
        self.onboarding = Some(Tour {
            step: Step::Welcome,
            sources: None,
            _detection: detection(cx),
        });
        cx.notify();
    }

    fn found_sources(&mut self, found: Vec<Source>, cx: &mut Context<Self>) {
        if let Some(tour) = self.onboarding.as_mut() {
            tour.sources = Some(found);
            cx.notify();
        }
    }

    fn advance(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tour) = self.onboarding.as_mut() else {
            return;
        };
        match tour.step.next() {
            Some(next) => tour.step = next,
            None => return self.finish_onboarding(window, cx),
        }
        cx.notify();
    }

    fn finish_onboarding(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.onboarding = None;
        self.change_settings(|settings| settings.onboarding_done = Switch::On, cx);
        self.show_palette(Mode::Everything(Target::NewTab), String::new(), window, cx);
    }

    fn source_mut(&mut self, index: usize) -> Option<&mut Source> {
        self.onboarding.as_mut()?.sources.as_mut()?.get_mut(index)
    }

    fn import_source(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(source) = self
            .source_mut(index)
            .filter(|source| source.progress == Progress::Ready)
        else {
            return;
        };
        source.progress = Progress::Importing;
        let profile = source.profile.clone();
        let reading = cx.background_spawn(async move {
            let imported = profile.read(Scope::Everything);
            (profile.label(), imported)
        });
        cx.spawn(async move |browser, cx| {
            let (label, imported) = reading.await;
            browser
                .update(cx, |browser, cx| {
                    browser.merge_import(&label, imported, cx);
                    browser.mark_imported(index, cx);
                })
                .ok();
        })
        .detach();
        cx.notify();
    }

    fn mark_imported(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(source) = self.source_mut(index) {
            source.progress = Progress::Imported;
            cx.notify();
        }
    }

    fn tones(&self) -> &'static Tones {
        tones(self.scheme())
    }

    fn on_tap(
        &self,
        element: Stateful<Div>,
        action: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        element.on_click(
            cx.listener(move |browser, _: &ClickEvent, window, cx| action(browser, window, cx)),
        )
    }

    fn segments<T: Choosable>(
        &self,
        id: &'static str,
        choices: &'static [T],
        chosen: T,
        choose: fn(&mut Settings, T),
        cx: &mut Context<Self>,
    ) -> Div {
        let tones = self.tones();
        let segments = choices.iter().enumerate().map(|(index, &choice)| {
            let selected = choice == chosen;
            let segment = div()
                .id((id, index))
                .flex_1()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(SEGMENT_CORNER_PX))
                .cursor_pointer()
                .text_color(rgba(if selected { tones.text } else { tones.muted }))
                .when(selected, |segment| {
                    segment
                        .bg(rgba(tones.card))
                        .font_weight(FontWeight::SEMIBOLD)
                })
                .child(SharedString::from(choice.label()));
            self.on_tap(
                segment,
                move |browser, _, cx| {
                    browser.change_settings(|settings| choose(settings, choice), cx)
                },
                cx,
            )
        });
        div()
            .h(px(CONTROL_HEIGHT_PX))
            .p(px(SEGMENT_INSET_PX))
            .flex()
            .gap(px(SEGMENT_INSET_PX))
            .rounded(px(CONTROL_CORNER_PX))
            .bg(rgba(tones.control))
            .children(segments)
    }

    fn protection(
        &self,
        id: &'static str,
        label: &'static str,
        state: Switch,
        set: fn(&mut Settings, Switch),
        cx: &mut Context<Self>,
    ) -> Div {
        let toggle = self.toggle(id, state, cx, move |browser, _, cx| {
            browser.change_settings(|settings| set(settings, state.toggled()), cx)
        });
        self.tones().row(label, toggle)
    }

    fn tour_header(&self, step: Step, cx: &mut Context<Self>) -> Div {
        let tones = self.tones();
        div()
            .flex()
            .items_center()
            .justify_between()
            .child(tones.dots(step))
            .child(self.on_tap(
                tones.quiet("onboarding-skip", "Skip"),
                Self::finish_onboarding,
                cx,
            ))
    }

    fn footer(&self, label: &'static str, cx: &mut Context<Self>) -> Div {
        let next = self.on_tap(
            self.tones().primary("onboarding-next", label),
            Self::advance,
            cx,
        );
        div().flex().justify_end().child(next)
    }

    fn welcome_step(&self, cx: &mut Context<Self>) -> Div {
        let tones = self.tones();
        div()
            .py_4()
            .flex()
            .flex_col()
            .items_center()
            .gap_3()
            .child(img(favicon::magpie()).size(px(LOGO_PX)))
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Welcome to Magpie"),
            )
            .child(
                div()
                    .text_color(rgba(tones.muted))
                    .child("Fast, private, distraction-free browsing."),
            )
            .child(div().pt_3().child(self.on_tap(
                tones.primary("onboarding-start", "Get started"),
                Self::advance,
                cx,
            )))
    }

    fn look_step(&self, cx: &mut Context<Self>) -> Div {
        let tones = self.tones();
        let settings = self.settings();
        let layout = settings.layout;
        let theme = self.segments(
            "onboarding-theme",
            &THEMES,
            settings.theme,
            |settings, theme| settings.theme = theme,
            cx,
        );
        let side = self.segments(
            "onboarding-sidebar-side",
            &SIDES,
            layout.sidebar_side,
            |settings, side| settings.layout.sidebar_side = side,
            cx,
        );
        let top_bar = self.segments(
            "onboarding-top-bar",
            &SWITCHES,
            layout.top_bar,
            |settings, switch| settings.layout.top_bar = switch,
            cx,
        );
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(tones.heading("Look & layout"))
            .child(tones.field("Theme", theme))
            .child(tones.field("Sidebar side", side))
            .child(tones.field("Top bar", top_bar))
            .child(self.footer("Next", cx))
    }

    fn engine_picker(&self, cx: &mut Context<Self>) -> Div {
        let tones = self.tones();
        let chosen = self.settings().general.search.engine;
        let engines = built_in_engines().enumerate().map(|(index, engine)| {
            let selected = engine == chosen;
            let chip = tones
                .secondary(("onboarding-engine", index), engine.label())
                .px_3()
                .when(!selected, |chip| {
                    chip.bg(rgba(tones.card)).text_color(rgba(tones.muted))
                });
            self.on_tap(
                chip,
                move |browser, _, cx| {
                    browser.change_settings(|settings| settings.general.search.engine = engine, cx)
                },
                cx,
            )
        });
        div().flex().flex_wrap().gap_1p5().children(engines)
    }

    fn search_step(&self, cx: &mut Context<Self>) -> Div {
        let tones = self.tones();
        let settings = self.settings();
        let protections = div()
            .flex()
            .flex_col()
            .child(self.protection(
                "onboarding-shield",
                "Ad & tracker blocker",
                settings.privacy.shield,
                |settings, switch| settings.privacy.shield = switch,
                cx,
            ))
            .child(self.protection(
                "onboarding-distractions",
                "YouTube Shorts & distraction blocker",
                distractions_blocked(&settings.focus),
                |settings, switch| block_distractions(&mut settings.focus, switch),
                cx,
            ))
            .child(self.protection(
                "onboarding-sponsorblock",
                "SponsorBlock",
                Switch::from(settings.sponsorblock.enabled),
                |settings, switch| settings.sponsorblock.enabled = switch.on(),
                cx,
            ));
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(tones.heading("Search & focus"))
            .child(tones.field("Search engine", self.engine_picker(cx)))
            .child(tones.field("Built-in protection", protections))
            .child(self.footer("Next", cx))
    }

    fn import_rows(&self, cx: &mut Context<Self>) -> Div {
        let tones = self.tones();
        let rows = div().flex().flex_col();
        let Some(sources) = self
            .onboarding
            .as_ref()
            .and_then(|tour| tour.sources.as_ref())
        else {
            return rows.child(tones.label("Looking for other browsers…"));
        };
        if sources.is_empty() {
            return rows.child(tones.label("No other browsers found on this PC."));
        }
        rows.children(sources.iter().enumerate().map(|(index, source)| {
            let button = tones.secondary(
                ("onboarding-import", index),
                progress_label(source.progress),
            );
            let button = self.on_tap(
                button,
                move |browser, _, cx| browser.import_source(index, cx),
                cx,
            );
            tones.row(source.profile.label(), button)
        }))
    }

    fn bring_step(&self, cx: &mut Context<Self>) -> Div {
        let tones = self.tones();
        let default_browser = self.on_tap(
            tones.secondary(
                "onboarding-default-browser",
                "Open default browser settings",
            ),
            |_, _, _| open_default_apps_settings(),
            cx,
        );
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(tones.heading("Bring your stuff"))
            .child(tones.label("All optional"))
            .child(tones.field("Import bookmarks & history", self.import_rows(cx)))
            .child(tones.field("Default browser", div().flex().child(default_browser)))
            .child(self.footer("Start browsing", cx))
    }

    fn step_body(&self, step: Step, cx: &mut Context<Self>) -> Div {
        match step {
            Step::Welcome => self.welcome_step(cx),
            Step::Look => self.look_step(cx),
            Step::Search => self.search_step(cx),
            Step::Bring => self.bring_step(cx),
        }
    }

    pub(super) fn onboarding_overlay(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let step = self.onboarding.as_ref()?.step;
        let tones = self.tones();
        let card = div()
            .w(px(CARD_WIDTH_PX))
            .p(px(CARD_PADDING_PX))
            .flex()
            .flex_col()
            .gap_4()
            .rounded(px(CARD_CORNER_PX))
            .bg(rgba(tones.card))
            .text_color(rgba(tones.text))
            .shadow_lg()
            .child(self.tour_header(step, cx))
            .child(self.step_body(step, cx));
        Some(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgba(tones.backdrop))
                .occlude()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(card),
        )
    }
}
