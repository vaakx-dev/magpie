use gpui::{Context, prelude::*};

use super::browser::Browser;
use crate::locale::Text;

impl Browser {
    pub(super) fn offer_recovery(&mut self, cx: &mut Context<Self>) {
        self.recovery_offered = true;
        cx.notify();
    }

    fn restore_previous_session(&mut self, cx: &mut Context<Self>) {
        self.recovery_offered = false;
        self.resume(cx);
        cx.notify();
    }

    fn start_fresh(&mut self, cx: &mut Context<Self>) {
        self.recovery_offered = false;
        self.archive_today(cx);
    }

    pub(super) fn recovery_banner(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        if !self.recovery_offered {
            return None;
        }
        let buttons = [
            self.banner_button(
                "restore-previous-session",
                Text::RestorePreviousSession,
                Self::restore_previous_session,
                cx,
            ),
            self.banner_button("start-fresh", Text::StartFresh, Self::start_fresh, cx),
        ];
        Some(self.banner(self.text(Text::ClosedUnexpectedly), buttons))
    }
}
