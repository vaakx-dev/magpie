(receive) => {
  const webview = window.chrome.webview;
  webview.addEventListener("message", (event) => receive(event.data));
  return (message) => webview.postMessage(message);
}
