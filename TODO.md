# TODO — Dungeon Core

Everything through the "make the simulation legible" and "give the player levers"
work is shipped. What follows is what is still genuinely open.

## Queued next (specced, not started)

Six items from playtesting, in the order they were raised. Directions below are
decided; the code references are where the work lands.

### Monster status in the room inspector

A room's defenders are effectively opaque. `draw_monster_progress_rows`
(`src/ui/upgrade_panel.rs:442`) prints only the type name and an evolution
status string — no HP, no element, no traits, no XP numbers — so there is no way
to tell a healthy defender from one crawling back at half HP, or to see which
monster is close to evolving.

- Extend the existing rows in place (decided: richer rows, not a new page or a
  board hover card). Each row wants: an HP bar (`monster.hp`/`max_hp` — respawn
  already returns living defenders at half HP when mana ran short), element,
  trait names, and floor-scaled attack/defense as numbers rather than a phrase.
  No per-creature level or XP — that model is dropped (see the variants item
  below); the row instead shows the *type's* variant progress, and it is also
  where the swap control lands, so design the two together.
- `DEFENDER_ROW_H` and `MAX_DEFENDER_ROWS` both need a pass — the taller rows
  change how many fit, and the card height math in `draw_selected_room` is
  hardcoded around the current row size.
- Dead-but-not-yet-respawned defenders should read as dead in the list, not just
  dimmed text.

### Adventurer visits should be mana-positive — *mechanism done, tuning open*

A living adventurer inside used to be worth a flat `count * 0.5` mana/hour
regardless of level, against raid costs of respawning each dead living defender
at half its summon cost (`src/simulation/monsters.rs:193`) and re-arming each
sprung trap at a quarter of its price (`src/simulation/combat/traps.rs:216`) —
so a visit lost mana.

Shipped: `adventurer_presence_regen` (`src/simulation/time.rs`) now pays
`mana_regen_per_adventurer + level * mana_regen_per_adventurer_level` per living
intruder per hour, times `income_mult`, with both rates in the `time` block of
`assets/constants.json` (0.5 / 0.5 to start). Difficulty now scales the trickle
as well as the death payout. Presence is deliberately the main income during a
raid — a death is still a burst, but it *ends* the trickle, so a long deep delve
beats a quick wipe and farming kills trades income for the `threat_tier()` climb
toward the siege.

Still open:

- **Tune the two rates in play.** The target: a full party clearing to the boss
  and wiping leaves the dungeon net mana-positive after respawn and re-arm
  costs, at every difficulty preset. Only the JSON needs to move.
- The `.min(state.max_mana)` clamp is kept — overflow is lost by design, which
  is what makes a cap upgrade a real choice. Revisit only if tuning shows the
  early cap eating a meaningful share of a raid's income.
- The raid report still shows only "Mana earned"; it never shows what the raid
  *cost* in respawns and re-arms, so the player cannot yet see the net. Worth a
  pass once the rates settle.

### Adventurer journal (follow the NPCs)

`HeroRecord` (`src/game_state.rs:252`) already persists name, race, class, level,
delves, kills, gold stolen, status, death floor/day, plus `is_rival()` and
`bounty()`. The HEROES tab (`src/ui/side_drawer/heroes_tab.rs`) lists all of it
but rows are not clickable, so an individual adventurer has no page.

- Make a HEROES row open a per-hero journal page: the profile stats above, laid
  out properly, with the rival badge and bounty made explicit.
- Add a per-delve history — a short persisted event list on `HeroRecord`
  (entered on day X, slew Y on floor Z, fled with N gold, died to M). This is
  new state on a serialized struct, so it needs a `#[serde(default)]` field and
  a save-migration check; keep the list bounded so a long run cannot grow the
  save without limit.
- Events are already narrated into the game log at the moments that matter
  (kills, deaths, escapes) — the journal wants the same facts recorded against
  the hero id rather than a second source of truth.
- Live tracking of a hero inside the dungeon (current room, HP, conditions) is
  explicitly out of scope for the first pass; the journal is a ledger, not a
  minimap. Revisit once the page exists.

### Traps and upgrades use the monster placement flow

Monsters are placed by selecting from the drawer and clicking a room. Traps and
the other room upgrades have that flow half-built — the TRAPS & LOOT tab already
sets `selected_upgrade` and `main.rs` already applies it on a room click — but
the board never highlights anything, because `placement_state`
(`src/ui/dungeon_view.rs:247`) only reacts to `selected_monster`. So the
discoverable path is still the inspector's `draw_upgrade_choices` catalog, which
is a different interaction for the same job.

- Teach `placement_state` and `current_objective` about `selected_upgrade`, so
  arming a trap highlights valid rooms and states the objective exactly as
  monster placement does. Validity differs from monsters: a room can hold only
  one of each `RoomUpgradeType` (see the `installed` filter in
  `draw_upgrade_choices`), so rooms that already hold that type read as invalid.
- The inspector keeps its catalog as **review + jump**, not a second build path:
  list what is installed with remove controls, and an "Add upgrade" control that
  arms the drawer tab pre-filtered to what this room can still take.
- The `state.adventurer_parties.is_empty()` gate on applying and removing
  upgrades must survive the move — the drawer path currently enforces cost but
  not the no-raid-in-progress rule that the inspector rows do.

### Evolution becomes type-level variant unlocks

**Individual monsters do not level and do not evolve.** XP accrues to a monster
*type*: every Goblin in the dungeon feeds one shared Goblin pool, and when that
pool crosses a threshold, Goblin Warrior unlocks — both as a new placeable unit
in the MONSTERS tab and as an upgrade the player can apply to an already-placed
Goblin. A given monster only gets deadlier by being placed on a deeper floor
(`get_scaled_stats(base, floor_number, is_boss)` already does this); there is no
per-creature progression to track.

Individual XP and levels belong to adventurers, not defenders — that side is
already correct (`HeroRecord.experience` / `level`, advanced in
`src/simulation/adventure.rs:449` with `xp_for_level`), and the journal item
above is where it gets surfaced.

What contradicts the model today:

- `Monster.experience` (`src/game_state.rs:91`) is per-creature. It gets deleted.
  Serde ignores unknown fields, so old saves drop it without a migration step.
- `reward_adventurer_kills` (`src/simulation/combat/rewards.rs:101`) awards XP to
  every surviving monster in the room individually. It should instead credit the
  shared pool for each of those monsters' types.
- `process_evolutions` (`src/simulation/monsters.rs:356`, reached from
  `DrawerAction::ProcessEvolutions` in `src/main.rs:430`) is a bulk button that
  transforms every eligible defender at once. It goes away.
- `process_evolution_unlocks` (`src/simulation/monsters.rs:323`) has the right
  shape already — it unlocks a form without transforming anything — but reads
  per-monster XP. It rereads the type pool instead, and can stop scanning every
  room every hour since the pool is a single lookup.

The work:

- New state: a type-keyed XP pool on `GameState` (`HashMap<String, i32>` keyed by
  `type_name`), `#[serde(default)]` so old saves start empty. Decide whether the
  pool survives prestige or resets with the run.
- **Placing a monster onto an existing one is the interaction**, and what happens
  depends on whether it is that creature's own line:
  - *Direct upgrade* (Goblin Warrior onto a Goblin): the creature transforms in
    place, no retirement, no refund.
  - *Unrelated monster* (Harpy onto a Goblin): the Goblin is retired for half its
    mana back, then the Harpy is summoned at full price. Exactly the existing
    dismiss-then-summon pair, done in one click — reuse `remove_monster`'s refund
    rule (`get_monster_mana_cost(base, floor, boss_surcharge) / 2`, souls stay
    spent) rather than inventing a second refund path.
  - The upgrade branch must be strictly cheaper than the retire-and-replace one,
    or the whole variant line is pointless. Its price is the one number still
    open: full variant cost, or the difference against what the base creature is
    worth.
- That means placement needs a **per-monster target**, which does not exist today
  — `place_monster` only appends to `room.monsters`. With a monster armed in the
  drawer, the inspector's defender rows become drop targets ("place on this
  defender") while clicking an open slot still adds a new one. Board-level
  targeting can come later; the inspector rows are the cheap version and they are
  already being rebuilt for the status item above. The room creature limit below
  is in, so this already bites: a full room can only be improved by replacing an
  occupant.
- `process_evolutions`' second pass already transforms correctly (rescales via
  `get_scaled_stats`, rebuilds `active_traits`) — reuse it as a single-monster
  function for the upgrade branch rather than rewriting it.
- `evolution_trees.json` stays the source of truth for which variant follows
  which monster; `experience_required` becomes a type-pool threshold rather than
  a per-creature one, so the numbers need a rebalance pass — one pool filled by
  N goblins fills far faster than any single goblin did.
- Rename throughout the UI. The EVOLVE tab, `monster_evolution_status`, and
  `template_evolution_hint` speak of "evolution" and "next form"; the model is
  "variants unlocked by a type's collective experience". The tab stops *doing*
  evolutions and becomes a progress board: pool XP per type, what unlocks next,
  how far off it is. Note that with XP no longer per-creature, the defender row
  in the inspector (first item above) shows the *type's* progress, identical for
  every monster of that type in the room.
- The player needs to see which branch a click will take *before* taking it — a
  direct upgrade and a retire-and-replace cost very different amounts. Show the
  outcome and price on the targeted row while a monster is armed.

### Rooms get a creature limit that grows with depth — *done, balance open*

Rooms used to hold unlimited defenders; `place_monster` just pushed onto
`room.monsters`.

Shipped: `room_monster_capacity(floor, is_boss)` in `src/data/constants.rs`,
driven by a `monster_capacity` column on the `floor_scaling` table (2/3/3/4/4 for
floors 1–5), extrapolated past the table at `monster_capacity_increase` per
floor, with `boss_room_capacity_delta` (-1) trading a throne room's slot for its
boss. `place_monster` refuses a full room; `placement_state` marks it an invalid
target so the board dims it and shows a "Full" pill instead of letting the click
fail; every combat room tile carries `used/capacity` on its label plate and the
inspector's `Defenders` line reads `N alive · used/capacity slots`.

Still open:

- **Balance the curve against room and monster cost.** Slots are now the scarce
  build resource, so adding a floor competes with improving what is placed.
  Check the siege and threat curves still hold when the early dungeon fields two
  defenders per room instead of an unbounded pile.
- Whether boss rooms should reserve one of their slots for a boss-only monster,
  rather than just being one slot smaller.
- Capacity gates *player placement* only. `SplitOnDeath`
  (`src/simulation/combat.rs:149`) still spawns over the cap mid-fight, which is
  the trait working as intended — revisit only if a split-heavy room gets
  silly.

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
