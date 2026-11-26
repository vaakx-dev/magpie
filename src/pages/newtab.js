(() => {
  const { send, element, tile, onState } = magpie;
  const search = document.getElementById("search");
  const shortcuts = document.getElementById("shortcuts");

  const shortcut = ({ url, title, host }) =>
    element("li", { className: "shortcut", title: url, on: { click: () => send("open_url", { url }) } }, [
      tile(host),
      element("span", { className: "label", text: title || host }),
    ]);

  document.getElementById("query").addEventListener("submit", (event) => {
    event.preventDefault();
    const text = search.value.trim();
    if (text) send("navigate", { text });
  });

  onState((payload) => {
    document.title = payload.space ? `New Tab · ${payload.space}` : "New Tab";
    shortcuts.replaceChildren(...payload.favorites.map(shortcut));
  });
})();
