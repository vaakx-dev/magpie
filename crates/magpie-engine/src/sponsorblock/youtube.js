(connect) => {
  const youtube = /(^|\.)youtube\.com$/.test(location.hostname);
  if (window !== window.top || !youtube) return;

  const REQUEST = "magpie_sponsorblock_request";
  const REPLY = "magpie_sponsorblock_segments";
  const PLAYERS = "#movie_player, #shorts-player";
  const NOTICE_MS = 4000;
  const MARKER_CLASS = "magpie-sponsorblock-marker";
  const state = { href: "", request: 0, skips: [], notice: true, unskipped: new Set() };
  const toast = { element: null, timer: 0 };

  const playerVideo = () => document.querySelector("#movie_player video, #shorts-player video");
  const pageSide = (object) => object?.wrappedJSObject ?? object;
  const details = (event) =>
    pageSide(event)?.detail?.response?.playerResponse?.videoDetails ??
    pageSide(document.getElementById("movie_player"))?.getPlayerResponse?.()?.videoDetails ??
    pageSide(window).ytInitialPlayerResponse?.videoDetails;

  const clearMarkers = () => document.querySelectorAll("." + MARKER_CLASS).forEach((marker) => marker.remove());
  const marker = (skip, duration) => {
    const element = document.createElement("div");
    element.className = MARKER_CLASS;
    Object.assign(element.style, {
      position: "absolute",
      top: "0",
      height: "100%",
      left: (skip.start / duration) * 100 + "%",
      width: ((skip.end - skip.start) / duration) * 100 + "%",
      background: skip.color,
      opacity: "0.6",
      pointerEvents: "none",
      zIndex: "40",
    });
    return element;
  };
  const drawMarkers = () => {
    clearMarkers();
    const video = playerVideo();
    const bar = video?.closest(PLAYERS)?.querySelector(".ytp-progress-bar");
    const playingAd = video?.closest(".ad-showing");
    if (!bar || playingAd || !(video.duration > 0)) return;
    state.skips.forEach((skip) => bar.appendChild(marker(skip, video.duration)));
  };

  const hideToast = () => {
    clearTimeout(toast.timer);
    toast.element?.remove();
    toast.element = null;
  };
  const toastElement = (skip, onUnskip) => {
    const element = document.createElement("div");
    Object.assign(element.style, {
      position: "absolute",
      right: "16px",
      bottom: "72px",
      zIndex: "100",
      display: "flex",
      gap: "8px",
      alignItems: "center",
      padding: "8px 12px",
      borderRadius: "8px",
      background: "rgba(20, 20, 20, 0.85)",
      color: "#fff",
      font: "500 13px/1.2 Roboto, Arial, sans-serif",
      boxShadow: "0 2px 8px rgba(0, 0, 0, 0.35)",
    });
    const text = document.createElement("span");
    text.textContent = "Skipped " + skip.label + " \u00b7";
    const unskip = document.createElement("button");
    unskip.textContent = "Unskip";
    Object.assign(unskip.style, {
      background: "none",
      border: "none",
      padding: "0",
      color: "#3ea6ff",
      font: "inherit",
      cursor: "pointer",
    });
    unskip.addEventListener("click", onUnskip);
    element.append(text, unskip);
    return element;
  };
  const showToast = (video, skip, onUnskip) => {
    hideToast();
    toast.element = toastElement(skip, () => {
      hideToast();
      onUnskip();
    });
    video.closest(PLAYERS).appendChild(toast.element);
    toast.timer = setTimeout(hideToast, NOTICE_MS);
  };

  const activeSkip = (time) =>
    state.skips.findIndex((skip, index) => !state.unskipped.has(index) && time >= skip.start && time < skip.end);
  const skipAhead = (event) => {
    if (location.href !== state.href) request();
    const video = event.target;
    if (!(video instanceof HTMLVideoElement) || !video.closest(PLAYERS) || video.closest(".ad-showing")) return;
    const index = activeSkip(video.currentTime);
    if (index < 0) return;
    const skip = state.skips[index];
    video.currentTime = skip.end;
    if (!state.notice) return;
    showToast(video, skip, () => {
      state.unskipped.add(index);
      video.currentTime = skip.start;
    });
  };

  const reset = () => {
    state.href = location.href;
    state.skips = [];
    state.unskipped = new Set();
    clearMarkers();
    hideToast();
  };
  const request = (event) => {
    if (location.href !== state.href) reset();
    const video = details(event);
    state.request += 1;
    send({
      kind: REQUEST,
      request: state.request,
      address: location.href,
      channel_id: video?.channelId ?? null,
      channel_name: video?.author ?? null,
    });
  };
  const receive = (reply) => {
    if (reply?.kind !== REPLY || reply.request !== state.request) return;
    state.skips = reply.skips;
    state.notice = reply.notice;
    drawMarkers();
  };

  const send = connect(receive);
  document.addEventListener("timeupdate", skipAhead, true);
  document.addEventListener("durationchange", drawMarkers, true);
  document.addEventListener("yt-navigate-finish", request);
  window.navigation?.addEventListener("navigatesuccess", () => request());
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => request(), { once: true });
  } else {
    request();
  }
}
