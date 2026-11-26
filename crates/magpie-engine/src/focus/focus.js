(sheets, apply) => {
  const host = location.hostname;
  const covers = (domain) => host === domain || host.endsWith(`.${domain}`);
  const site = sheets.findIndex(({ domains }) => domains.some(covers));
  if (site === -1) return;

  const sheet = new CSSStyleSheet();
  const fill = (next) => sheet.replaceSync(next[site].css);
  const markPath = () => {
    const root = document.documentElement;
    if (root && root.dataset.magpiePath !== location.pathname) root.dataset.magpiePath = location.pathname;
  };

  fill(sheets);
  document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
  Object.defineProperty(window, apply, { value: fill });
  markPath();
  new MutationObserver(markPath).observe(document, { childList: true, subtree: true });
  addEventListener("popstate", markPath);
}
