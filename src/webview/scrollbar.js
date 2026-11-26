(slim) => {
  const sheet = new CSSStyleSheet();
  sheet.replaceSync(`
    * { scrollbar-width: auto !important; scrollbar-color: auto !important; }
    ::-webkit-scrollbar { width: 12px !important; height: 12px !important; background: transparent !important; }
    ::-webkit-scrollbar-track, ::-webkit-scrollbar-corner { background: transparent !important; }
    ::-webkit-scrollbar-button { display: none !important; width: 0 !important; height: 0 !important; }
    ::-webkit-scrollbar-thumb {
      border: 3px solid transparent !important;
      border-radius: 12px !important;
      background: rgba(92, 92, 92, 0.5) padding-box !important;
    }
    ::-webkit-scrollbar-thumb:hover { border-width: 0.5px !important; background-color: rgba(92, 92, 92, 0.75) !important; }
    ::-webkit-scrollbar-thumb:active { border-width: 0.5px !important; background-color: rgba(92, 92, 92, 0.9) !important; }
    @media (prefers-color-scheme: dark) {
      ::-webkit-scrollbar-thumb { background-color: rgba(163, 163, 163, 0.5) !important; }
      ::-webkit-scrollbar-thumb:hover { background-color: rgba(163, 163, 163, 0.75) !important; }
      ::-webkit-scrollbar-thumb:active { background-color: rgba(163, 163, 163, 0.9) !important; }
    }
  `);
  sheet.disabled = !slim;
  const adopt = (root) => {
    root.adoptedStyleSheets = [...root.adoptedStyleSheets, sheet];
    return root;
  };
  const attachShadow = Element.prototype.attachShadow;
  Element.prototype.attachShadow = function (init) {
    return adopt(attachShadow.call(this, init));
  };
  adopt(document);
  const bridge = window.chrome && window.chrome.webview;
  if (window !== window.top || !bridge) return;
  bridge.addEventListener("message", (event) => {
    if (event.data?.kind !== "scrollbars") return;
    sheet.disabled = !event.data.slim;
  });
}
