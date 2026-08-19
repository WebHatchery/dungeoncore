# TODO — Dungeon Core

AI-executable implementation and automated verification only. Audio work is
tracked separately and is not included here.

## Game readiness

- [ ] Finish the remaining title/prestige presentation and elemental, trap,
  siege, and prestige VFX implementation, with deterministic captures at
  standard and narrow browser sizes.
- [ ] Complete browser input/accessibility behavior: a touch/click-only player
  must be able to start a run, finish the tutorial, build, fight, pause,
  recover, change settings, and reach game over. Cover focus-loss pause,
  reduced motion, colour-independent cues, tooltips, and narrow layouts with
  automated checks where practical.
- [ ] Add an automated maximum-dungeon/full-roster soak at 4× speed and fix
  frame-time, memory, and drawer-performance regressions it exposes.
- [ ] Add end-to-end tests covering visible controls, log output, save/reload,
  pause, overlays, and resource-panel state.
