const MAIN_WINDOW_GEOMETRY = Object.freeze({
  width: 1200,
  height: 800,
  minWidth: 900,
  minHeight: 600,
});

function resolveMainWindowGeometry(savedBounds) {
  return {
    ...MAIN_WINDOW_GEOMETRY,
    ...(savedBounds ?? {}),
  };
}

module.exports = { MAIN_WINDOW_GEOMETRY, resolveMainWindowGeometry };
