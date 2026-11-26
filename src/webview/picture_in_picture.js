((x, y, action) => {
  const atPoint = document
    .elementsFromPoint(x, y)
    .find((element) => element instanceof HTMLVideoElement);
  const main = document.querySelector("video.html5-main-video") || document.querySelector("video");
  const watching = location.hostname.endsWith("youtube.com") && location.pathname === "/watch";
  const playing = Boolean(main) && !main.paused && !main.ended;
  if (action === "find") return Boolean(atPoint) || (watching && playing);
  const toggled = document.pictureInPictureElement
    ? document.exitPictureInPicture()
    : (atPoint || main)?.requestPictureInPicture();
  toggled?.catch(() => {});
  return true;
})
