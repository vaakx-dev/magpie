# Magpie UI v1

Arc-inspired layout, drawn natively with GPUI. This is the locked v1 scope; anything not listed here is out of v1. Items marked *(later)* are in scope but not built yet.

## Window

- **Borderless frame**: no OS title bar. Magpie draws its own minimize, maximize and close buttons (close turns red on hover). Empty bar space drags the window; double-click maximizes; Windows Snap works.
- **Vertical sidebar** holds the tabs. No horizontal tab strip.
- **Two sidebar states**, toggled with `Ctrl+B` or `Ctrl+Shift+S`: **Shown** and **Hidden**. When hidden, hovering the sidebar's edge slides it out over the page.
- **Inset viewport**: the page sits in a borderless, rounded-corner panel inset 4px from the window edge, on the Space's tinted frame colour, with 8px corners and a hairline edge. The gap to the sidebar and between Split View panes is the same 4px, so the frame looks the same on every side of the page.

## Settings

`Ctrl+,`, the sidebar gear, Settings in the menu, or "Settings" in the Command Bar opens the built-in `magpie://settings` page in a tab, or focuses the Settings tab that is already open. The address bar shows a `Magpie` badge before the address. A left rail holds Find in Settings under the Settings header, which filters rows across every section, then lists General, Appearance, Layout, Search, Privacy & Security, Downloads, Languages, Add-ons, Import Data and About Magpie, each reachable as an anchor (`magpie://settings#privacy`). Changes apply live in every window and are saved to `%APPDATA%\Magpie\settings.conf`; the page follows the shell's theme.

- **Theme**: *System* (default, follows the Windows app theme live), *Light* or *Dark*. The whole shell switches palette, Space frames have a light and a dark tint, and pages get the matching `prefers-color-scheme`.

Layout:

- **Top bar**: *Off* (default, Pure Arc: navigation, address and window controls live in the sidebar header) or *On* (a slim 36px bar with sidebar toggle, navigation, address bar, Split button, the menu button and window controls).
- **Sidebar**: Shown or Hidden.
- **Sidebar side**: Left (default) or Right. The edge reveal follows it.
- **Sidebar width**: 240px by default; drag the handle on the sidebar's inner edge to resize it between 180px and 360px (never narrower than its header needs). Saved as `sidebar_width`.
- **Window controls**: *Windows* (default; minimize, maximize and close glyphs) or *Mac* (red, yellow and green traffic lights).
- **Window controls side**: Left or Right (default) end of whichever bar hosts them: the top bar when it is on, else the sidebar header. Close always sits on the outer edge: each style keeps its usual order on its home side (Windows on the right, Mac on the left) and mirrors it on the other.
- With the top bar off and the sidebar hidden, hovering the top corner on the sidebar's side shows a floating pill with window controls, sidebar toggle, Back, the host and the menu button.

## Sidebar, top to bottom

1. **Header** (top bar off only): window controls, sidebar toggle, a drag area, Back, Forward and the menu button.
2. **Address bar** (top bar off only; otherwise in the top bar): shows the current site; click (or `Ctrl+L`) opens the Command Bar to edit it. A Reload button sits inside its right edge and turns into Stop while the page loads. On web pages a lock (or **Not secure** on `http://`) sits before the address and opens site info; Magpie pages show the `Magpie` badge instead.
3. **Favorites**: a grid of site icons, three per row, shared by every Space. Click focuses that site's tab or opens it.
4. **Pinned**: tabs kept apart from Today. Closing a pinned tab only unloads it; selecting it loads it again. Folders and persistence across restarts *(later)*.
5. **Today**: ordinary tabs, newest first, with a New Tab row. Auto-archive after 12 hours *(later)*.
6. **Mini Player** for media playing in a hidden tab *(later, needs media support)*.
7. **Spaces switcher**: icons at the very bottom, with the current Space's name. Each Space has its own Pinned and Today lists, name, icon and colour theme. "New space" in the Command Bar adds one.

## Site permissions

- When a site asks for the camera, microphone, notifications, location or clipboard, or opens a pop-up without a click, a card over the page asks: **Block**, **Allow once** or **Allow**. Allow and Block are remembered for that origin (scheme, host and port); a saved rule answers later requests without asking.
- Site info (the lock) shows whether the connection is secure, an Ask / Allow / Block switch for each permission the site has asked for or has a rule for, and **Clear site data**.
- Settings → Privacy & Security lists every saved rule, to change or remove it. Rules are saved to `%APPDATA%\Magpie\permissions.conf`; private windows keep their decisions in memory for that window only.

## Menu

The three-line menu button sits at the right end of the top bar when it is on, else in the sidebar header (and in the floating pill when the sidebar is hidden). It opens a popover that floats over the live page, and Escape or a click outside closes it.

1. **Navigation & View**: New Tab, New Window, New Private Window, Split View (on or off) and Find in Page.
2. **Browser**: Downloads, Settings, the Ad blocker row and Clear Browsing Data, which opens Settings at Privacy & Security. The Ad blocker row has a switch that pauses or resumes blocking on the current site; clicking the row opens Privacy & Security.

## Split View

- Two tabs side by side in the viewport, each in its own rounded panel. The focused pane gets a neutral grey ring; clicking a pane focuses it, and navigation acts on the focused pane.
- Start with `Ctrl+Shift+\` or "Split view" in the Command Bar: it pairs the active tab with the next Today tab, or opens the Command Bar to choose the second page.
- A split shows as one entry in the sidebar. "Close split" separates it again.
- Up to four panes and draggable dividers *(later)*.

## Command Bar

- `Ctrl+T` opens a centred floating palette over the page instead of a new blank tab.
- One input searches open and pinned tabs and runs browser commands (split view, close split, new space, pin, unpin, close tab, toggle sidebar, settings, reload). Enter with plain text opens the address, or a search, in a new Today tab. Arrow keys move the selection; Escape or a click outside closes it.
- Searching history, bookmarks and archived tabs *(later)*.

## Shortcuts

| Keys | Action |
|---|---|
| `Ctrl+B`, `Ctrl+Shift+S` | Toggle sidebar: Shown or Hidden |
| `Ctrl+,` | Settings |
| `Escape` | Close a popover |
| `Ctrl+T` | Command Bar |
| `Ctrl+L` | Edit address |
| `Ctrl+Shift+\` | Split View |
| `Alt+←` / `Alt+→` | Back / Forward |
| `Ctrl+R`, `F5` | Reload |
| `Ctrl+W` | Close tab |
