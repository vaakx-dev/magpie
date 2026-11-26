(() => {
  const { send, element, iconButton, tile, matches, onState } = magpie;
  const search = document.getElementById("search");
  const groupsView = document.getElementById("groups");
  const empty = document.getElementById("empty");
  const clear = document.getElementById("clear");
  const confirm = document.getElementById("confirm");
  let groups = [];

  const time = (visitedSecs) =>
    new Date(visitedSecs * 1000).toLocaleString([], { month: "short", day: "numeric", hour: "numeric", minute: "2-digit" });

  const remove = (url) => {
    groups = groups.map((group) => ({ ...group, visits: group.visits.filter((visit) => visit.url !== url) }));
    send("delete_history", { url });
    draw();
  };

  const row = (visit) =>
    element("li", { className: "row", title: visit.url, on: { click: () => send("open_url", { url: visit.url }) } }, [
      tile(visit.host),
      element("span", { className: "text" }, [
        element("span", { className: "label", text: visit.title || visit.host }),
        element("span", { className: "detail", text: visit.url }),
      ]),
      element("span", { className: "meta", text: time(visit.visited) }),
      element("span", { className: "actions" }, [iconButton("close", "Delete from history", () => remove(visit.url))]),
    ]);

  const section = ({ period, visits }) =>
    element("section", {}, [
      element("h2", { className: "section-title", text: period }),
      element("ul", { className: "list" }, visits.map(row)),
    ]);

  const filtered = (query) =>
    groups
      .map((group) => ({ ...group, visits: group.visits.filter((visit) => matches(query, visit.title, visit.url)) }))
      .filter((group) => group.visits.length > 0);

  const draw = () => {
    const query = search.value;
    const shown = filtered(query);
    groupsView.replaceChildren(...shown.map(section));
    empty.hidden = shown.length > 0;
    empty.textContent = query.trim() ? "No history matches your search." : "Pages you visit will show up here.";
  };

  const askToClear = (asking) => {
    clear.hidden = asking;
    confirm.hidden = !asking;
  };

  clear.addEventListener("click", () => askToClear(true));
  document.getElementById("cancel-clear").addEventListener("click", () => askToClear(false));
  document.getElementById("confirm-clear").addEventListener("click", () => {
    askToClear(false);
    groups = [];
    send("clear_history");
    draw();
  });
  search.addEventListener("input", draw);

  onState((payload) => {
    groups = payload.groups;
    draw();
  });
  draw();
})();
