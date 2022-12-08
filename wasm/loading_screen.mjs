const MAX_DOT_COUNT = 3;

window.addEventListener("DOMContentLoaded", () => {
  const body = document.getElementById("body");
  const loadingScreen = document.getElementById("loading-screen");
  const dotElement = document.getElementById("loading-dots");

  if (body === null || loadingScreen === null || dotElement === null) {
    console.error("Can't find loading screen elements!");
    return;
  };

  let dotCount = 0;

  function animateLoadingDots() {
    dotElement.innerText = ".".repeat(dotCount).padEnd(3, "\u{00A0}");

    dotCount += 1;

    if (dotCount > MAX_DOT_COUNT) {
      dotCount = 0;
    }
  }

  function removeLoadingScreen() {
    clearInterval(loadingInterval);
    loadingScreen.remove();
  }

  // Wait for the `canvas` element to be created
  // This tells us that the game has been fully loaded
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      for (const addedNode of mutation.addedNodes) {
        console.debug("addedNode", addedNode, addedNode.ELEMENT_NODE);
        if (addedNode.ELEMENT_NODE === 1) {
          /** @type HTMLElement */
          const element = addedNode;

          console.debug(element.tagName);

          if (element.tagName === "CANVAS") {
            // The game has been loaded!
            // We don't need the loading screen anymore
            removeLoadingScreen();
            observer.disconnect();
          }
        }
      }
    }
  });

  observer.observe(body, {
    childList: true,
  });

  const loadingInterval = setInterval(animateLoadingDots, 1000);
});
