use std::time::{Duration, Instant};

use gpui::{
    AnyElement, Context, Corner, Div, Entity, FontWeight, ObjectFit, Stateful, Task, anchored,
    deferred, div, img, point, prelude::*, px, rgba,
};
use url::Url;

use super::{
    browser::Browser,
    favicon::site_icon,
    frame::INSET_PX,
    marquee::{Marquee, marquee},
    sidebar::tab_title,
    surface::{PANEL_CORNER_PX, surface},
};
use crate::{
    config::Side,
    locale::Text,
    storage::session::{Tab, TabId},
    webview::{Memory, SNAPSHOT_HEIGHT_PX, SNAPSHOT_WIDTH_PX, WebView},
};
use magpie_engine::blocker::web_site;

const DELAY: Duration = Duration::from_millis(375);
const WARM_FOR: Duration = Duration::from_millis(300);
const PADDING_PX: f32 = 10.0;
const PADDING_Y_PX: f32 = 6.0;
const WIDTH_PX: f32 = SNAPSHOT_WIDTH_PX + 2.0 * PADDING_PX;
const SIDEBAR_GAP_PX: f32 = INSET_PX + 6.0;
const WINDOW_MARGIN_PX: f32 = 8.0;
const THUMBNAIL_CORNER_PX: f32 = 6.0;
const TITLE_LINE_PX: f32 = 18.0;
const DETAIL_LINE_PX: f32 = 16.0;

pub enum Peek {
    Idle {
        hidden_at: Option<Instant>,
    },
    Dismissed {
        tab: TabId,
    },
    Waiting {
        tab: TabId,
        _delay: Task<()>,
    },
    Shown {
        tab: TabId,
        memory: Option<Memory>,
        title_px: Option<f32>,
        _measure: Task<()>,
    },
}

impl Default for Peek {
    fn default() -> Self {
        Self::Idle { hidden_at: None }
    }
}

impl Peek {
    fn tab(&self) -> Option<TabId> {
        match self {
            Self::Idle { .. } => None,
            Self::Dismissed { tab } | Self::Waiting { tab, .. } | Self::Shown { tab, .. } => {
                Some(*tab)
            }
        }
    }

    fn warm(&self) -> bool {
        match self {
            Self::Shown { .. } => true,
            Self::Idle { hidden_at } => hidden_at.is_some_and(|at| at.elapsed() < WARM_FOR),
            Self::Dismissed { .. } | Self::Waiting { .. } => false,
        }
    }

    fn left(&self) -> Self {
        let hidden_at = matches!(self, Self::Shown { .. }).then(Instant::now);
        Self::Idle { hidden_at }
    }
}

fn domain(address: &str) -> String {
    Url::parse(address)
        .ok()
        .as_ref()
        .and_then(web_site)
        .unwrap_or_else(|| address.to_owned())
}

impl Browser {
    pub(super) fn peekable(
        &self,
        row: Stateful<Div>,
        id: TabId,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        row.on_mouse_move(cx.listener(move |browser, _, _, cx| browser.hover_tab(id, cx)))
            .on_hover(cx.listener(move |browser, hovered: &bool, _, cx| {
                if !hovered {
                    browser.leave_tab(id, cx);
                }
            }))
            .on_any_mouse_down(cx.listener(move |browser, _, _, cx| browser.dismiss_peek(id, cx)))
            .children(self.peek_card(id, cx))
    }

    pub(super) fn stop_peeking(&mut self, cx: &mut Context<Self>) {
        if self.peek.tab().is_some() {
            self.peek = Peek::default();
            cx.notify();
        }
    }

    fn dismiss_peek(&mut self, id: TabId, cx: &mut Context<Self>) {
        self.peek = Peek::Dismissed { tab: id };
        cx.notify();
    }

    fn hover_tab(&mut self, id: TabId, cx: &mut Context<Self>) {
        if self.peek.tab() == Some(id) || cx.has_active_drag() {
            return;
        }
        match self.peek.warm() {
            true => self.peek_at(id, cx),
            false => self.wait_to_peek(id, cx),
        }
    }

    fn leave_tab(&mut self, id: TabId, cx: &mut Context<Self>) {
        if self.peek.tab() == Some(id) {
            self.peek = self.peek.left();
            cx.notify();
        }
    }

    fn wait_to_peek(&mut self, id: TabId, cx: &mut Context<Self>) {
        let delay = cx.spawn(async move |browser, cx| {
            cx.background_executor().timer(DELAY).await;
            browser
                .update(cx, |browser, cx| browser.peek_at(id, cx))
                .ok();
        });
        self.peek = Peek::Waiting {
            tab: id,
            _delay: delay,
        };
    }

    fn refresh_thumbnail(&mut self, id: TabId, views: &[Entity<WebView>], cx: &mut Context<Self>) {
        let Some(page) = self.current().tab(id).map(|tab| tab.url().to_owned()) else {
            return;
        };
        self.thumbnails.load(&page);
        if self.thumbnails.fresh(&page) {
            return;
        }
        for view in views {
            view.update(cx, WebView::capture);
        }
    }

    fn peek_at(&mut self, id: TabId, cx: &mut Context<Self>) {
        let views: Vec<_> = self
            .live
            .get(&id)
            .into_iter()
            .flatten()
            .map(|pane| pane.view.clone())
            .collect();
        self.refresh_thumbnail(id, &views, cx);
        self.peek = Peek::Shown {
            tab: id,
            memory: None,
            title_px: None,
            _measure: self.measure(id, &views, cx),
        };
        cx.notify();
    }

    fn measure(&self, id: TabId, views: &[Entity<WebView>], cx: &mut Context<Self>) -> Task<()> {
        if self.asleep(id, cx) {
            return Task::ready(());
        }
        let frames = views
            .iter()
            .filter_map(|view| view.read(cx).frame())
            .collect();
        let measured = self.runtime.memory(frames);
        cx.spawn(async move |browser, cx| match measured.await {
            Ok(memory) => browser
                .update(cx, |browser, cx| browser.measured(id, memory, cx))
                .unwrap_or_default(),
            Err(error) => eprintln!("magpie: could not measure the memory of a tab: {error}"),
        })
    }

    fn measured(&mut self, id: TabId, measured: Memory, cx: &mut Context<Self>) {
        if let Peek::Shown { tab, memory, .. } = &mut self.peek
            && *tab == id
        {
            *memory = Some(measured);
            cx.notify();
        }
    }

    fn measured_title(&mut self, id: TabId, width_px: f32, cx: &mut Context<Self>) {
        if let Peek::Shown { tab, title_px, .. } = &mut self.peek
            && *tab == id
            && *title_px != Some(width_px)
        {
            *title_px = Some(width_px);
            cx.notify();
        }
    }

    fn title(&self, tab: &Tab, title_px: Option<f32>, cx: &Context<Self>) -> Div {
        let browser = cx.weak_entity();
        let id = tab.id;
        let line = Marquee {
            id: ("peek-title", id.0).into(),
            text: tab_title(tab).into(),
            width_px: SNAPSHOT_WIDTH_PX,
            text_px: title_px,
        };
        marquee(line, move |width_px, cx| {
            browser
                .update(cx, |browser, cx| browser.measured_title(id, width_px, cx))
                .ok();
        })
        .font_weight(FontWeight::SEMIBOLD)
        .line_height(px(TITLE_LINE_PX))
    }

    fn thumbnail(&self, tab: &Tab, cx: &mut Context<Self>) -> AnyElement {
        let colors = self.colors();
        let frame = div()
            .w(px(SNAPSHOT_WIDTH_PX))
            .h(px(SNAPSHOT_HEIGHT_PX))
            .rounded(px(THUMBNAIL_CORNER_PX));
        match self.thumbnails.image(tab.url()) {
            Some(image) => frame
                .child(
                    img(image)
                        .size_full()
                        .rounded(px(THUMBNAIL_CORNER_PX))
                        .object_fit(ObjectFit::Cover),
                )
                .into_any_element(),
            None => frame
                .bg(rgba(colors.raised))
                .flex()
                .items_center()
                .justify_center()
                .child(site_icon(&self.favicons, tab.url(), rgba(colors.faint), cx))
                .into_any_element(),
        }
    }

    fn memory_label(&self, id: TabId, memory: Option<Memory>, cx: &Context<Self>) -> String {
        if self.asleep(id, cx) {
            return self.text(Text::SleepingMemorySaved).to_owned();
        }
        memory
            .map(|memory| {
                let megabytes = format!("{} MB", memory.megabytes());
                self.locale().fill(Text::MemoryUsage, &megabytes)
            })
            .unwrap_or_default()
    }

    fn card(
        &self,
        tab: &Tab,
        memory: Option<Memory>,
        title_px: Option<f32>,
        cx: &mut Context<Self>,
    ) -> Div {
        let colors = self.colors();
        let line = || {
            div()
                .text_xs()
                .line_height(px(DETAIL_LINE_PX))
                .min_h(px(DETAIL_LINE_PX))
                .text_color(rgba(colors.muted))
                .truncate()
        };
        surface(colors)
            .w(px(WIDTH_PX))
            .px(px(PADDING_PX))
            .py(px(PADDING_Y_PX))
            .rounded(px(PANEL_CORNER_PX))
            .flex()
            .flex_col()
            .gap_1()
            .child(self.title(tab, title_px, cx))
            .child(line().child(domain(tab.url())))
            .child(self.thumbnail(tab, cx))
            .child(line().child(self.memory_label(tab.id, memory, cx)))
    }

    fn beside_sidebar(&self, card: Div) -> Div {
        let floating = anchored().snap_to_window_with_margin(px(WINDOW_MARGIN_PX));
        let (spot, floating) = match self.layout().sidebar_side {
            Side::Left => (
                div().left_full(),
                floating
                    .anchor(Corner::TopLeft)
                    .offset(point(px(SIDEBAR_GAP_PX), px(0.0))),
            ),
            Side::Right => (
                div().right_full(),
                floating
                    .anchor(Corner::TopRight)
                    .offset(point(px(-SIDEBAR_GAP_PX), px(0.0))),
            ),
        };
        let see_through = self.cutouts.see_through(PANEL_CORNER_PX).child(card);
        spot.absolute()
            .top_0()
            .child(deferred(floating.child(see_through)))
    }

    fn peek_card(&self, id: TabId, cx: &mut Context<Self>) -> Option<Div> {
        let Peek::Shown {
            tab,
            memory,
            title_px,
            ..
        } = &self.peek
        else {
            return None;
        };
        let tab = self.current().tab(*tab).filter(|tab| tab.id == id)?;
        let card = self.card(tab, *memory, *title_px, cx);
        Some(self.beside_sidebar(card))
    }
}
