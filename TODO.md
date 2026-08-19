# TODO — Dungeon Core

Release checklist for itch.io. Audio work is tracked separately and is not
included here. This file lists unfinished work only.

## Game readiness

- [ ] Tune the raid economy, room capacity, threat/siege curve, and reputation
  against real runs on every difficulty preset. Confirm that visits are
  meaningfully mana-positive without making the resource cap irrelevant.
- [ ] Give the title and prestige screens their final shipping pass. Finish the
  remaining elemental, trap, siege, and prestige VFX polish while keeping unit
  state readable at normal and narrow browser sizes.
- [ ] Complete browser accessibility and input QA: a touch/click-only player
  must be able to start a run, finish the tutorial, build, fight, pause,
  recover, change settings, and reach game over. Check focus-loss pause,
  reduced motion, colour-independent cues, tooltips, and narrow layouts.
- [ ] Run a full-roster, maximum-dungeon soak at 4× speed and through a long
  session; resolve frame-time, memory, and drawer-performance regressions.
- [ ] Add and run a small end-to-end playthrough check covering visible controls,
  log output, save/reload, pause, overlays, and resource-panel state.

## Itch.io release gate

- [ ] Add the project `itch.json` with the final Butler target, HTML5 channel,
  Windows channel, and release version.
- [ ] Build a release candidate with `publish.ps1`, stage it with
  `publish-itch.ps1 -DryRun`, and verify the browser package, asset loading,
  local saves after reload, Windows download, and final page metadata.
