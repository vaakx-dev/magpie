(({ origin, username, password }) => {
  if (location.origin !== origin) {
    return false;
  }
  const USERNAME_HINTS = /user|email|login|account|identifier/i;
  const TEXT_TYPES = ["text", "email", "tel", ""];
  const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set;

  const visible = (input) =>
    input.getClientRects().length > 0 && input.ownerDocument.defaultView.getComputedStyle(input).visibility !== "hidden";
  const editable = (input) => !input.disabled && !input.readOnly && visible(input);
  const fields = (root) => [...root.querySelectorAll("input")].filter(editable);
  const documents = (root) => [
    root,
    ...[...root.querySelectorAll("iframe, frame")]
      .map((frame) => frame.contentDocument)
      .filter(Boolean)
      .flatMap(documents),
  ];
  const withPassword = (inputs) => inputs.some((input) => input.type === "password");
  const inputs = documents(document).map(fields).find(withPassword) ?? fields(document);
  const textual = (input) => TEXT_TYPES.includes(input.getAttribute("type")?.toLowerCase() ?? "");
  const hinted = (input) =>
    input.autocomplete === "username" ||
    input.type === "email" ||
    USERNAME_HINTS.test(`${input.name} ${input.id} ${input.placeholder} ${input.getAttribute("aria-label") ?? ""}`);

  const passwordInput = inputs.find((input) => input.type === "password");
  const before = (input) =>
    !passwordInput || input.compareDocumentPosition(passwordInput) & Node.DOCUMENT_POSITION_FOLLOWING;
  const candidates = inputs.filter((input) => textual(input) && before(input));
  const usernameInput = candidates.find(hinted) ?? candidates.at(-1);

  const fill = (input, value) => {
    input.focus();
    setValue.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  };

  if (usernameInput && username) {
    fill(usernameInput, username);
  }
  if (passwordInput && password) {
    fill(passwordInput, password);
  }
  return Boolean(usernameInput || passwordInput);
})
