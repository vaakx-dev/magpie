((token) => {
  const bridge = window.chrome && window.chrome.webview;
  if (window !== window.top || !bridge) return;
  const post = bridge.postMessage.bind(bridge);
  const stringify = JSON.stringify;
  const create = Object.create;
  const MIDDLE = 1;
  const PRIMARY = 0;

  const backgrounded = (event) =>
    event.button === MIDDLE || (event.button === PRIMARY && event.ctrlKey && !event.shiftKey);
  const link = (event) => {
    const target = event.target instanceof Element ? event.target.closest("a[href], area[href]") : null;
    const web = target && (target.protocol === "http:" || target.protocol === "https:");
    return web ? target : null;
  };
  const send = (url) => {
    const message = create(null);
    message.kind = "background";
    message.url = url;
    message.token = token;
    post(stringify(message));
  };
  const open = (event) => {
    if (!event.isTrusted || event.defaultPrevented || !backgrounded(event)) return;
    const target = link(event);
    if (!target) return;
    event.preventDefault();
    send(target.href);
  };

  window.addEventListener("click", open);
  window.addEventListener("auxclick", open);
})
