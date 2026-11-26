(() => {
  const bridge = window.chrome && window.chrome.webview;
  if (window !== window.top || !bridge) return;
  const NOTCH = 50;
  let travelled = 0;

  addEventListener(
    "wheel",
    (event) => {
      if (!event.ctrlKey) return;
      event.preventDefault();
      travelled += event.deltaY;
      if (Math.abs(travelled) < NOTCH) return;
      bridge.postMessage({ kind: "zoom", step: travelled < 0 ? "in" : "out" });
      travelled = 0;
    },
    { passive: false, capture: true },
  );
})();
