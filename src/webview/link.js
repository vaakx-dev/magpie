(() => {
  const bridge = window.chrome && window.chrome.webview;
  if (window !== window.top || !bridge) return;
  const MIDDLE = 1;
  const PRIMARY = 0;

  const backgrounded = (event) =>
    event.button === MIDDLE || (event.button === PRIMARY && event.ctrlKey && !event.shiftKey);
  const link = (event) => {
    const target = event.target instanceof Element ? event.target.closest("a[href], area[href]") : null;
    const web = target && (target.protocol === "http:" || target.protocol === "https:");
    return web ? target : null;
  };
  const open = (event) => {
    if (event.defaultPrevented || !backgrounded(event)) return;
    const target = link(event);
    if (!target) return;
    event.preventDefault();
    bridge.postMessage({ kind: "background", url: target.href });
  };

  window.addEventListener("click", open);
  window.addEventListener("auxclick", open);
})();
