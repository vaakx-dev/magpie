(() => {
  const { send, element, tile, matches, onState } = magpie;
  const search = document.getElementById("search");
  const list = document.getElementById("downloads");
  const empty = document.getElementById("empty");
  const pills = [...document.querySelectorAll(".pill")];
  const EMPTY_TEXT = {
    all: "Files you download will show up here.",
    active: "Nothing is downloading right now.",
    completed: "No finished downloads yet.",
  };
  const FINISHED_STATUS = { completed: "", failed: "Failed", cancelled: "Cancelled" };
  const UNFINISHED = new Set(["active", "paused"]);
  let downloads = [];
  let filter = "all";

  const button = (label, action, path) =>
    element("button", { className: "button", text: label, on: { click: () => send(action, { path }) } });

  const progress = (fraction) => {
    const known = fraction !== null;
    const bar = element("span", {});
    if (known) bar.style.width = `${Math.round(fraction * 100)}%`;
    return element("div", { className: known ? "progress" : "progress unknown" }, [bar]);
  };

  const activeDetails = (download) => [
    progress(download.progress),
    element("span", { className: "detail", text: download.readout }),
  ];

  const finishedDetails = (download) => {
    const status = FINISHED_STATUS[download.status];
    const facts = [status || download.size, download.url].join(" · ");
    return [element("span", { className: status ? "detail failed" : "detail", text: facts, title: download.url })];
  };

  const unfinished = (download) => UNFINISHED.has(download.status);

  const actions = (download) => {
    const cancel = button("Cancel", "cancel_download", download.path);
    if (download.status === "active" && download.pausable) return [button("Pause", "pause_download", download.path), cancel];
    if (download.status === "active") return [cancel];
    if (download.status === "paused") return [button("Resume", "resume_download", download.path), cancel];
    if (download.status === "completed")
      return [button("Open", "open_download", download.path), button("Show in folder", "show_download_in_folder", download.path)];
    return [];
  };

  const row = (download) =>
    element("li", { className: "row download", attributes: { "data-status": download.status } }, [
      tile(download.name),
      element("span", { className: "text" }, [
        element("span", { className: "label", text: download.name, title: download.path }),
        ...(unfinished(download) ? activeDetails(download) : finishedDetails(download)),
      ]),
      element("span", { className: "actions" }, actions(download)),
    ]);

  const shown = () =>
    downloads
      .filter((download) => filter === "all" || (filter === "active") === unfinished(download))
      .filter((download) => matches(search.value, download.name, download.url));

  const draw = () => {
    const visible = shown();
    list.replaceChildren(...(visible.length ? [element("ul", { className: "list" }, visible.map(row))] : []));
    empty.hidden = visible.length > 0;
    empty.textContent = search.value.trim() ? "No downloads match your search." : EMPTY_TEXT[filter];
  };

  const choose = (pill) => {
    filter = pill.dataset.filter;
    pills.forEach((other) => other.setAttribute("aria-pressed", String(other === pill)));
    draw();
  };

  pills.forEach((pill) => pill.addEventListener("click", () => choose(pill)));
  search.addEventListener("input", draw);
  document.getElementById("clear").addEventListener("click", () => {
    downloads = downloads.filter(unfinished);
    send("clear_downloads");
    draw();
  });

  onState((payload) => {
    downloads = payload.downloads;
    draw();
  });
  draw();
})();
