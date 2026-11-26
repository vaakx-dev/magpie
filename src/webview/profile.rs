use std::path::PathBuf;

use futures::channel::oneshot;
use webview2_com::{
    ClearBrowsingDataCompletedHandler, Microsoft::Web::WebView2::Win32::*, take_pwstr,
};
use windows::{
    Win32::Foundation::E_ABORT,
    Win32::UI::Shell::{FOLDERID_Downloads, KF_FLAG_DEFAULT, SHGetKnownFolderPath},
    core::{Error, HSTRING, Interface, Result},
};

use super::native::Native;
use crate::config::{Settings, Theme, TrackingPrevention};

const CLEARED_DATA: COREWEBVIEW2_BROWSING_DATA_KINDS = COREWEBVIEW2_BROWSING_DATA_KINDS(
    COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_SITE.0
        | COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE.0
        | COREWEBVIEW2_BROWSING_DATA_KINDS_BROWSING_HISTORY.0
        | COREWEBVIEW2_BROWSING_DATA_KINDS_DOWNLOAD_HISTORY.0,
);

pub struct Profile {
    profile: ICoreWebView2Profile,
    _anchor: Native,
}

fn color_scheme(theme: Theme) -> COREWEBVIEW2_PREFERRED_COLOR_SCHEME {
    match theme {
        Theme::System => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_AUTO,
        Theme::Light => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_LIGHT,
        Theme::Dark => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_DARK,
    }
}

fn tracking_level(tracking: TrackingPrevention) -> COREWEBVIEW2_TRACKING_PREVENTION_LEVEL {
    match tracking {
        TrackingPrevention::Basic => COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_BASIC,
        TrackingPrevention::Balanced => COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_BALANCED,
        TrackingPrevention::Strict => COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_STRICT,
    }
}

pub fn default_downloads() -> Option<PathBuf> {
    let path = unsafe { SHGetKnownFolderPath(&FOLDERID_Downloads, KF_FLAG_DEFAULT, None) }.ok()?;
    Some(PathBuf::from(take_pwstr(path)))
}

impl Profile {
    pub fn new(anchor: Native) -> Result<Self> {
        let profile = unsafe { anchor.webview.cast::<ICoreWebView2_13>()?.Profile() }?;
        Ok(Self {
            profile,
            _anchor: anchor,
        })
    }

    pub fn apply(&self, settings: &Settings) -> Result<()> {
        unsafe {
            self.profile
                .SetPreferredColorScheme(color_scheme(settings.theme))
        }?;
        let tracking = tracking_level(settings.privacy.tracking);
        unsafe {
            self.profile
                .cast::<ICoreWebView2Profile3>()?
                .SetPreferredTrackingPreventionLevel(tracking)
        }?;
        let folder = settings.general.downloads.folder.clone();
        match folder.or_else(default_downloads) {
            Some(folder) => unsafe {
                self.profile
                    .SetDefaultDownloadFolderPath(&HSTRING::from(folder.as_os_str()))
            },
            None => Ok(()),
        }
    }

    pub fn delete(&self) -> Result<()> {
        unsafe { self.profile.cast::<ICoreWebView2Profile8>()?.Delete() }
    }

    pub async fn clear(&self) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        let completed = ClearBrowsingDataCompletedHandler::create(Box::new(move |result| {
            sender.send(result).ok();
            Ok(())
        }));
        unsafe {
            self.profile
                .cast::<ICoreWebView2Profile2>()?
                .ClearBrowsingData(CLEARED_DATA, &completed)
        }?;
        receiver.await.map_err(|_| Error::from(E_ABORT))?
    }
}
