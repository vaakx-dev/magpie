# Magpie

A fast Arc-style Windows web browser written in Rust. The shell (sidebar, tabs, Spaces, Command Bar, popovers) is drawn on the GPU with [GPUI](https://gpui.rs), Zed's UI framework. Web pages render in Microsoft Edge WebView2, so the whole web works, and Magpie's own Rust ad blocker filters every request WebView2 makes.

Magpie also runs on Android, on Mozilla GeckoView, sharing the same Rust ad blocker, tracker stripping and Bitwarden client.

## Features

### Windows

- **WebView2 + GPUI**: a GPU-drawn shell around Microsoft Edge WebView2, so every site works.
- **Spaces** with their own tabs, pinned tabs and folders, and optional **isolated Space profiles** with separate cookies and logins.
- **Favorites**, a Command Bar (`Ctrl+T`) that finds tabs, history, archive and downloads, and an auto-archive for idle tabs.
- **Split View**, **Tab Peeking** (hover a tab for a live preview) and a **Mini Player** for media playing in another tab.
- **Magpie Shield**, a native Rust ad blocker using EasyList, EasyPrivacy and uBlock Origin lists, with cosmetic filtering, YouTube ad handling and per-site pause.
- **SponsorBlock**, **Reader Mode** and **Force Dark Mode** with per-site exceptions.
- A **standalone Bitwarden client**: sign-in with two-step login, unlock, search, autofill, TOTP codes and a password generator. No extension needed.
- A **streaming video downloader** built on yt-dlp and FFmpeg.
- **Privacy controls**: tracker parameter stripping, tracking prevention, Do Not Track, Global Privacy Control, HTTPS-Only mode and clearing browsing data. Private windows save nothing.
- **Site Permissions** for camera, microphone, notifications, location, clipboard and pop-ups.
- **Crash recovery**, **automatic updates** from GitHub Releases, browser data import and six UI languages.

### Android

- Mozilla **GeckoView** with **Jetpack Compose**, and the same Rust engines through `libmagpie_core.so`.
- A single-row bottom address bar with the tab count and a compact floating menu. The menu sits on the right by default; Settings can move it to the left.
- A full-screen tab switcher: Favorites, a two-column grid of tab cards with page previews, Space pills (Personal, Work, Private) and New Tab.
- Spaces with separate GeckoView profiles, plus a Private Space that saves nothing.
- A silent ad blocker that downloads and caches EasyList, EasyPrivacy and uBlock Origin's lists in the background. Also tracker stripping, strict tracking protection, Force Dark and Reader Mode.
- Bitwarden: sign in with two-step login (authenticator, email or YubiKey), unlock, matches for the current site, search, TOTP codes and a password generator.
- History, bookmarks and downloads.

## Install on Windows

Get the latest installer (`Magpie-<version>-setup.exe`) from the Releases page, or build it yourself:

```
powershell -ExecutionPolicy Bypass -File packaging\build-release.ps1
```

This builds `magpie.exe` and stages `dist\Magpie-<version>`, a portable `Magpie-<version>-windows-x64.exe` and zip, and `dist\checksums.txt`. Run `install.ps1` in that folder to install Magpie per user (`%LOCALAPPDATA%\Programs\Magpie`, with a Start menu shortcut). With Inno Setup 6 installed, the script also writes the installer to `dist\Magpie-<version>-setup.exe`.

## Build and run

### Windows

Needs the stable Rust MSVC toolchain (`rustup`), Visual Studio's C++ build tools with a Windows SDK, and the WebView2 Runtime (built into Windows 11).

```
cargo run --release                     # opens Magpie
cargo run --release -- example.com      # opens with tabs for the given addresses
cargo test
```

The first build compiles GPUI and takes a few minutes; later builds are incremental.

### Android

Needs JDK 21, the Android SDK and NDK, and `rustup`. `android/build-rust.ps1` adds the Android and WebAssembly Rust targets, installs `cargo-ndk` if needed, builds `libmagpie_core.so` for `arm64-v8a` and `x86_64` into `android/app/src/main/jniLibs`, and builds the ad blocker's `magpie_blocker.wasm` into `android/app/src/main/assets/extension` (run through `wasm-opt` when it's installed).

```
powershell -ExecutionPolicy Bypass -File android\build-rust.ps1
cd android
.\gradlew :app:assembleDebug :app:testDebugUnitTest
```

Gradle writes one APK per ABI to `android/app/build/outputs/apk/debug`. Without the Rust library the app still runs, with the ad blocker, link cleaning and Bitwarden turned off.

`:app:assembleRelease` signs the release APKs with the key named by `MAGPIE_KEYSTORE_PATH`, `MAGPIE_KEYSTORE_PASSWORD`, `MAGPIE_KEY_ALIAS` and `MAGPIE_KEY_PASSWORD`, or else by `secrets/android-release.properties` (`storeFile`, `storePassword`, `keyAlias`, `keyPassword`, with `storeFile` relative to `secrets/`). With neither, it builds them unsigned.

## Releases

Pushing a `v<version>` tag that matches `Cargo.toml` runs `.github/workflows/release.yml`. It builds the Windows installer, portable exe and zip, and a signed `arm64-v8a` APK, then publishes them with `checksums.txt` to the GitHub release for that tag, where Magpie's updater finds them. The APK is signed with the repository secrets `MAGPIE_KEYSTORE_BASE64` (the keystore, base64-encoded), `MAGPIE_KEYSTORE_PASSWORD`, `MAGPIE_KEY_ALIAS` and `MAGPIE_KEY_PASSWORD`.

## Architecture

| Module | Job |
|---|---|
| `src/webview` | One WebView2 per pane, hosted in a child window of the GPUI window and kept on the pane's rounded viewport rect in physical pixels. Hidden while the Command Bar, a popover or the peeking sidebar is open. Sends navigation, title, Back/Forward state, favicons (as PNG), new-window requests (opened as Magpie tabs) and Magpie's shortcuts back to the shell. Applies the settings to the shared WebView2 profile (preferred color scheme, tracking prevention, download folder), adds `DNT` and `Sec-GPC` headers, upgrades `http://` navigations in HTTPS-Only mode, asks where to save downloads and clears browsing data. Profile in `%LOCALAPPDATA%\Magpie\WebView2` |
| `crates/magpie-engine` | The platform-independent engines shared by desktop and Android: `blocker`, `bitwarden` (vault, TOTP and password generator), `tracking` (tracker parameter stripping), `update` (GitHub Releases check: version comparison, release parsing, picking the asset for the platform, SHA-256 from `checksums.txt` or `<asset>.sha256`, and the 12-hour check schedule) and `file` (atomic JSON writes) |
| `src/address` | Turns Command Bar input into an address or a search with the chosen engine (`magpie://` addresses open only for known pages), and decides which `http://` addresses HTTPS-Only mode upgrades |
| `src/internal` | Built-in `magpie://` pages. WebView2 registers `magpie` as a secure custom scheme with an authority, and `src/webview` answers its requests from the router here: each `Page` is a host (`magpie://settings`) with embedded HTML, CSS and JS, plus the shared `theme.css` and the Magpie icon. Pages talk to the shell with `chrome.webview.postMessage`; messages are accepted only from `magpie://` documents, parsed per page, and answered with `PostWebMessageAsJson` (Settings gets its full state on load and after every change in any window or of the Windows theme) |
| `src/storage` | Local data in `%LOCALAPPDATA%\Magpie`, written atomically: the session (Spaces, Favorites, pinned tabs and folders, Today tabs), browsing history ranked by frecency, the auto-archive of idle Today tabs, and paused Shield sites |
| `src/pages` | The `magpie://history`, `magpie://bookmarks`, `magpie://downloads` and `magpie://newtab` pages: their HTML, CSS and JS (served as separate files under the same strict Content-Security-Policy as Settings), the JSON state each page renders, and the `PageAction` messages they send. Each page asks for its state on load and gets it again with the theme whenever history, the session, downloads or the settings change |
| `src/config` | Settings in `%APPDATA%\Magpie\settings.conf` (`key = value` lines, written atomically; an older `layout.conf` is read once and replaced): theme, layout, search engine, startup, home page, auto-archive, downloads, privacy, Force Dark Mode and Reader Mode |
| `src/darkmode` | Force Dark Mode: a master switch plus per-site exceptions (never dark, or always dark while the switch is off). A document script inverts bright pages with one root filter and restores images, video, canvas, embeds and the reader view, skips pages that are already dark unless the site is forced, and pauses while something is fullscreen |
| `src/reader` | Reader Mode (`Alt+R`): a script run on demand pulls the main article out of a web page into a centered overlay that follows Magpie's theme, with text size, Sans/Serif and Exit (`Alt+R` or `Esc`); closing it leaves the page untouched. The chosen font and size are saved |
| `src/system_theme` | Reads the Windows app theme (`AppsUseLightTheme`) and watches it for changes, so the System theme follows Windows live |
| `src/ui` | The browser shell: borderless frame, sidebar, rail, top bar, Spaces, tabs, Split View, Command Bar, the menu with per-site Shield, and the shell side of `magpie://` pages |
| `crates/magpie-android` | JNI bridge (`libmagpie_core.so`) behind the Android app's `app.magpie.browser.core.MagpieCore`: `init` (locates the vault), the cached and refreshed ad-blocker snapshot (`engine.dat` in the app's files folder, rebuilt from the filter lists when older than a day), tracker stripping, and the Bitwarden calls (status, sign-in, two-step answer and email code, unlock, lock, matches, search, TOTP, password generator) and the update check (`updateDue`, `updateCheck`), all answering in JSON |
| `crates/magpie-engine/src/blocker` | Ad and tracker blocking with the Brave `adblock` engine. `src/webview` checks every WebView2 request against it and answers blocked ones with an empty 204, and a document script asks it for cosmetic hide rules (site rules, plus generic rules for the classes and ids on the page). Starts from a built-in baseline list, then loads EasyList, EasyPrivacy and uBlock Origin's filters and quick fixes (cached in `%LOCALAPPDATA%\Magpie\filters` on Windows and the app's files folder on Android, refreshed every day). On Android, `crates/magpie-blocker-wasm` compiles the same engine to WebAssembly: the app hands the extension a serialized snapshot once, and its background script answers every request and injects cosmetic CSS synchronously, with no trip to the app. Can be paused per site from the menu. On YouTube a document script prunes the ad fields from player responses, skips any ad that still plays and hides promoted slots and the ad-blocker warning |
| `src/sponsorblock` | SponsorBlock for YouTube: fetches crowd-sourced segments (sponsor, self-promo, interaction reminder, intro, outro, preview, off-topic music, filler) and skips the categories chosen in Settings |
| `src/stream_download` | Streaming video downloader: fetches `yt-dlp` and FFmpeg on first use into `%LOCALAPPDATA%\Magpie\tools`, then saves the page's video with progress in Downloads |
| `src/import` | Imports bookmarks and history from Chrome, Edge, Brave and Firefox |
| `src/update`, `src/ui/update.rs` | Updates from GitHub Releases (`vaakx-dev/magpie`): checks on startup at most every 12 hours (Settings › About › Check for updates automatically) or on demand, shows a banner when a release is newer, then downloads the setup or portable `.exe` to `%TEMP%\magpie-update`, verifies its SHA-256 and runs the installer or swaps `magpie.exe` after Magpie exits. On Android the app downloads the APK for the device ABI into its cache and opens the system installer |
| `src/locale` | UI text in English, Spanish, French, German, Japanese and Chinese |
| `src/ui/password` | The standalone Bitwarden client panel: sign in (bitwarden.com, bitwarden.eu or self-hosted) with two-step login, unlock, search, fill, TOTP codes, password generator and saving new logins |
| `src/ui/mini_player`, `src/ui/peek` | The Mini Player for media playing in a background tab, and Tab Peeking: hover a sidebar tab for a live thumbnail |
| `src/storage/permissions.rs`, `src/ui/permission.rs` | Site Permissions: camera, microphone, notifications, location, clipboard and pop-ups, each Ask, Allow or Block per site |
| `src/storage/recovery.rs`, `src/ui/recovery.rs` | Crash recovery: an unclean exit offers to restore the previous session or start fresh |
| `android/` | The Android app: Kotlin, Jetpack Compose and Mozilla GeckoView, calling the shared engines through `crates/magpie-android` |

GPUI draws without DirectComposition (`GPUI_DISABLE_DIRECT_COMPOSITION`), so the WebView2 child windows show above its swap chain. To float shell UI over a live page, a WebView2 cuts that shape out of its window region and GPUI draws through the hole: the Find card sits in the page's top-right corner this way without moving the page.

The session survives restarts: Magpie reopens your Spaces and tabs, loading only the tab you see. Today tabs you haven't focused for 12 hours (configurable in Settings) move to the archive, where the Command Bar still finds them.

All windows run in one process and share one WebView2 environment, the Shield and the settings; a settings change in one window applies to every window. Only the first window Magpie opens owns the saved session: its Spaces, tabs and archive go to `session.json` and `archive.json`. New windows (`Ctrl+N`) start with a fresh tab and a session that is never saved, so they can't overwrite the first window's Spaces or pinned tabs; their visits still go to the shared history. Private windows (`Ctrl+Shift+N`) use WebView2 InPrivate mode and show a Private badge: they save no session, record no history and cache no favicons on disk, while Shield, HTTPS-Only, Do Not Track and Global Privacy Control still apply. WebView2 discards their cookies and cache when the last Private window closes.

Settings (`Ctrl+,`, the sidebar gear, Settings in the menu, or Settings in the Command Bar) is the `magpie://settings` tab, opened once and focused after that; Clear Browsing Data and the Ad blocker row jump to `#privacy`. It has General (startup, home page, auto-archive, default zoom and per-site zoom), Appearance (theme, Favorites size and columns, classic scrollbars), Layout, Search, Privacy & Security (Shield and paused sites, tracking prevention, Do Not Track and Global Privacy Control, HTTPS-Only mode, clearing browsing data), Downloads, Languages, Add-ons (SponsorBlock, Force Dark Mode and its site exceptions, Reader Mode font and size), Import Data and About Magpie, with Find in Settings across all of them.

History, Bookmarks and Downloads in the menu open the `magpie://history`, `magpie://bookmarks` and `magpie://downloads` tabs, focusing one that is already open. History groups visits by day with search, delete and Clear history; Bookmarks lists each Space's Favorites and pinned tabs and folders; Downloads matches downloads by file path. Every `magpie://` page shows a text-only `Magpie` badge in the address bar.

The UI is specified in [docs/ui-spec-v1.md](docs/ui-spec-v1.md).

## Shortcuts

| Keys | Action |
|---|---|
| `Ctrl+T` | Command Bar: open an address or search, and find open, pinned, favorite, archived and visited pages and downloads; also runs commands |
| `Ctrl+L` | Edit the current address |
| `Ctrl+W` | Close tab (pinned tabs only unload) |
| `Ctrl+Shift+T` | Reopen the last closed tab |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | Next / previous tab |
| `Ctrl+1` … `Ctrl+8`, `Ctrl+9` | Go to that tab, or the last tab |
| `Ctrl+D` | Add or remove the page from Favorites |
| `Ctrl+N` | New window |
| `Ctrl+Shift+N` | New Private window |
| `Ctrl+B`, `Ctrl+Shift+S` | Cycle the sidebar: full, rail, hidden; when hidden, hover the sidebar edge to peek |
| `Ctrl+Shift+\` | Split View |
| `Ctrl+F` | Find in page; `Enter` / `Shift+Enter` step through matches |
| `Alt+R` | Reader Mode; again or `Esc` to leave |
| `Ctrl+J` | Downloads |
| `Ctrl+H` | History |
| `Ctrl+,` | Settings |
| `Alt+←` / `Alt+→` | Back / Forward |
| `Ctrl+R`, `F5` | Reload |
| `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | Zoom in / out / reset |
| `Ctrl+P` | Print |
| `Ctrl+S` | Save page |
| `F11` | Full screen |
| `F12`, `Ctrl+Shift+I` | Developer tools |
| `Esc` | Close a popover |

## License

Magpie is released under the [MIT License](LICENSE).

## Credits

The ad blocker uses Brave's [`adblock`](https://github.com/brave/adblock-rust) engine and these filter lists, downloaded at runtime under their own licenses:

- [EasyList and EasyPrivacy](https://easylist.to) by The EasyList authors (GPLv3 / CC BY-SA 3.0)
- [uBlock Origin filters and quick fixes](https://github.com/uBlockOrigin/uAssets) by Raymond Hill and contributors (GPLv3)

Video segments come from [SponsorBlock](https://sponsor.ajay.app) (CC BY-NC-SA 4.0). The video downloader fetches [yt-dlp](https://github.com/yt-dlp/yt-dlp) and [FFmpeg](https://ffmpeg.org) at first use. The Android app is built on [Mozilla GeckoView](https://mozilla.github.io/geckoview/) (MPL 2.0).
