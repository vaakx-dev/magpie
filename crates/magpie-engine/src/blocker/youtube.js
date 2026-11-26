(() => {
  if (!/(^|\.)youtube\.com$/.test(location.hostname)) return;

  const AD_FIELDS = ["adPlacements", "playerAds", "adSlots", "adBreakHeartbeatParams"];
  const PRUNED_ENDPOINTS = /\/youtubei\/v1\/(player|next|get_watch|reel\/reel_item_watch)([?#]|$)/;
  const SKIP_BUTTONS = ".ytp-skip-ad-button, .ytp-ad-skip-button-modern, .ytp-ad-skip-button";
  const ENFORCEMENT = "ytd-enforcement-message-view-model";
  const HIDDEN = [
    "ytd-ad-slot-renderer",
    "ytd-in-feed-ad-layout-renderer",
    "ytd-promoted-sparkles-web-renderer",
    "ytd-promoted-video-renderer",
    "ytd-banner-promo-renderer",
    "ytd-rich-item-renderer:has(> #content > ytd-ad-slot-renderer)",
    "#masthead-ad",
    "#player-ads",
    'ytd-engagement-panel-section-list-renderer[target-id="engagement-panel-ads"]',
    `tp-yt-paper-dialog:has(${ENFORCEMENT})`,
    `body:has(tp-yt-paper-dialog ${ENFORCEMENT}) tp-yt-iron-overlay-backdrop`,
  ];
  const MEDIA_EVENTS = ["loadedmetadata", "playing", "timeupdate"];

  const prune = (value) => {
    if (value === null || typeof value !== "object") return value;
    AD_FIELDS.forEach((field) => delete value[field]);
    Object.values(value).forEach(prune);
    return value;
  };
  const mentionsAds = (text) => AD_FIELDS.some((field) => text.includes(`"${field}"`));
  const pruneText = (text) => {
    if (!mentionsAds(text)) return text;
    try {
      return JSON.stringify(prune(JSON.parse(text)));
    } catch {
      return text;
    }
  };
  const pruned = (url) => PRUNED_ENDPOINTS.test(String(url));

  const trapInitialResponse = () => {
    let initial;
    Object.defineProperty(window, "ytInitialPlayerResponse", {
      configurable: true,
      get: () => initial,
      set: (value) => {
        initial = prune(value);
      },
    });
  };

  const prunedResponse = async (response) => {
    const text = await response.clone().text();
    const body = pruneText(text);
    if (body === text) return response;
    const { status, statusText, headers } = response;
    const replacement = new Response(body, { status, statusText, headers });
    Object.defineProperties(replacement, {
      url: { value: response.url },
      redirected: { value: response.redirected },
      type: { value: response.type },
    });
    return replacement;
  };
  const patchFetch = () => {
    window.fetch = new Proxy(window.fetch, {
      apply: async (fetch, self, args) => {
        const [resource] = args;
        const response = await Reflect.apply(fetch, self, args);
        return pruned(resource instanceof Request ? resource.url : resource)
          ? prunedResponse(response)
          : response;
      },
    });
  };

  const patchRequest = () => {
    window.XMLHttpRequest = class extends window.XMLHttpRequest {
      #prunes = false;
      #text;

      open(method, url, ...rest) {
        this.#prunes = pruned(url);
        return super.open(method, url, ...rest);
      }

      #done() {
        return this.#prunes && this.readyState === XMLHttpRequest.DONE;
      }

      get responseText() {
        if (!this.#done()) return super.responseText;
        this.#text ??= pruneText(super.responseText);
        return this.#text;
      }

      get response() {
        if (!this.#done()) return super.response;
        if (this.responseType === "json") return prune(super.response);
        if (this.responseType === "" || this.responseType === "text") return this.responseText;
        return super.response;
      }
    };
  };

  const hideSlots = () => {
    const sheet = new CSSStyleSheet();
    sheet.replaceSync(`${HIDDEN.join(",\n")} { display: none !important; }`);
    document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
  };

  const silenced = new WeakSet();
  const showsAd = (video) =>
    video.closest(".html5-video-player")?.classList.contains("ad-showing") ?? false;
  const skipAd = (video) => {
    if (!video.muted) {
      video.muted = true;
      silenced.add(video);
    }
    if (Number.isFinite(video.duration)) video.currentTime = video.duration;
    document.querySelector(SKIP_BUTTONS)?.click();
  };
  const restoreSound = (video) => {
    if (silenced.delete(video)) video.muted = false;
  };
  const watchVideo = (event) => {
    const video = event.target;
    if (!(video instanceof HTMLVideoElement)) return;
    if (showsAd(video)) skipAd(video);
    else restoreSound(video);
  };
  const skipPlayingAds = () =>
    MEDIA_EVENTS.forEach((type) => document.addEventListener(type, watchVideo, true));

  const enforcement = (node) =>
    node.localName === ENFORCEMENT ? node : node.getElementsByTagName(ENFORCEMENT)[0];
  const dismiss = (message) => {
    message.closest("tp-yt-paper-dialog")?.close();
    document.querySelector("video")?.play().catch(() => {});
  };
  const dismissEnforcement = () => {
    new MutationObserver((records) => {
      records.forEach((record) =>
        record.addedNodes.forEach((node) => {
          const message = node instanceof Element && enforcement(node);
          if (message) dismiss(message);
        }),
      );
    }).observe(document, { childList: true, subtree: true });
  };

  trapInitialResponse();
  patchFetch();
  patchRequest();
  hideSlots();
  skipPlayingAds();
  dismissEnforcement();
})();
