# Magpie

Minimal, fast, privacy-first browser for **Windows** and **Android**.

Built in Rust with **GPUI + WebView2** on Windows and **Jetpack Compose + GeckoView** on Android, sharing one Rust core for ad blocking, distraction blocking, tracker stripping, Bitwarden, and auto-updates.

## Download

Grab the latest Windows installer, portable `.exe`, or Android APK from [Releases](https://github.com/vaakx-dev/magpie/releases).

## Features

- **Clean, monochrome UI** — Spaces, Favorites, Split View, Tab Peeking, and a Command Bar (`Ctrl+T`) on Windows; gesture-driven bottom bar and fast tab switcher on Android.
- **Built-in Ad & Tracker Blocker** — native Rust + WASM engine powered by EasyList, EasyPrivacy, and uBlock Origin lists, plus URL tracker stripping and HTTPS-Only mode.
- **Distraction Blocker** — hides YouTube Shorts (and redirects `/shorts/` to normal watch pages), Playables, Members-only videos, Reels, and algorithmic feeds across PC and mobile.
- **Built-in Add-ons** — SponsorBlock, Reader Mode, Force Dark Mode, Background Play (Android), and video downloads.
- **Standalone Bitwarden Client** — sign in with 2FA, fill logins right under the field or from the address-bar key, save new and changed passwords, unlock with Windows Hello, copy TOTP codes, and generate passwords with no extension required. On Android, Magpie hands logins to your phone's autofill service by default.
- **Auto-Updates** — checks GitHub Releases automatically on both Windows and Android.

## Build from Source

**Windows** (Rust MSVC toolchain + WebView2 Runtime):

```powershell
cargo run --release
```

**Android** (JDK 21, Android SDK/NDK, Rust):

```powershell
powershell -ExecutionPolicy Bypass -File android\build-rust.ps1
cd android && .\gradlew :app:assembleDebug
```

## License

[MIT](LICENSE). Uses [`adblock-rust`](https://github.com/brave/adblock-rust) (EasyList, EasyPrivacy, uBlock Origin filters), [SponsorBlock](https://sponsor.ajay.app), and [Mozilla GeckoView](https://mozilla.github.io/geckoview/).
