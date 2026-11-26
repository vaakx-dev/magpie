(() => {
  const bridge = window.chrome && window.chrome.webview;
  if (window !== window.top || !bridge) return;
  const state = { field: null, frame: 0, scan: 0, found: false };

  const place = () => {
    state.frame = 0;
    if (!state.field) return;
    const role = form.role(state.field);
    if (!role) {
      leave();
      return;
    }
    const { x, y, width, height } = state.field.getBoundingClientRect();
    bridge.postMessage({ kind: "login_field", role, x, y, width, height });
  };
  const follow = () => {
    if (state.field && !state.frame) state.frame = requestAnimationFrame(place);
  };
  const leave = () => {
    if (!state.field) return;
    state.field = null;
    bridge.postMessage({ kind: "login_field_left" });
  };
  const announce = () => {
    if (state.found || form.passwords(document).length === 0) return;
    state.found = true;
    observer.disconnect();
    bridge.postMessage({ kind: "login_form" });
  };
  const scan = () => {
    state.scan = 0;
    announce();
  };
  const observer = new MutationObserver(() => {
    if (!state.scan) state.scan = requestAnimationFrame(scan);
  });

  const submitted = (root) => {
    const filled = form.passwords(root).filter((input) => input.value);
    if (filled.length === 0) return;
    const username = form.username(root)?.value ?? "";
    bridge.postMessage({ kind: "login_submitted", username, password: filled.at(-1).value });
  };

  addEventListener(
    "focusin",
    (event) => {
      if (!form.role(event.target)) {
        leave();
        return;
      }
      state.field = event.target;
      announce();
      place();
    },
    true,
  );
  addEventListener(
    "focusout",
    (event) => {
      if (event.target !== state.field || !document.hasFocus() || form.role(event.relatedTarget)) return;
      leave();
    },
    true,
  );
  addEventListener(
    "keydown",
    (event) => {
      if (event.key === "Escape") leave();
      if (event.key === "Enter" && form.role(event.target)) submitted(form.scope(event.target));
    },
    true,
  );
  addEventListener("submit", (event) => submitted(event.target), true);
  addEventListener(
    "click",
    (event) => {
      const control = form.submitter(event.target);
      if (control) submitted(form.scope(control));
    },
    true,
  );
  addEventListener("scroll", follow, { capture: true, passive: true });
  addEventListener("resize", follow, { passive: true });
  addEventListener("DOMContentLoaded", () => {
    announce();
    if (!state.found) observer.observe(document.documentElement, { childList: true, subtree: true });
  });
})();
