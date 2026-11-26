(() => {
  const NATIVE_APP = "magpie";
  const DARKEN = "invert(1) hue-rotate(180deg) contrast(0.85)";
  const RESTORE = "contrast(1.18) hue-rotate(180deg) invert(1)";
  const KEPT = "img, picture, video, canvas, iframe, embed, object, svg image, [style*='background-image']";
  const DARK_LIMIT = 0.5;
  const READER = "magpie-reader";
  const MIN_PARAGRAPH = 25;
  const ANCESTOR_WEIGHTS = [1, 0.5, 0.25];
  const SEMANTIC = "article, main, [role='main'], [itemprop='articleBody']";
  const SEMANTIC_BONUS = 1.25;
  const PARAGRAPHS = "p, pre, blockquote, li, h2, h3, td";
  const CLUTTER = [
    "script", "style", "noscript", "template", "link", "meta", "iframe", "embed", "object", "form", "button",
    "input", "select", "textarea", "nav", "footer", "aside", "dialog", "svg", "canvas",
    "[role='navigation']", "[role='banner']", "[role='complementary']", "[role='dialog']", "[aria-hidden='true']", "[hidden]",
    "[class*='advert' i]", "[class*='sponsor' i]", "[class*='share' i]", "[class*='social' i]", "[class*='related' i]",
    "[class*='newsletter' i]", "[class*='comment' i]", "[class*='promo' i]", "[class*='cookie' i]", "[class*='popup' i]",
  ].join(", ");
  const KEPT_ATTRIBUTES = new Set(["href", "src", "srcset", "sizes", "alt", "title", "colspan", "rowspan", "datetime", "controls", "poster"]);
  const READER_STYLE = `
    :host { all: initial; position: fixed !important; inset: 0 !important; z-index: 2147483647 !important; display: block !important; }
    .reader {
      --page: #ffffff; --text: #1f1f1f; --muted: #5c5c5c; --raised: #f2f2f2;
      position: absolute; inset: 0; overflow-y: auto; background: var(--page); color: var(--text);
      font: 18px/1.7 Charter, Georgia, serif;
    }
    @media (prefers-color-scheme: dark) {
      .reader { --page: #1c1c1c; --text: #ececec; --muted: #a3a3a3; --raised: #303030; }
    }
    .bar { position: sticky; top: 0; display: flex; align-items: center; gap: 8px; padding: 10px 16px; background: var(--page); box-shadow: 0 1px 8px rgb(0 0 0 / 0.08); font: 14px/1 system-ui, sans-serif; }
    .site { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--muted); }
    button { font: inherit; color: var(--text); background: var(--raised); border: 0; border-radius: 8px; padding: 8px 12px; }
    article { max-width: 680px; margin: 0 auto; padding: 24px 20px 96px; overflow-wrap: break-word; }
    h1 { font-size: 1.6em; line-height: 1.25; margin: 0 0 0.8em; }
    a { color: inherit; }
    img, video, picture { max-width: 100%; height: auto; border-radius: 6px; }
    blockquote { margin-inline: 0; padding-left: 1em; color: var(--muted); }
    pre { overflow-x: auto; padding: 12px; background: var(--raised); border-radius: 8px; }
  `;

  const dark = document.createElement("style");
  dark.media = "not all";
  dark.textContent = `
    :root { filter: ${DARKEN} !important; background: #fff; }
    :root :is(${KEPT}) { filter: ${RESTORE} !important; }
    :root :is(${KEPT}) :is(${KEPT}) { filter: none !important; }
  `;
  const zoom = document.createElement("style");
  document.documentElement.append(dark, zoom);

  let forceDark = false;
  let pageDark = false;
  let backgroundPlay = false;

  const keepVisible = (property, visible) => {
    Object.defineProperty(window.wrappedJSObject.Document.prototype, property, {
      configurable: true,
      enumerable: true,
      get: exportFunction(() => (backgroundPlay ? visible : document[property]), window),
    });
  };
  keepVisible("hidden", false);
  keepVisible("visibilityState", "visible");
  window.addEventListener(
    "visibilitychange",
    (event) => {
      if (backgroundPlay) event.stopImmediatePropagation();
    },
    true,
  );

  const brightness = (color) => {
    const [red, green, blue, alpha = 1] = (color.match(/[\d.]+/g) ?? []).map(Number);
    if (alpha === 0) return undefined;
    return (0.2126 * red + 0.7152 * green + 0.0722 * blue) / 255;
  };

  const renderDark = () => {
    dark.media = forceDark && !pageDark ? "all" : "not all";
  };

  const renderZoom = (factor) => {
    zoom.textContent = factor === 1 ? "" : `:root { zoom: ${factor} !important; }`;
  };

  const measure = () => {
    dark.media = "not all";
    const backdrop = [document.body, document.documentElement]
      .filter(Boolean)
      .map((element) => brightness(getComputedStyle(element).backgroundColor))
      .find((value) => value !== undefined);
    pageDark = backdrop !== undefined && backdrop < DARK_LIMIT;
    renderDark();
  };

  const scores = () => {
    const tally = new Map();
    for (const paragraph of document.body.querySelectorAll(PARAGRAPHS)) {
      const length = paragraph.textContent.trim().length;
      if (length < MIN_PARAGRAPH) continue;
      let ancestor = paragraph.parentElement;
      for (const weight of ANCESTOR_WEIGHTS) {
        if (!ancestor || ancestor === document.documentElement) break;
        tally.set(ancestor, (tally.get(ancestor) ?? 0) + length * weight);
        ancestor = ancestor.parentElement;
      }
    }
    return tally;
  };

  const mainContent = () =>
    [...scores()]
      .map(([element, score]) => [element, element.matches(SEMANTIC) ? score * SEMANTIC_BONUS : score])
      .reduce((best, entry) => (entry[1] > best[1] ? entry : best), [document.body, 0])[0];

  const tidy = (root) => {
    root.querySelectorAll(CLUTTER).forEach((element) => element.remove());
    for (const element of root.querySelectorAll("*")) {
      [...element.attributes].filter(({ name }) => !KEPT_ATTRIBUTES.has(name)).forEach(({ name }) => element.removeAttribute(name));
    }
    return root;
  };

  const openReader = () => {
    const host = document.createElement(READER);
    const shadow = host.attachShadow({ mode: "closed" });
    const style = Object.assign(document.createElement("style"), { textContent: READER_STYLE });
    const reader = Object.assign(document.createElement("div"), { className: "reader" });
    const bar = Object.assign(document.createElement("div"), { className: "bar" });
    const site = Object.assign(document.createElement("span"), { className: "site", textContent: location.hostname });
    const exit = Object.assign(document.createElement("button"), { textContent: "Close" });
    const article = document.createElement("article");
    const heading = Object.assign(document.createElement("h1"), { textContent: document.title });
    article.append(heading, tidy(mainContent().cloneNode(true)));
    exit.addEventListener("click", () => host.remove());
    bar.append(site, exit);
    reader.append(bar, article);
    shadow.append(style, reader);
    document.documentElement.append(host);
  };

  const toggleReader = () => {
    const open = document.querySelector(READER);
    if (open) open.remove();
    else openReader();
  };

  const port = browser.runtime.connectNative(NATIVE_APP);
  port.onMessage.addListener((message) => {
    switch (message.kind) {
      case "page":
        forceDark = message.dark;
        backgroundPlay = message.backgroundPlay;
        renderDark();
        renderZoom(message.zoom);
        return;
      case "background":
        backgroundPlay = message.enabled;
        return;
      case "dark":
        forceDark = message.enabled;
        renderDark();
        return;
      case "zoom":
        renderZoom(message.zoom);
        return;
      case "reader":
        toggleReader();
        return;
    }
  });
  port.postMessage({ kind: "page" });
  document.addEventListener("DOMContentLoaded", measure);
})();
