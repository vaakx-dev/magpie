const magpie = (() => {
  const root = document.documentElement;
  const bridge = window.chrome && window.chrome.webview;
  const page = root.dataset.page;
  const SVG = "http://www.w3.org/2000/svg";
  const PATHS = {
    close: "M3 3l8 8M11 3l-8 8",
    rename: "M9.5 2.5l2 2-7 7h-2v-2z",
    up: "M7 11.5v-9M3.5 6L7 2.5 10.5 6",
    down: "M7 2.5v9M3.5 8L7 11.5 10.5 8",
    folder: "M1.5 3.5h4l1.5 1.5h5.5v6h-11z",
    chevron: "M4 5.5l3 3 3-3",
  };
  let render = null;
  let latest = null;

  const receive = ({ scheme, state }) => {
    root.dataset.scheme = scheme;
    latest = state;
    if (render) render(state);
  };

  const send = (action, fields = {}) => {
    if (bridge) bridge.postMessage({ kind: "magpie_page", page, action, ...fields });
  };

  const element = (tag, { className, text, title, attributes = {}, on = {} } = {}, children = []) => {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    if (title) node.title = title;
    Object.entries(attributes).forEach(([name, value]) => node.setAttribute(name, value));
    Object.entries(on).forEach(([event, listener]) => node.addEventListener(event, listener));
    node.append(...children);
    return node;
  };

  const icon = (name) => {
    const svg = document.createElementNS(SVG, "svg");
    svg.setAttribute("viewBox", "0 0 14 14");
    const line = document.createElementNS(SVG, "path");
    line.setAttribute("d", PATHS[name]);
    line.setAttribute("stroke", "currentColor");
    line.setAttribute("stroke-width", "1.5");
    line.setAttribute("stroke-linecap", "round");
    line.setAttribute("fill", "none");
    svg.append(line);
    return svg;
  };

  const iconButton = (name, label, run) =>
    element(
      "button",
      {
        className: "icon-button",
        title: label,
        attributes: { type: "button", "aria-label": label, ...(run ? {} : { disabled: "" }) },
        on: {
          click: (event) => {
            event.stopPropagation();
            run();
          },
        },
      },
      [icon(name)],
    );

  const tile = (host) => element("span", { className: "tile", text: (host || "?").replace(/^www\./, "").charAt(0) });

  const matches = (query, ...fields) => {
    const needle = query.trim().toLowerCase();
    return !needle || fields.some((field) => field.toLowerCase().includes(needle));
  };

  const onState = (callback) => {
    render = callback;
    if (latest) render(latest);
  };

  if (bridge) bridge.addEventListener("message", ({ data }) => receive(data));
  send("ready");

  return { send, element, icon, iconButton, tile, matches, onState };
})();
