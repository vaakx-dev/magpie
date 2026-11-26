({ font, size, sizes, text }) => {
  const TAG = "magpie-reader";
  const EXIT = "magpie-reader-exit";
  const MIN_PARAGRAPH = 25;
  const ANCESTOR_WEIGHTS = [1, 0.5, 0.25];
  const SEMANTIC_BONUS = 1.25;
  const SEMANTIC = "article, main, [role='main'], [itemprop='articleBody']";
  const PARAGRAPHS = "p, pre, blockquote, li, h2, h3, td";
  const CLUTTER = [
    "script", "style", "noscript", "template", "link", "meta", "iframe", "frame", "embed", "object",
    "form", "button", "input", "select", "textarea", "nav", "footer", "aside", "dialog", "svg", "canvas",
    "[role='navigation']", "[role='banner']", "[role='complementary']", "[role='contentinfo']", "[role='dialog']",
    "[aria-hidden='true']", "[hidden]",
    "[class*='advert' i]", "[id*='advert' i]", "[class*='sponsor' i]", "[class*='sidebar' i]", "[id*='sidebar' i]",
    "[class*='share' i]", "[class*='social' i]", "[class*='related' i]", "[class*='newsletter' i]",
    "[class*='comment' i]", "[id*='comment' i]", "[class*='promo' i]", "[class*='cookie' i]", "[class*='popup' i]",
  ].join(", ");
  const KEPT_ATTRIBUTES = new Set([
    "href", "src", "srcset", "sizes", "alt", "title", "colspan", "rowspan", "datetime", "lang", "dir", "controls", "poster",
  ]);
  const LAZY_SOURCES = ["data-src", "data-lazy-src", "data-original"];
  const STYLE = `
    :host { all: initial; position: fixed !important; inset: 0 !important; z-index: 2147483647 !important; display: block !important; color-scheme: light dark; }
    .reader {
      --page: #ffffff; --text: #1d1d1f; --muted: #66666d; --hairline: #e5e5e8; --raised: #ededef;
      position: absolute; inset: 0; overflow-y: auto; outline: none;
      background: var(--page); color: var(--text);
    }
    @media (prefers-color-scheme: dark) {
      .reader { --page: #1b1b1d; --text: #ededef; --muted: #a0a0a7; --hairline: #323235; --raised: #303033; }
    }
    .bar {
      position: sticky; top: 0; display: flex; align-items: center; gap: 4px; padding: 8px 16px;
      background: var(--page); border-bottom: 1px solid var(--hairline);
      font: 13px/1 "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif;
    }
    .site { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--muted); }
    button {
      font: inherit; color: var(--text); background: transparent; border: 0; border-radius: 6px;
      padding: 6px 10px; cursor: pointer;
    }
    button:hover, button[aria-pressed='true'] { background: var(--raised); }
    button:focus-visible { outline: 2px solid var(--muted); }
    article { max-width: 680px; margin: 0 auto; padding: 40px 24px 120px; font-size: var(--size); line-height: 1.7; overflow-wrap: break-word; }
    article.sans { font-family: "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif; }
    article.serif { font-family: Charter, "Iowan Old Style", Georgia, Cambria, "Times New Roman", serif; }
    h1 { font-size: 1.9em; line-height: 1.2; margin: 0 0 0.8em; }
    h2, h3, h4 { line-height: 1.3; margin: 1.6em 0 0.6em; }
    p, ul, ol, blockquote, pre, figure, table { margin: 0 0 1.1em; }
    a { color: inherit; text-decoration: underline; text-underline-offset: 2px; }
    img, video, picture { max-width: 100%; height: auto; border-radius: 4px; }
    figure { margin-inline: 0; }
    figcaption { font-size: 0.85em; color: var(--muted); margin-top: 0.4em; }
    blockquote { margin-inline: 0; padding-left: 1em; border-left: 3px solid var(--hairline); color: var(--muted); }
    pre, code { font-family: "Cascadia Code", Consolas, monospace; font-size: 0.9em; }
    pre { overflow-x: auto; padding: 12px 14px; background: var(--raised); border-radius: 6px; }
    table { border-collapse: collapse; display: block; overflow-x: auto; }
    td, th { border: 1px solid var(--hairline); padding: 4px 8px; }
    hr { border: 0; border-top: 1px solid var(--hairline); }
  `;

  const open = document.querySelector(TAG);
  if (open) {
    open.dispatchEvent(new Event(EXIT));
    return;
  }

  const textLength = (element) => element.textContent.trim().length;

  const scores = () => {
    const tally = new Map();
    for (const paragraph of document.body.querySelectorAll(PARAGRAPHS)) {
      const length = textLength(paragraph);
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

  const mainContent = () => {
    const ranked = [...scores()].map(([element, score]) => [element, element.matches(SEMANTIC) ? score * SEMANTIC_BONUS : score]);
    const best = ranked.reduce((top, entry) => (entry[1] > top[1] ? entry : top), [document.body, 0]);
    return best[0];
  };

  const keepAttributes = (element) => {
    const lazy = LAZY_SOURCES.map((name) => element.getAttribute(name)).find(Boolean);
    if (element.localName === "img" && lazy && !element.getAttribute("src")) element.setAttribute("src", lazy);
    for (const { name } of [...element.attributes]) {
      if (!KEPT_ATTRIBUTES.has(name)) element.removeAttribute(name);
    }
  };

  const cleanCopy = (source) => {
    const copy = source.cloneNode(true);
    const originals = [...source.querySelectorAll("*")];
    const copies = [...copy.querySelectorAll("*")];
    const hidden = (index) => !originals[index].checkVisibility();
    copies.filter((element, index) => element.matches(CLUTTER) || hidden(index)).forEach((element) => element.remove());
    [copy, ...copy.querySelectorAll("*")].forEach(keepAttributes);
    return copy;
  };

  const headline = (content) => {
    if (content.querySelector("h1")) return [];
    const title = document.querySelector("meta[property='og:title']")?.content || document.title;
    return title ? [Object.assign(document.createElement("h1"), { textContent: title })] : [];
  };

  const control = (label, describe, act) => {
    const button = Object.assign(document.createElement("button"), { type: "button", textContent: label, title: describe });
    button.setAttribute("aria-label", describe);
    button.addEventListener("click", act);
    return button;
  };

  const content = cleanCopy(mainContent());
  const article = document.createElement("article");
  article.append(...headline(content), ...content.childNodes);

  const host = document.createElement(TAG);
  const shadow = host.attachShadow({ mode: "closed" });
  const sheet = new CSSStyleSheet();
  sheet.replaceSync(STYLE);
  shadow.adoptedStyleSheets = [sheet];
  const reader = Object.assign(document.createElement("div"), { className: "reader", tabIndex: -1 });
  const bar = Object.assign(document.createElement("div"), { className: "bar" });
  const site = Object.assign(document.createElement("span"), { className: "site", textContent: location.hostname });

  let style = { font, size };
  const fonts = [
    ["Sans", control(text.sans, text.sans, () => restyle({ font: "Sans" }))],
    ["Serif", control(text.serif, text.serif, () => restyle({ font: "Serif" }))],
  ];
  const stepSize = (step) => {
    const index = Math.min(Math.max(sizes.indexOf(style.size) + step, 0), sizes.length - 1);
    restyle({ size: sizes[index] });
  };
  const paint = () => {
    article.className = style.font.toLowerCase();
    article.style.setProperty("--size", `${style.size}px`);
    fonts.forEach(([name, button]) => button.setAttribute("aria-pressed", String(name === style.font)));
  };
  const restyle = (change) => {
    style = { ...style, ...change };
    paint();
    window.chrome?.webview?.postMessage({ kind: "magpie_reader", ...style });
  };

  const root = document.documentElement;
  const overflow = [root.style.getPropertyValue("overflow"), root.style.getPropertyPriority("overflow")];
  const escape = (event) => {
    if (event.key !== "Escape") return;
    event.preventDefault();
    event.stopImmediatePropagation();
    exit();
  };
  const exit = () => {
    host.remove();
    root.style.setProperty("overflow", ...overflow);
    removeEventListener("keydown", escape, true);
  };

  bar.append(
    site,
    control("A−", text.smaller, () => stepSize(-1)),
    control("A+", text.larger, () => stepSize(1)),
    ...fonts.map(([, button]) => button),
    control(text.exit, `${text.exit} (Alt+R, Esc)`, exit),
  );
  reader.append(bar, article);
  shadow.append(reader);
  paint();
  host.addEventListener(EXIT, exit);
  addEventListener("keydown", escape, true);
  root.style.setProperty("overflow", "hidden", "important");
  root.append(host);
  reader.focus();
}
