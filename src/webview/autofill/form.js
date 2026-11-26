const form = (() => {
  const USERNAME_HINTS = /user|email|login|account|identifier/i;
  const TEXT_TYPES = ["text", "email", "tel", ""];
  const SUBMIT_LABELS = /log ?in|sign ?in|continue|next|submit|create|register|sign ?up/i;

  const visible = (input) =>
    input.getClientRects().length > 0 && input.ownerDocument.defaultView.getComputedStyle(input).visibility !== "hidden";
  const editable = (input) => !input.disabled && !input.readOnly && visible(input);
  const fields = (root) => [...root.querySelectorAll("input")].filter(editable);
  const password = (input) => input.type === "password";
  const textual = (input) => TEXT_TYPES.includes(input.getAttribute("type")?.toLowerCase() ?? "");
  const hinted = (input) =>
    input.autocomplete === "username" ||
    input.type === "email" ||
    USERNAME_HINTS.test(`${input.name} ${input.id} ${input.placeholder} ${input.getAttribute("aria-label") ?? ""}`);
  const scope = (element) => element.closest("form") ?? element.ownerDocument;

  const passwords = (root) => fields(root).filter(password);
  const username = (root) => {
    const inputs = fields(root);
    const first = inputs.find(password);
    const before = (input) => !first || input.compareDocumentPosition(first) & Node.DOCUMENT_POSITION_FOLLOWING;
    const candidates = inputs.filter((input) => textual(input) && before(input));
    return candidates.find(hinted) ?? (first ? candidates.at(-1) : undefined);
  };
  const role = (element) => {
    if (!(element instanceof HTMLInputElement) || !editable(element)) return null;
    if (password(element)) return element.autocomplete === "new-password" ? "new_password" : "password";
    const root = scope(element);
    const signIn = passwords(root).length > 0 || element.autocomplete === "username";
    return signIn && username(root) === element ? "username" : null;
  };
  const submitter = (element) => {
    const control = element instanceof Element ? element.closest("button, input, [role=button]") : null;
    if (!control) return null;
    const submits = control.type === "submit" || SUBMIT_LABELS.test(control.textContent || control.value || "");
    return submits ? control : null;
  };

  return { fields, passwords, username, role, scope, submitter };
})();
