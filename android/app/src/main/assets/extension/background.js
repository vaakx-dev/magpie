const NATIVE_APP = "magpie";
const BLOCKER = "magpie_blocker.wasm";
const MAX_UTF8_BYTES_PER_CHAR = 3;
const BASE64_BYTES_PER_CHAR = 3 / 4;
const FIELD_SEPARATOR = "\n";
const WEB_PAGE = /^https?:/;

const encoder = new TextEncoder();
const decoder = new TextDecoder();
let blocker;
let blocking = true;
let focus = {};
let sponsorBlock;

const exchange = (length) => {
  const pointer = blocker.exchange(length);
  return new Uint8Array(blocker.memory.buffer, pointer, length);
};

const write = (text) => encoder.encodeInto(text, exchange(text.length * MAX_UTF8_BYTES_PER_CHAR)).written;

const ready = fetch(BLOCKER)
  .then((response) => response.arrayBuffer())
  .then((module) => WebAssembly.instantiate(module))
  .then(({ instance }) => {
    blocker = instance.exports;
  });

const load = (snapshot) => {
  const { written } = exchange(Math.ceil(snapshot.length * BASE64_BYTES_PER_CHAR)).setFromBase64(snapshot);
  blocker.load_snapshot(written);
};

const active = () => blocking && blocker !== undefined;

const blocks = ({ url, documentUrl, originUrl, type, method }) => {
  const request = [url, documentUrl ?? originUrl ?? "", type, method].join(FIELD_SEPARATOR);
  return blocker.should_block(write(request)) !== 0;
};

const cosmeticCss = (url) => decoder.decode(exchange(blocker.cosmetic_css(write(url))));

const hide = ({ tabId, frameId, url }) => {
  if (!active() || !WEB_PAGE.test(url)) return;
  const code = cosmeticCss(url);
  if (code) browser.tabs.insertCSS(tabId, { code, frameId, cssOrigin: "user", runAt: "document_start" });
};

const inject = (code, { tabId, frameId, url }) => {
  if (code && WEB_PAGE.test(url)) browser.tabs.executeScript(tabId, { code, frameId, runAt: "document_start" });
};

const refocus = (tabs) => {
  tabs.filter(({ url }) => WEB_PAGE.test(url)).forEach(({ id }) => browser.tabs.executeScript(id, { code: focus.update }));
};

const port = browser.runtime.connectNative(NATIVE_APP);
port.onMessage.addListener((message) => {
  switch (message.kind) {
    case "blocking":
      blocking = message.enabled;
      return;
    case "focus":
      focus = message;
      if (focus.update) browser.tabs.query({}).then(refocus);
      return;
    case "sponsorblock":
      sponsorBlock = message.script;
      return;
    case "snapshot":
      ready.then(() => load(message.snapshot));
      return;
  }
});

browser.webRequest.onBeforeRequest.addListener(
  (details) => ({ cancel: details.type !== "main_frame" && active() && blocks(details) }),
  { urls: ["<all_urls>"] },
  ["blocking"],
);

browser.webNavigation.onCommitted.addListener(hide);
browser.webNavigation.onCommitted.addListener((details) => inject(focus.script, details));
browser.webNavigation.onCommitted.addListener((details) => inject(sponsorBlock, details));
