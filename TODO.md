# TODO — Dungeon Core

Everything through the "make the simulation legible" and "give the player levers"
work is shipped. What follows is what is still genuinely open.

## Dungeon graph (branching layouts)

The dungeon is still a linear room queue. The edge model (`Room::exits`,
`Floor::validate_graph`, save migration) and fork path-selection
(`simulation/pathing.rs`) are in and tested, but no floor can actually branch yet.

- Build op: "Branch from here" creates a parallel room that reconverges at the
  selected room's successor (series-parallel diamonds, max 3 exits per room,
  never orphan the Entrance→Core route).
- Layered rendering: a new `ui/dungeon_view/layout.rs` computing depth (column)
  and lane (row) from `exits`, per-edge connectors, party token riding the
  chosen edge instead of `position → position + 1`.
- Surface the party's chosen route and its reason (loot bait vs. beeline) on the
  board and in the log so the fork decision is legible.
- Tutorial beat teaching the fork; balance pass on choke-point value.

## Art

- Pick a visual identity that is affordable across ~70 assets (pixel art is the
  realistic option) — everything else here depends on that call.
- Monster sprites (47, sharing bases across evolution lines is fine);
  `assets/image_prompts.json` is prepped for generation.
- Adventurer sprites (7 classes × 4 races, palette/part swaps fine).
- Trap, upgrade, and attunement art; themed room interiors with floor-depth
  theming; the core room as a visual centrepiece.
- Idle/walk/attack animation and element-distinct VFX (fire, frost, poison,
  trap triggers, deaths, siege arrival, prestige).
- Cohesive UI kit replacing the programmer-art panels and emoji glyphs.
- Title, game-over, and prestige screens at shipping quality.

## Audio

- SFX set (~30–50): build/summon, per-element hits, trap triggers, UI, deaths,
  income ticks, threat stings, siege alarm, core-damage heartbeat, prestige.
- Music: build theme, raid tension layer, siege track, title; adaptive layering.
- Room ambience that deepens with floor depth.
- Mixing, ducking, and per-channel volume (needs the settings menu).
- Verify macroquad audio on native *and* WASM early — web audio needs an
  unlock-on-first-input path.

## Product & UX infrastructure

- Settings menu: audio sliders, fullscreen/resolution, UI scale, speed defaults,
  autosave interval, colourblind mode, reduced motion, key rebinding.
- Save hardening: multiple slots with metadata, explicit save-version discipline,
  corrupt-save backup/recovery that never panics or silently resets.
- True pause as a first-class state (pause-on-focus-loss, pause menu).
- Confirmations on destructive actions (reset run, dismiss monster, overwrite).
- Tooltips on every stat, cost, icon, and abbreviation.
- Large-dungeon hardening: dungeon-view scroll/zoom, log scrollback and
  filtering, drawer performance with the full monster roster, 20-floor legibility.
- Full keyboard coverage with an in-game reference; decide on gamepad/Steam Deck.
- Accessibility: shape-and-label element cues (the system is 100% colour-coded
  today), text-size options, reduced-motion/no-flash, no reaction-time gates.
- Localization readiness: externalize player-facing strings to an ID-keyed table,
  audit for text expansion, pick fonts with the needed glyph coverage.

## Platform & technical

- Platform decision (Steam primary, itch secondary, keep the WASM build as a
  demo funnel) and Steamworks integration: achievements seeded from the
  milestone ids, cloud saves, store plumbing.
- Windows packaging: icon/version metadata, code-signing decision, installer.
- macOS/Linux support-tier decision.
- Performance: soak-test a max-size dungeon at 4× speed, profile the per-tick
  sim, long-session leak check.
- Top-level panic hook that writes a crash log and preserves the save.
- Seeded per-run RNG for reproducible bug reports and seeded challenge runs.
- Release pipeline on top of the existing fmt/clippy/test CI: per-platform
  builds, versioned artifacts, a build stamp visible on the title screen.

## QA, balance & business

- External playtest program (10–20 testers) with a feedback form and a build
  channel; validate the first hour lands a "my combo crushed a party" moment.
- Balance instrumentation: log per-run income curves, pick rates, death causes,
  and time-to-first-prestige to a local file so passes are data-driven.
- Full balance pass across all species and difficulty presets.
- Compatibility matrix: min spec, multiple GPU vendors, high-DPI, ultrawide,
  60/144 Hz tick behaviour.
- Positioning and pricing; Steam page live early for wishlists; trailer; demo
  build.
- Name check — "Dungeon Core" is generic and heavily used in LitRPG; search
  Steam and the trademark registers before spending on marketing.
- Legal/admin: entity and tax setup, EULA, privacy policy if telemetry ships,
  third-party licence audit including fonts and generated-art tool terms,
  age-rating questionnaires.
- Post-launch plan: patch cadence, community channel, a native bug-report path
  to match the web widget, and a 1.x content roadmap.

## Testing & code health

- Input-state tests for pause, focus, tooltip blocking, resource-panel updates,
  and log-message ordering.
- Move resource-panel calculations into pure helpers with fixtures for zero
  income and capped resources.
- Small dungeon-run scenarios that verify controls, log output, and theme-driven
  UI states together.
- Extract repeated drawing constants into toolkit-backed theme helpers shared by
  controls, logs, and resource panels.
- Keep the JSON-integrity suite growing with content, and every file under the
  800-line limit as UI work lands.

## Deferred on purpose

- No more monsters, species, or elements until branching layouts and the art
  pass make the existing 47 feel distinct.
- No multiplayer, no deeper combat math, no player-facing equipment-loot economy.
- Party splitting at forks, and species beyond the current eight
  (Plant/Fungal, Insect), are post-launch stretches at best.
