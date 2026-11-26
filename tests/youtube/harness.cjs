const fs = require("node:fs");
const vm = require("node:vm");

const [scriptPath, hostname] = process.argv.slice(2);
const AD_FIELDS = {
  adPlacements: [{ adPlacementRenderer: {} }],
  playerAds: [{ playerLegacyDesktopWatchAdsRenderer: {} }],
  adSlots: [{ adSlotRenderer: {} }],
  adBreakHeartbeatParams: "heartbeat",
};
const player = () => ({ videoDetails: { videoId: "abc" }, ...AD_FIELDS });
const next = () => ({ contents: {}, playerResponse: player() });
const watch = () => [{ playerResponse: player() }, { response: next() }];
const served = (url) => {
  if (url.includes("/get_watch")) return watch();
  if (url.includes("/next")) return next();
  return player();
};

class FakeRequest {
  open(method, url) {
    this.url = url;
  }
  send() {
    this.readyState = 4;
    this.responseType = "";
    this.body = JSON.stringify(served(this.url));
  }
  get responseText() {
    return this.body;
  }
  get response() {
    return this.body;
  }
}
FakeRequest.DONE = 4;

Object.assign(globalThis, {
  window: globalThis,
  location: { hostname },
  document: { adoptedStyleSheets: [], addEventListener() {}, querySelector: () => null },
  CSSStyleSheet: class {
    replaceSync() {}
  },
  MutationObserver: class {
    observe() {}
  },
  Element: class {},
  HTMLVideoElement: class {},
  XMLHttpRequest: FakeRequest,
  fetch: async (url) => new Response(JSON.stringify(served(url))),
});

vm.runInThisContext(fs.readFileSync(scriptPath, "utf8"));
vm.runInThisContext(`var ytInitialPlayerResponse = ${JSON.stringify(player())};`);

const requested = (url) => {
  const request = new XMLHttpRequest();
  request.open("POST", url);
  request.send();
  return JSON.parse(request.responseText);
};

(async () => {
  const fetched = await (await fetch("/youtubei/v1/player?prettyPrint=false")).json();
  const browsed = await (await fetch("/youtubei/v1/browse")).json();
  const navigated = await (await fetch("/youtubei/v1/get_watch?prettyPrint=false")).json();
  const watched = requested("/youtubei/v1/next?prettyPrint=false");
  console.log(
    JSON.stringify({
      initial: globalThis.ytInitialPlayerResponse,
      fetched,
      browsed,
      watched,
      navigated,
    }),
  );
})();
