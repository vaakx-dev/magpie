(() => {
  const bridge = window.chrome && window.chrome.webview;
  if (window !== window.top || !bridge) return;
  const seen = { classes: new Set(), ids: new Set() };
  let fresh = { classes: [], ids: [] };
  let queued = false;
  const sheet = document.createElement("style");

  const learn = (names, list, name) => {
    if (name && !names.has(name)) {
      names.add(name);
      list.push(name);
    }
  };
  const note = (element) => {
    learn(seen.ids, fresh.ids, element.id);
    element.classList.forEach((name) => learn(seen.classes, fresh.classes, name));
  };
  const send = () => {
    queued = false;
    if (fresh.classes.length === 0 && fresh.ids.length === 0) return;
    bridge.postMessage({ kind: "names", classes: fresh.classes, ids: fresh.ids });
    fresh = { classes: [], ids: [] };
  };
  const queue = () => {
    if (queued) return;
    queued = true;
    setTimeout(send, 50);
  };
  const scan = (node) => {
    if (!(node instanceof Element)) return;
    note(node);
    node.querySelectorAll("[id],[class]").forEach(note);
    queue();
  };

  bridge.addEventListener("message", (event) => {
    if (typeof event.data !== "string") return;
    sheet.textContent += event.data;
    if (!sheet.isConnected) (document.head || document.documentElement).appendChild(sheet);
  });
  new MutationObserver((records) => {
    records.forEach((record) => {
      if (record.type === "attributes") {
        note(record.target);
        queue();
      } else {
        record.addedNodes.forEach(scan);
      }
    });
  }).observe(document, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ["class", "id"],
  });
  bridge.postMessage({ kind: "page" });
})();
