(() => {
  const { send, element, icon, iconButton, tile, matches, onState } = magpie;
  const search = document.getElementById("search");
  const spacesView = document.getElementById("spaces");
  const content = document.getElementById("content");
  const STEPS = { ArrowUp: "back", ArrowDown: "forward" };
  const DROP_CLASSES = ["drop-before", "drop-after", "drop-into"];
  const expanded = new Map();
  let spaces = [];
  let selected = null;
  let editing = null;
  let dragging = null;

  const currentSpace = () => spaces.find((space) => space.id === selected) || spaces[0];

  const removeFavorite = (space, url) => {
    space.favorites = space.favorites.filter((favorite) => favorite.url !== url);
    send("remove_favorite", { space: space.id, url });
    draw();
  };

  const withoutTab = (items, id) =>
    items
      .filter((item) => !(item.kind === "tab" && item.id === id))
      .map((item) => (item.kind === "folder" ? { ...item, items: withoutTab(item.items, id) } : item));

  const removePinned = (space, id) => {
    space.pinned = withoutTab(space.pinned, id);
    send("remove_pinned", { tab: id });
    draw();
  };

  const moveFavorite = (space, url, slot) => send("move_favorite", { space: space.id, url, slot: Math.max(0, slot) });

  const reference = (item) => ({ kind: item.kind, id: item.id });

  const movePinned = (item, spot) => send("move_pinned", { item, spot });

  const startRename = (renamer) => {
    editing = { ...renamer, draft: renamer.current, input: null, fresh: true };
    draw();
  };

  const finishRename = () => {
    const { save, draft, url } = editing;
    editing = null;
    save(draft, url);
    draw();
  };

  const cancelRename = () => {
    editing = null;
    draw();
  };

  const renameKey = (event) => {
    event.stopPropagation();
    if (event.key === "Enter") finishRename();
    if (event.key === "Escape") cancelRename();
  };

  const editField = (label, value, store) => {
    const input = element("input", {
      className: "rename",
      attributes: { "aria-label": label, spellcheck: "false" },
      on: {
        input: () => store(input.value),
        keydown: renameKey,
        click: (event) => event.stopPropagation(),
      },
    });
    input.value = value;
    return input;
  };

  const leftEditor = (fields, event) => editing?.input === fields.firstChild && !fields.contains(event.relatedTarget);

  const renameFields = () => {
    const title = editField("Name", editing.draft, (draft) => (editing.draft = draft));
    const url = editing.url === undefined ? [] : [editField("URL", editing.url, (url) => (editing.url = url))];
    const fields = element("span", { className: "edit-fields", on: { focusout: (event) => leftEditor(fields, event) && finishRename() } }, [title, ...url]);
    editing.input = title;
    return fields;
  };

  const name = (renamer) => (editing?.key === renamer.key ? renameFields() : element("span", { className: "label", text: renamer.current }));

  const rowKeys =
    ({ open, rename, moves }) =>
    (event) => {
      if (event.target !== event.currentTarget) return;
      const step = event.altKey && moves[STEPS[event.key]];
      if (step) {
        event.preventDefault();
        step();
        return;
      }
      if (event.key === "F2") {
        event.preventDefault();
        rename();
        return;
      }
      if (event.key === "Enter") open();
    };

  const place = (event, node, into) => {
    const box = node.getBoundingClientRect();
    const horizontal = node.classList.contains("favorite");
    const ratio = horizontal ? (event.clientX - box.left) / box.width : (event.clientY - box.top) / box.height;
    if (into && ratio > 1 / 3 && ratio < 2 / 3) return "into";
    return ratio < 0.5 ? "before" : "after";
  };

  const clearDrop = (node) => node.classList.remove(...DROP_CLASSES);

  const draggable = (node, carried) => {
    node.draggable = true;
    node.addEventListener("dragstart", (event) => {
      dragging = carried;
      event.dataTransfer.effectAllowed = "move";
      event.stopPropagation();
    });
    node.addEventListener("dragend", () => {
      dragging = null;
    });
  };

  const dropTarget = (node, { kind, into, drop }) => {
    node.addEventListener("dragover", (event) => {
      if (dragging?.kind !== kind) return;
      event.preventDefault();
      clearDrop(node);
      node.classList.add(`drop-${place(event, node, into)}`);
    });
    node.addEventListener("dragleave", () => clearDrop(node));
    node.addEventListener("drop", (event) => {
      event.preventDefault();
      clearDrop(node);
      drop(place(event, node, into));
    });
  };

  const row = ({ tag = "li", className, title, lead, renamer, detail, buttons, trail = [], open, moves, carried, drop }) => {
    const rename = () => startRename(renamer);
    const node = element(
      tag,
      {
        className,
        title,
        attributes: { tabindex: "0", "aria-keyshortcuts": "F2 Alt+ArrowUp Alt+ArrowDown" },
        on: { click: open, keydown: rowKeys({ open, rename, moves }) },
      },
      [
        lead,
        element("span", { className: "text" }, [name(renamer), element("span", { className: "detail", text: detail })]),
        element("span", { className: "actions" }, [iconButton("rename", renamer.url === undefined ? "Rename" : "Edit bookmark", rename), ...buttons]),
        ...trail,
      ],
    );
    node.dataset.key = renamer.key;
    if (editing?.key !== renamer.key) draggable(node, carried);
    dropTarget(node, drop);
    return node;
  };

  const moveButtons = ({ back, forward }) => [iconButton("up", "Move up", back), iconButton("down", "Move down", forward)];

  const favorite = (space, link) => {
    const index = space.favorites.indexOf(link);
    const last = space.favorites.length - 1;
    return row({
      className: "row favorite",
      title: link.url,
      lead: tile(link.host),
      renamer: {
        key: `favorite:${link.url}`,
        current: link.title || link.host,
        url: link.url,
        save: (title, url) => send("edit_favorite", { space: space.id, url: link.url, edited: { title, url } }),
      },
      detail: link.host,
      buttons: [iconButton("close", "Remove from Favorites", () => removeFavorite(space, link.url))],
      open: () => send("open_url", { url: link.url }),
      moves: {
        back: index > 0 ? () => moveFavorite(space, link.url, index - 1) : null,
        forward: index < last ? () => moveFavorite(space, link.url, index + 2) : null,
      },
      carried: { kind: "favorite", url: link.url },
      drop: { kind: "favorite", into: false, drop: (where) => moveFavorite(space, dragging.url, where === "after" ? index + 1 : index) },
    });
  };

  const tabMatches = (item) => matches(search.value, item.title, item.url);

  const visible = (items) =>
    items
      .map((item) => (item.kind === "folder" ? { ...item, items: visible(item.items) } : item))
      .filter((item) => (item.kind === "folder" ? item.items.length > 0 || matches(search.value, item.name) : tabMatches(item)));

  const folderOpen = (folder) => (search.value.trim() ? true : expanded.get(folder.id) ?? !folder.collapsed);

  const toggle = (folder) => {
    expanded.set(folder.id, !folderOpen(folder));
    draw();
  };

  const count = (items) => (items === 1 ? "1 item" : `${items} items`);

  const siblingMoves = (siblings, index) => {
    const item = reference(siblings[index]);
    return {
      back: index > 0 ? () => movePinned(item, { before: reference(siblings[index - 1]) }) : null,
      forward: index < siblings.length - 1 ? () => movePinned(item, { after: reference(siblings[index + 1]) }) : null,
    };
  };

  const pinnedDrop = (item, into) => ({
    kind: "pinned",
    into,
    drop: (where) => movePinned(dragging.item, where === "into" ? { into: item.id } : { [where]: reference(item) }),
  });

  const pinnedTab = (space, item, moves) =>
    row({
      className: "row",
      title: item.url,
      lead: tile(item.host),
      renamer: { key: `tab:${item.id}`, current: item.title || item.host, save: (title) => send("rename_tab", { tab: item.id, title }) },
      detail: item.host,
      buttons: [...moveButtons(moves), iconButton("close", "Unpin tab", () => removePinned(space, item.id))],
      open: () => send("focus_tab", { tab: item.id }),
      moves,
      carried: { kind: "pinned", item: reference(item) },
      drop: pinnedDrop(item, false),
    });

  const folderHeader = (item, moves) => {
    const header = row({
      tag: "div",
      className: "row",
      lead: element("span", { className: "tile" }, [icon("folder")]),
      renamer: { key: `folder:${item.id}`, current: item.name, save: (name) => send("rename_folder", { folder: item.id, name }) },
      detail: count(item.items.length),
      buttons: moveButtons(moves),
      trail: [element("span", { className: "chevron" }, [icon("chevron")])],
      open: () => toggle(item),
      moves,
      carried: { kind: "pinned", item: reference(item) },
      drop: pinnedDrop(item, true),
    });
    header.setAttribute("role", "button");
    header.setAttribute("aria-expanded", String(folderOpen(item)));
    return header;
  };

  const pinnedList = (space, items, className) => element("ul", { className }, items.map((item, index) => pinnedItem(space, items, index)));

  const pinnedItem = (space, siblings, index) => {
    const item = siblings[index];
    const moves = siblingMoves(siblings, index);
    if (item.kind === "tab") return pinnedTab(space, item, moves);
    return element("li", { className: folderOpen(item) ? "folder" : "folder collapsed" }, [folderHeader(item, moves), pinnedList(space, item.items, "")]);
  };

  const block = (title, children, note) =>
    element("section", {}, [element("h2", { className: "section-title", text: title }), ...(children.length ? children : [element("p", { className: "list note", text: note })])]);

  const spaceButton = (space) =>
    element("button", {
      className: "space",
      text: space.name,
      attributes: { type: "button", "aria-current": String(space === currentSpace()) },
      on: {
        click: () => {
          selected = space.id;
          draw();
        },
      },
    });

  const focusedKey = () => document.activeElement?.closest("[data-key]")?.dataset.key;

  const restoreFocus = (key) => {
    if (editing) {
      editing.input.focus();
      if (editing.fresh) editing.input.select();
      editing.fresh = false;
      return;
    }
    [...content.querySelectorAll("[data-key]")].find((node) => node.dataset.key === key)?.focus();
  };

  const sections = (space) => {
    const favorites = space.favorites.filter((link) => matches(search.value, link.title, link.url));
    const pinned = visible(space.pinned);
    return [
      block("Favorites", favorites.length ? [element("ul", { className: "favorites" }, favorites.map((link) => favorite(space, link)))] : [], "No Favorites in this Space."),
      block("Pinned Tabs & Folders", pinned.length ? [pinnedList(space, pinned, "list")] : [], "No pinned tabs in this Space."),
    ];
  };

  const draw = () => {
    const space = currentSpace();
    const focused = focusedKey();
    if (editing) editing.input = null;
    spacesView.replaceChildren(...spaces.map(spaceButton));
    content.replaceChildren(...(space ? sections(space) : [element("p", { className: "empty", text: "No Spaces yet." })]));
    if (editing && !editing.input) editing = null;
    restoreFocus(focused);
  };

  document.getElementById("import").addEventListener("click", (event) => {
    event.preventDefault();
    send("open_settings_section", { section: "import" });
  });
  search.addEventListener("input", draw);

  onState((payload) => {
    spaces = payload.spaces;
    selected = spaces.some((space) => space.id === selected) ? selected : payload.active;
    draw();
  });
  draw();
})();
