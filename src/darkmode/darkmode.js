(rules) => {
  if (window !== window.top) return;
  const DARKEN = "invert(1) hue-rotate(180deg) contrast(0.8)";
  const RESTORE = "contrast(1.25) hue-rotate(180deg) invert(1)";
  const KEPT = "img, picture, video, canvas, iframe, embed, object, svg image, [style*='background-image'], magpie-reader";
  const ACTIVE = ":root:not(:has(:fullscreen))";
  const DARK_LIMIT = 0.5;
  const BLANK_CANVAS = { red: 255, green: 255, blue: 255 };
  const sheet = new CSSStyleSheet();
  sheet.replaceSync(`
    ${ACTIVE} { filter: ${DARKEN} !important; }
    ${ACTIVE} :is(${KEPT}) { filter: ${RESTORE} !important; }
    ${ACTIVE} :is(${KEPT}) :is(${KEPT}) { filter: none !important; }
  `);
  const site = location.hostname.replace(/^(www\.)+/, "");
  const web = location.protocol === "https:" || location.protocol === "http:";
  const painter = new OffscreenCanvas(1, 1).getContext("2d", { willReadFrequently: true });
  let mode = "off";
  let pageDark = false;

  const modeOf = ({ standard, sites }) => (web ? sites[site] ?? standard : "off");

  const rgba = (color) => {
    painter.clearRect(0, 0, 1, 1);
    painter.fillStyle = color;
    painter.fillRect(0, 0, 1, 1);
    const [red, green, blue, alpha] = painter.getImageData(0, 0, 1, 1).data;
    return { red, green, blue, alpha };
  };

  const brightness = ({ red, green, blue }) => (0.2126 * red + 0.7152 * green + 0.0722 * blue) / 255;

  const backdrop = () =>
    [document.body, document.documentElement]
      .map((element) => rgba(getComputedStyle(element).backgroundColor))
      .find(({ alpha }) => alpha > 0);

  const darkScheme = () => {
    const scheme = getComputedStyle(document.documentElement).colorScheme;
    const prefersDark = matchMedia("(prefers-color-scheme: dark)").matches;
    return scheme.includes("dark") && (!scheme.includes("light") || prefersDark);
  };

  const lightText = () => brightness(rgba(getComputedStyle(document.body).color)) > DARK_LIMIT;

  const alreadyDark = (color) => (color ? brightness(color) < DARK_LIMIT : darkScheme() || lightText());

  const paintCanvas = ({ red, green, blue }) => {
    sheet.cssRules[0].style.setProperty("background-color", `rgb(${red} ${green} ${blue})`, "important");
  };

  const render = () => {
    sheet.disabled = mode === "off" || (mode === "auto" && pageDark);
  };

  const measure = () => {
    if (!document.body) return requestAnimationFrame(measure);
    sheet.disabled = true;
    const color = backdrop();
    pageDark = alreadyDark(color);
    paintCanvas(color ?? BLANK_CANVAS);
    render();
  };

  const follow = (next) => {
    mode = modeOf(next);
    render();
  };

  document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
  paintCanvas(BLANK_CANVAS);
  follow(rules);
  requestAnimationFrame(measure);
  document.addEventListener("DOMContentLoaded", measure);
  addEventListener("load", measure);
  window.chrome?.webview?.addEventListener("message", (event) => {
    if (event.data?.kind === "dark_mode") follow(event.data.rules);
  });
}
