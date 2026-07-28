(function () {
  if (window.__pakeHoverTitlebar) return;
  window.__pakeHoverTitlebar = true;

  let isShown = false;
  let lastShowTime = 0;
  let lastHideTime = 0;
  let hideTimeout = null;

  function setDecorations(show) {
    const invoke = window.__TAURI__?.core?.invoke;
    if (invoke) {
      invoke("set_window_decorations", { show }).catch((err) => {
        console.error("Failed to set window decorations:", err);
      });
    }
  }

  function toggleTitlebar(show) {
    const now = Date.now();
    if (show === isShown) return;

    if (show) {
      if (now - lastShowTime < 500) return;
      lastShowTime = now;
    } else {
      if (now - lastHideTime < 500) return;
      lastHideTime = now;
    }

    isShown = show;

    if (hideTimeout) {
      clearTimeout(hideTimeout);
      hideTimeout = null;
    }

    setDecorations(show);
  }

  // When the mouse is anywhere inside the webview document, show the title bar
  document.addEventListener("mousemove", () => {
    if (hideTimeout) {
      clearTimeout(hideTimeout);
      hideTimeout = null;
    }
    toggleTitlebar(true);
  });

  // When the mouse leaves the webview document (moving to the desktop or the native title bar)
  document.addEventListener("mouseleave", () => {
    if (hideTimeout) {
      clearTimeout(hideTimeout);
      hideTimeout = null;
    }

    if (isShown) {
      hideTimeout = setTimeout(() => {
        toggleTitlebar(false);
      }, 1200); // 1200ms delay to allow clicking window buttons
    }
  });
})();
