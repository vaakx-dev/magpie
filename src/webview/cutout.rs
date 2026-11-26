use std::{cell::RefCell, rc::Rc};

use gpui::{Bounds, Div, Pixels, div, px};

use super::native::Rounded;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cutout {
    bounds: Bounds<Pixels>,
    corner_px: f32,
}

#[derive(Clone, Default)]
pub struct Cutouts(Rc<RefCell<Vec<Cutout>>>);

pub(super) fn physical(logical: Pixels, scale: f32) -> i32 {
    (f32::from(logical) * scale).round() as i32
}

impl Cutout {
    fn within(&self, frame: Bounds<Pixels>, scale: f32) -> Rounded {
        let offset = self.bounds.origin - frame.origin;
        Rounded {
            x: physical(offset.x, scale),
            y: physical(offset.y, scale),
            width: physical(self.bounds.size.width, scale),
            height: physical(self.bounds.size.height, scale),
            corner: physical(px(2.0 * self.corner_px), scale),
        }
    }
}

impl Cutouts {
    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    pub fn see_through(&self, corner_px: f32) -> Div {
        let cutouts = Rc::clone(&self.0);
        div().on_children_prepainted(move |children, _, _| {
            let cut = children
                .into_iter()
                .map(|bounds| Cutout { bounds, corner_px });
            cutouts.borrow_mut().extend(cut);
        })
    }

    pub(super) fn within(&self, frame: Bounds<Pixels>, scale: f32) -> Vec<Rounded> {
        self.0
            .borrow()
            .iter()
            .filter(|cutout| cutout.bounds.intersects(&frame))
            .map(|cutout| cutout.within(frame, scale))
            .collect()
    }
}
