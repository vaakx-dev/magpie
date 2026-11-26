(() => {
  const SPECULATIVE = 'link[rel~="prefetch" i], link[rel~="prerender" i], script[type="speculationrules" i]';
  const supports = HTMLScriptElement.supports;
  HTMLScriptElement.supports = (type) =>
    type.toLowerCase() !== "speculationrules" && supports.call(HTMLScriptElement, type);
  const drop = (node) => {
    if (node.nodeType !== Node.ELEMENT_NODE) return;
    if (node.matches(SPECULATIVE)) return node.remove();
    node.querySelectorAll(SPECULATIVE).forEach((element) => element.remove());
  };
  new MutationObserver((records) => {
    for (const record of records) record.addedNodes.forEach(drop);
  }).observe(document, { childList: true, subtree: true });
})();
