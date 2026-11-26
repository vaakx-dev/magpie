use std::{rc::Rc, time::Duration};

use gpui::{
    Animation, AnimationExt, AnyElement, App, Div, ElementId, SharedString, div, prelude::*, px,
};

const HOLD: Duration = Duration::from_millis(600);
const SPEED_PX_PER_SEC: f32 = 40.0;
const GAP_PX: f32 = 40.0;

pub struct Marquee {
    pub id: ElementId,
    pub text: SharedString,
    pub width_px: f32,
    pub text_px: Option<f32>,
}

struct Travel {
    distance_px: f32,
    scroll: Duration,
}

impl Travel {
    fn new(text_px: f32) -> Self {
        let distance_px = text_px + GAP_PX;
        Self {
            distance_px,
            scroll: Duration::from_secs_f32(distance_px / SPEED_PX_PER_SEC),
        }
    }

    fn cycle(&self) -> Duration {
        HOLD + self.scroll
    }

    fn offset_px(&self, delta: f32) -> f32 {
        let elapsed = self.cycle().mul_f32(delta);
        let scrolled = elapsed.saturating_sub(HOLD).as_secs_f32() / self.scroll.as_secs_f32();
        scrolled.min(1.0) * self.distance_px
    }
}

fn copy(text: &SharedString) -> Div {
    div().flex_none().whitespace_nowrap().child(text.clone())
}

fn rolling(strip: Div, id: ElementId, text: &SharedString, text_px: f32) -> AnyElement {
    let travel = Travel::new(text_px);
    strip
        .child(copy(text))
        .child(div().flex_none().w(px(GAP_PX)))
        .child(copy(text))
        .with_animation(
            id,
            Animation::new(travel.cycle()).repeat(),
            move |strip, delta| strip.ml(px(-travel.offset_px(delta))),
        )
        .into_any_element()
}

pub fn marquee(marquee: Marquee, measured: impl Fn(f32, &mut App) + 'static) -> Div {
    let measured = Rc::new(measured);
    let known_px = marquee.text_px;
    let strip = div()
        .flex()
        .flex_none()
        .on_children_prepainted(move |copies, _, cx| {
            let Some(text_px) = copies
                .first()
                .map(|first| f32::from(first.size.width))
                .filter(|text_px| Some(*text_px) != known_px)
            else {
                return;
            };
            let measured = Rc::clone(&measured);
            cx.defer(move |cx| measured(text_px, cx));
        });
    let overflowing = marquee
        .text_px
        .filter(|text_px| *text_px > marquee.width_px);
    let line = match overflowing {
        Some(text_px) => rolling(strip, marquee.id, &marquee.text, text_px),
        None => strip.child(copy(&marquee.text)).into_any_element(),
    };
    div().w(px(marquee.width_px)).overflow_hidden().child(line)
}
