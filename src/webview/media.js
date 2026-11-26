(() => {
  const bridge = window.chrome && window.chrome.webview;
  if (window !== window.top || !bridge) return;
  const BATCH_MS = 50;
  const WATCHED_EVENTS = ["play", "pause", "ended", "emptied", "loadedmetadata"];
  const handlers = new Map();
  const watched = new WeakSet();
  let current = null;
  let sent = "";
  let queued = false;

  const track = () => {
    const metadata = navigator.mediaSession.metadata;
    return {
      kind: "media",
      title: (metadata && metadata.title) || document.title,
      artist: (metadata && metadata.artist) || "",
      state: current.paused ? "paused" : "playing",
    };
  };
  const report = () => {
    queued = false;
    if (!current) return;
    const message = track();
    const json = JSON.stringify(message);
    if (json === sent) return;
    sent = json;
    bridge.postMessage(message);
  };
  const queue = () => {
    if (queued) return;
    queued = true;
    setTimeout(report, BATCH_MS);
  };
  const adopt = (element) => {
    current = element;
    if (!watched.has(element)) {
      watched.add(element);
      WATCHED_EVENTS.forEach((name) => element.addEventListener(name, queue));
    }
    queue();
  };

  const play = HTMLMediaElement.prototype.play;
  HTMLMediaElement.prototype.play = function (...args) {
    adopt(this);
    return play.apply(this, args);
  };
  document.addEventListener("play", (event) => adopt(event.target), true);

  const session = MediaSession.prototype;
  const setActionHandler = session.setActionHandler;
  session.setActionHandler = function (action, handler) {
    handlers.set(action, handler);
    return setActionHandler.call(this, action, handler);
  };
  const metadata = Object.getOwnPropertyDescriptor(session, "metadata");
  Object.defineProperty(session, "metadata", {
    ...metadata,
    set(value) {
      metadata.set.call(this, value);
      queue();
    },
  });

  const run = (action) => {
    const handler = handlers.get(action);
    if (handler) handler({ action });
    return Boolean(handler);
  };
  const press = (selector) => {
    const button = document.querySelector(selector);
    if (button) button.click();
    return Boolean(button);
  };
  const restart = () => {
    if (current) current.currentTime = 0;
  };
  const toggle = () => {
    if (!current) return;
    const action = current.paused ? "play" : "pause";
    if (!run(action)) current[action]();
  };
  const next = () => run("nexttrack") || press(".ytp-next-button");
  const previous = () => run("previoustrack") || press(".ytp-prev-button") || restart();
  const controls = { toggle, next, previous };

  bridge.addEventListener("message", (event) => {
    const control = event.data && controls[event.data.control];
    if (control) control();
  });
})();
