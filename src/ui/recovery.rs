use std::time::Duration;

use gpui::{Context, Div, Task};

use super::browser::Browser;
use crate::locale::Text;

const SHOWN_FOR: Duration = Duration::from_secs(8);

pub(super) struct Notice {
    _expiry: Task<()>,
}

fn expire(cx: &mut Context<Browser>) -> Task<()> {
    cx.spawn(async move |browser, cx| {
        cx.background_executor().timer(SHOWN_FOR).await;
        browser.update(cx, Browser::dismiss_recovery).ok();
    })
}

impl Browser {
    pub(super) fn recover(&mut self, cx: &mut Context<Self>) {
        self.resume(cx);
        self.recovery = Some(Notice {
            _expiry: expire(cx),
        });
        cx.notify();
    }

    pub(super) fn dismiss_recovery(&mut self, cx: &mut Context<Self>) {
        self.recovery = None;
        cx.notify();
    }

    fn start_fresh(&mut self, cx: &mut Context<Self>) {
        self.dismiss_recovery(cx);
        self.archive_today(cx);
    }

    pub(super) fn recovery_banner(&self, cx: &mut Context<Self>) -> Option<Div> {
        self.recovery.as_ref()?;
        let buttons = [
            self.banner_button("start-fresh", Text::StartFresh, Self::start_fresh, cx),
            self.banner_close("dismiss-recovery", Self::dismiss_recovery, cx),
        ];
        Some(self.banner(self.text(Text::ClosedUnexpectedly), buttons))
    }
}
