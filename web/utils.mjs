let wlSentinel = Promise.resolve();

// Acquire screen wake lock when entering fullscreen.
window.document.onfullscreenchange = function () {
  if (document.fullscreenElement) {
    wlSentinel = wlSentinel.then(async () => {
      try {
        let sentinel = await navigator.wakeLock.request("screen");
        console.log("screen wake lock acquired");
        return sentinel;
      } catch (err) {
        console.warn("failed to acquire screen wake lock", err);
      }
    });
  } else {
    wlSentinel = wlSentinel.then(async (sentinel) => {
      if (sentinel) {
        await sentinel.release();
        console.log("screen wake lock released");
      }
    });
  }
};
