use webview2_com::{
    IsDocumentPlayingAudioChangedEventHandler, IsMutedChangedEventHandler,
    Microsoft::Web::WebView2::Win32::{ICoreWebView2, ICoreWebView2_8},
};
use windows::core::{BOOL, Interface, Result};

use super::signal::{Signal, Signals};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Audio {
    pub playing: bool,
    pub muted: bool,
}

fn audio(webview: &ICoreWebView2_8) -> Result<Audio> {
    let mut playing = BOOL::default();
    let mut muted = BOOL::default();
    unsafe { webview.IsDocumentPlayingAudio(&mut playing) }?;
    unsafe { webview.IsMuted(&mut muted) }?;
    Ok(Audio {
        playing: playing.as_bool(),
        muted: muted.as_bool(),
    })
}

fn report(sender: Option<ICoreWebView2>, signals: &Signals) -> Result<()> {
    let Some(sender) = sender else {
        return Ok(());
    };
    signals.send(Signal::Audio(audio(&sender.cast()?)?));
    Ok(())
}

pub fn wire(webview: &ICoreWebView2, signals: &Signals) -> Result<()> {
    let webview = webview.cast::<ICoreWebView2_8>()?;
    let playing_signals = signals.clone();
    let playing = IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |sender, _| {
        report(sender, &playing_signals)
    }));
    unsafe { webview.add_IsDocumentPlayingAudioChanged(&playing, &mut 0) }?;
    let muted_signals = signals.clone();
    let muted = IsMutedChangedEventHandler::create(Box::new(move |sender, _| {
        report(sender, &muted_signals)
    }));
    unsafe { webview.add_IsMutedChanged(&muted, &mut 0) }
}
