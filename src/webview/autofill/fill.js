(({ origin, username, password, purpose }) => {
  if (location.origin !== origin) return;
  const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set;

  const documents = (root) => [
    root,
    ...[...root.querySelectorAll("iframe, frame")]
      .map((frame) => frame.contentDocument)
      .filter(Boolean)
      .flatMap(documents),
  ];
  const focused = form.role(document.activeElement) ? document.activeElement : null;
  const withPassword = () => documents(document).find((page) => form.passwords(page).length > 0) ?? document;
  const root = focused ? form.scope(focused) : withPassword();
  const passwords = form.passwords(root);
  const chosen = passwords.filter((input) => input.autocomplete === "new-password");
  const fresh = chosen.length > 0 ? chosen : passwords.slice(passwords.length > 2 ? 1 : 0);
  const targets = purpose === "new_password" ? fresh : passwords.slice(0, 1);

  const fill = (input, value) => {
    input.focus();
    setValue.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  };

  const usernameInput = form.username(root);
  if (usernameInput && username) fill(usernameInput, username);
  if (password) targets.forEach((input) => fill(input, password));
})
