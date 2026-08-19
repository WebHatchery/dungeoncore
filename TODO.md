# TODO — Dungeon Core

AI-executable implementation and automated verification only. Audio work is
tracked separately and is not included here.

## Game readiness

- [x] Finish the remaining title/prestige presentation and elemental, trap,
  siege, and prestige VFX implementation, with deterministic captures at
  standard and narrow browser sizes. The capture wrapper now includes the VFX
  showcase and accepts `-Narrow` for a 390×844 browser viewport.
- [x] Complete browser input/accessibility behavior: a touch/click-only player
  must be able to start a run, finish the tutorial, build, fight, pause,
  recover, change settings, and reach game over. Focus-loss suspension pauses
  safely, reduced motion freezes animated emphasis, colour-independent markers
  remain visible, and narrow settings controls are geometry-tested.
- [x] Add an automated maximum-dungeon/full-roster soak at 4× speed and fix
  frame-time, memory, and drawer-performance regressions it exposes. The soak
  covers 20 floors, 100 combat rooms, all 47 authored monsters, bounded logs
  and effects, and the stale-Core descent edge case.
- [x] Add end-to-end tests covering visible controls, log output, save/reload,
  pause, overlays, and resource-panel state.
