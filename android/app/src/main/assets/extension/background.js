const NATIVE_APP = "magpie";
const BLOCKER = "magpie_blocker.wasm";
const MAX_UTF8_BYTES_PER_CHAR = 3;
const BASE64_BYTES_PER_CHAR = 3 / 4;
const FIELD_SEPARATOR = "\n";
const WEB_PAGE = /^https?:/;
const WWW = /^www\./;
const TOP_FRAME = 0;
const STARTUP_TIMEOUT_MS = 1500;
const YOUTUBE_SITES = {
  "youtube.com": ["*://youtube.com/*", "*://www.youtube.com/*"],
  "m.youtube.com": ["*://m.youtube.com/*"],
};
const YOUTUBE_SCRIPT = {
  id: "youtube",
  js: ["youtube.js"],
  runAt: "document_start",
  world: "MAIN",
  persistAcrossSessions: false,
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();
let blocker;
let blocking = true;
let paused = new Set();
let privatePaused = new Set();
let focus = {};
let sponsorBlock;
let filtering = false;
let youtube = Promise.resolve();
const filters = Promise.withResolvers();
const configured = Promise.withResolvers();

const exchange = (length) => {
  const pointer = blocker.exchange(length);
  return new Uint8Array(blocker.memory.buffer, pointer, length);
};

const write = (text) => encoder.encodeInto(text, exchange(text.length * MAX_UTF8_BYTES_PER_CHAR)).written;

const instantiated = fetch(BLOCKER)
  .then((response) => response.arrayBuffer())
  .then((module) => WebAssembly.instantiate(module))
  .then(({ instance }) => {
    blocker = instance.exports;
  });

const load = (snapshot) => {
  const { written } = exchange(Math.ceil(snapshot.length * BASE64_BYTES_PER_CHAR)).setFromBase64(snapshot);
  if (blocker.load_snapshot(written) !== 0) filters.resolve();
};

const settled = (promise) => Promise.race([promise, new Promise((resolve) => setTimeout(resolve, STARTUP_TIMEOUT_MS))]);

const site = (url) => (WEB_PAGE.test(url) ? new URL(url).hostname.replace(WWW, "") : undefined);

const pausedIn = (incognito) => (incognito ? privatePaused : paused);

const active = (page, incognito) => blocking && blocker !== undefined && !pausedIn(incognito).has(site(page));

const requestPage = ({ frameAncestors, documentUrl, originUrl }) => frameAncestors?.at(-1)?.url ?? documentUrl ?? originUrl ?? "";

const framePage = ({ tabId, frameId, url }) =>
  frameId === TOP_FRAME
    ? Promise.resolve(url)
    : browser.webNavigation.getFrame({ tabId, frameId: TOP_FRAME }).then((frame) => frame?.url ?? url);

const blocks = ({ url, documentUrl, originUrl, type, method }) => {
  const request = [url, documentUrl ?? originUrl ?? "", type, method].join(FIELD_SEPARATOR);
  return blocker.should_block(write(request)) !== 0;
};

const cosmeticCss = (url) => decoder.decode(exchange(blocker.cosmetic_css(write(url))));

const hide = async (details) => {
  const { tabId, frameId, url } = details;
  if (!WEB_PAGE.test(url)) return;
  const [page, { incognito }] = await Promise.all([framePage(details), browser.tabs.get(tabId)]);
  if (!active(page, incognito)) return;
  const code = cosmeticCss(url);
  if (code) browser.tabs.insertCSS(tabId, { code, frameId, cssOrigin: "user", runAt: "document_start" });
};

const inject = (code, { tabId, frameId, url }) => {
  if (code && WEB_PAGE.test(url)) browser.tabs.executeScript(tabId, { code, frameId, runAt: "document_start" });
};

const youtubeMatches = () =>
  blocking
    ? Object.entries(YOUTUBE_SITES)
        .filter(([name]) => !paused.has(name))
        .flatMap(([, matches]) => matches)
    : [];

const registerYoutube = (registered, matches) => {
  if (registered && matches.length > 0) return browser.scripting.updateContentScripts([{ id: YOUTUBE_SCRIPT.id, matches }]);
  if (registered) return browser.scripting.unregisterContentScripts({ ids: [YOUTUBE_SCRIPT.id] });
  if (matches.length > 0) return browser.scripting.registerContentScripts([{ ...YOUTUBE_SCRIPT, matches }]);
  return Promise.resolve();
};

const youtubeRegistered = () =>
  browser.scripting.getRegisteredContentScripts({ ids: [YOUTUBE_SCRIPT.id] }).then((scripts) => scripts.length > 0);

const syncYoutube = () => {
  const matches = youtubeMatches();
  youtube = youtube
    .then(youtubeRegistered)
    .then((registered) => registerYoutube(registered, matches))
    .catch(console.error);
  return youtube;
};

const refocus = (tabs) => {
  tabs.filter(({ url }) => WEB_PAGE.test(url)).forEach(({ id }) => browser.tabs.executeScript(id, { code: focus.update }));
};

const port = browser.runtime.connectNative(NATIVE_APP);
port.onMessage.addListener((message) => {
  switch (message.kind) {
    case "blocking":
      blocking = message.enabled;
      paused = new Set(message.paused);
      privatePaused = new Set(message.privatePaused);
      if (!blocking) filters.resolve();
      syncYoutube().then(configured.resolve);
      return;
    case "focus":
      focus = message;
      if (focus.update) browser.tabs.query({}).then(refocus);
      return;
    case "sponsorblock":
      sponsorBlock = message.script;
      return;
    case "snapshot":
      instantiated.then(() => load(message.snapshot));
      return;
  }
});

filters.promise.then(() => {
  filtering = true;
});
Promise.all([filters.promise, configured.promise]).then(() => port.postMessage({ kind: "ready" }));

const decision = (details) => ({ cancel: active(requestPage(details), details.incognito) && blocks(details) });

const decide = (details) => (filtering ? decision(details) : settled(filters.promise).then(() => decision(details)));

browser.webRequest.onBeforeRequest.addListener(
  (details) => (details.type === "main_frame" ? undefined : decide(details)),
  { urls: ["<all_urls>"] },
  ["blocking"],
);

browser.webNavigation.onCommitted.addListener(hide);
browser.webNavigation.onCommitted.addListener((details) => inject(focus.script, details));
browser.webNavigation.onCommitted.addListener((details) => inject(sponsorBlock, details));
