# TODO — Dungeon Core

Everything through the "make the simulation legible" and "give the player levers"
work is shipped. What follows is what is still genuinely open.

## Queued next (specced, not started)

Six items from playtesting, in the order they were raised. All six are now
built; what remains under each is balance tuning against real play.

### Monster status in the room inspector — *done*

A room's defenders were opaque: `draw_monster_progress_rows` printed a type name
and a progress string, nothing else.

Shipped: each defender now gets a card (`draw_defender_row`) carrying a health
bar coloured by how hurt it is, `hp/max_hp`, floor-scaled ATK/DEF, its element
and traits, and its line's variant progress — with the card tinted by element
while alive. A fallen defender takes the danger tone and says "Fallen" rather
than merely dimming, so it reads as a state to act on. `DEFENDER_ROW_H` went
24 → 46 and `MAX_DEFENDER_ROWS` 6 → 4; the card height math follows both.
Verified against a `defenders` capture scene holding one whole, one wounded and
one fallen creature in an over-capacity room.

The `*_preview` / summary text builders moved to `ui/upgrade_panel/previews.rs`
to make room, which is the extraction the code-health list called for.

Still open:

- The swap control lands on these rows — see the variants item below.

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

### Adventurer journal (follow the NPCs) — *done*

`HeroRecord` already persisted the numbers; the HEROES tab listed them but rows
were inert, so an individual adventurer had no page.

Shipped:

- `HeroRecord.journal` — a `Vec<HeroEvent>` (`day` + a sentence),
  `#[serde(default)]` so older saves start empty, bounded to
  `HERO_JOURNAL_LIMIT` (12) by `remember()`, which drops the oldest line as it
  pushes. A test pins that bound: a long campaign cannot grow the save without
  limit.
- Four moments are recorded against the hero id: entering (first delve or
  returning for delve N), each kill *by name and floor*, escaping with a purse
  (and any level gained), and falling. `record_hero_kill` now takes the monster
  and floor, and `kill_credits` in combat carries the slain creature's name
  along with its slayer.
- Clicking a HEROES row opens that hero's page in place of the list: name,
  level/race/class, standing, the rival badge with its bounty spelled out, the
  three lifetime totals, death circumstances when fallen, and the history
  newest-first. A "< Heroes" control returns to the list.
- `GameState.selected_hero` is `#[serde(skip)]` UI state, driven by two new
  `DrawerAction` variants.

Verified against a `journal` capture scene: a five-delve rival with a full
history and a live bounty.

Still open:

- Live tracking of a hero inside the dungeon (current room, HP, conditions)
  remains out of scope — the journal is a ledger; the board is where a raid is
  watched.

### Traps and upgrades use the monster placement flow — *done*

Traps had the flow half-built: the TRAPS & LOOT tab set `selected_upgrade` and
`main.rs` applied it on a room click, but the board never reacted, so the
discoverable path stayed the inspector's own catalog — a second, different
interaction for the same job.

Shipped:

- `placement_state` and `current_objective` know about `selected_upgrade`.
  Arming a trap lights the rooms that can take it and states the objective
  ("Install Poison Dart in a combat room"), exactly as arming a monster does.
  Validity differs by kind: a monster wants a free slot, an upgrade wants a room
  not already holding one of its type, and the board says which refusal it is —
  "Full" against "Has one".
- The inspector's catalog is gone. ACTIONS now leads with **Add upgrade (N)**,
  which arms the drawer tab, and lists what is installed with its Remove control
  underneath. One build path, reviewed from the room.
- The catalog's description text moved to the drawer rows, which now say what
  each upgrade actually does rather than just naming its family — the drawer is
  the only place to choose from now, so it carries the information.
- Drawer entries grey out during a raid, matching `apply_upgrade`'s existing
  refusal instead of letting the click fail. (The earlier note here was wrong:
  the gate was always enforced in the simulation; only the affordance was
  missing.)
- `upgrade_scroll` went with the catalog — the panel no longer has anything to
  scroll.

Verified against a `traps` capture scene: one trapped room, one empty, a trap
armed, both refusals and the highlight visible at once.

### Variants: pooled per line — *done, thresholds need rebalancing*

**Individual monsters do not level and do not evolve.** XP accrues to a monster
*type*: every Goblin feeds one shared Goblin pool, and crossing a threshold
unlocks the next variant. A creature only gets deadlier by being placed on a
deeper floor. Individual XP and levels belong to adventurers alone
(`HeroRecord.experience` / `level`) — the journal item above surfaces those.

Shipped (the pooled model):

- `Monster.experience` is gone. `GameState.monster_type_experience`
  (`HashMap<String, i32>`, `#[serde(default)]`) holds the pools, read and
  written through `type_experience` / `add_type_experience`. Serde drops the old
  per-creature field silently, so no save migration was needed. Prestige never
  resets the dungeon, so the pool simply persists.
- `reward_adventurer_kills` credits the pool of every type that survived the
  fight, once per surviving creature.
- `process_evolution_unlocks` reads the pool and still gates on depth: a line
  unlocks a variant only while fielded at that variant's `min_floor`.
- The bulk `process_evolutions` button is deleted, along with
  `DrawerAction::ProcessEvolutions` and the dead `ControlAction` variant.
- The EVOLVE tab is now **VARIANTS**: one row per line, with pooled XP against
  the threshold and what unlocks next.

Shipped (the swap):

- `simulation/monsters/swap.rs`. `plan_swap` says what placing a monster onto an
  occupied slot would do and what it costs; `swap_monster` does it. A newcomer
  one step along the occupant's evolution path is an **upgrade** — same slot,
  same creature id, new form scaled for the floor, paying only the mana
  *difference* plus that path's `gold_cost` (which is what finally put
  `EvolutionConditions.gold_cost` back to work). Anything else is a **replace**:
  the occupant is retired at `remove_monster`'s half-refund and the newcomer
  summoned at full price.
- The upgrade branch is arithmetically guaranteed cheaper than replacing — it
  pays `new - old` where replacing pays `new - old/2` — which is what makes
  growing a line worth more than swapping in the best affordable thing. A test
  pins that relationship rather than trusting the numbers.
- Every check runs before anything is destroyed, so a swap the dungeon cannot
  afford leaves the occupant untouched. Swapping is barred mid-raid: upgrading
  rebuilds the creature at full HP, which would otherwise be a free heal for a
  defender being hit.
- The inspector's defender rows are the drop targets. With a monster armed each
  card states its own verdict and price ("Upgrade · 15M", "Replace · 20M"),
  tinted by branch, and unaffordable ones read in danger colour and refuse the
  click. The armed monster's own card compacts to identity and price while a
  room is open so the rows stay on screen. A `swap` capture scene covers both
  branches side by side.

Still open:

- **Rebalance `experience_required`.** The thresholds were written for a single
  creature's XP; a pool filled by N creatures crosses them far faster. This is
  the last real gap in the variants work.
- A two-step reach (placing a Troll onto a Goblin) counts as a replace, not an
  upgrade — only one step along the line is an upgrade. That is deliberate for
  now; revisit if it feels punitive in play.

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

### Little creatures fighting — *planned next*

**Goal:** every living defender and adventurer is represented on the dungeon
board by a recognisable little sprite, parties visibly walk between rooms, and
an occupied combat room reads as a fight without requiring bespoke attack
animation for every creature. Melee can happen inside a dust cloud: the units,
health bars, impacts, damage and deaths matter more than literal weapon swings.
Combat remains deterministic and authoritative; all animation is cosmetic,
transient and absent from saves.

#### Visual direction and affordable asset scope

- Use compact 24×24 or 32×32 pixel-art sprites drawn with nearest-neighbour
  filtering. This is affordable across the current roster and stays legible in
  the board's small room tiles.
- Begin with one adventurer sheet per class (Warrior, Rogue, Mage, Cleric,
  Ranger, Paladin and Alchemist) and one base monster sheet per species.
  Adventurer races use palette/accessory variations; evolved monsters share
  their species body and vary by palette, equipment, silhouette details and
  scale. Do not make 47 unrelated monster sheets or 28 unrelated adventurer
  sheets merely to achieve coverage.
- Give each sheet small `idle`, `walk`, `attack` and `death` clips. The first
  vertical slice only needs Goblin plus Warrior, Rogue and Mage art; prove the
  pipeline and room composition before filling the roster.
- Keep the existing coloured initial discs as a conspicuous missing-asset
  fallback. A broken mapping must log clearly and never make a combatant
  invisible.

#### Shared animation primitive

`macroquad-toolkit::sprite` can draw a texture but has no sprite-sheet clip
abstraction. Add the missing shared capability there before inventing one in
Dungeon Core: atlas frame rectangles, named looping/one-shot clips, horizontal
flip, nearest-neighbour drawing, and pure frame selection from elapsed time.
Unit id supplies a stable phase offset so a room of goblins does not breathe in
perfect synchrony. Cover frame selection, looping and one-shot clamping with
toolkit unit tests.

#### Asset loading and mappings

- Add a sprite manifest mapping every monster name and adventurer class to its
  sheet, clips, palette/variation and display scale. Load sheets through the
  existing `AssetManager` in `main.rs`, including the capture harness, and pass
  the visual assets down to the dungeon board.
- Keep sprite PNGs transparent, load them with `FilterMode::Nearest`, and make
  sure the root publisher includes them in both the loose-assets and
  `assets.zip` paths used by native and WebGL builds.
- Add a coverage test that every monster in `assets/monsters.json` and every
  class in `assets/adventurers.json` resolves to a sprite definition or an
  intentional documented fallback.

#### Room unit presentation

`ui/dungeon_view/room_art.rs` is already substantial. Before expanding it,
extract unit composition to `ui/dungeon_view/room_art/units.rs` and transient
visuals to `ui/dungeon_view/room_art/effects.rs`; do not drive the file toward
the 800-line gate.

- Replace the current defender and adventurer discs with sprites: defenders on
  the left, invaders on the right, facing the room centre. Retain compact health
  bars, monster element cues, rival rings/names and the `+N` overflow rule.
- Derive idle bob and clip frame from time plus stable unit id. Drawing must not
  mutate gameplay state. At narrow room scales, prioritise the front units and
  counts instead of shrinking every sprite into noise.
- Upgrade the current single `A` corridor marker to the first two or three
  surviving adventurer sprites in a walking formation during the existing
  `PARTY_MOVE_SECONDS` tween, with excess members represented by a count badge.

#### Combat presentation cues

Extend the room-anchored transient effect model separately from floating text.
It needs short-lived `MeleeDust`, `HitSpark`, `MonsterDeath` and
`AdventurerDeath` cues with their own lifetimes. Screen position remains a UI
concern so effects follow a room when layout or window size changes.

On each `resolve_combat` tick:

1. Spawn a 0.6–0.9 second dust cloud in the room centre when both sides engage.
2. Briefly lunge the visible front units toward it or select their attack clip.
3. Spark on the side taking damage while the existing damage text rises above
   that side.
4. Puff/fade each casualty and then remove its sprite; health bars and controls
   must remain readable throughout.
5. Later layer element-distinct colours and trap cues over this same event path
   without changing combat timing or damage rules.

#### Capture, verification and completion gate

- Add a `combat_sprites` capture scene containing several adventurer classes,
  multiple monster silhouettes, wounded units, one party in transit, an active
  dust cloud and a death cue. Use the toolkit filmstrip capture to review motion
  across frames, plus static captures at 1280×720 and a narrower common desktop
  size.
- The goal is done only when every living unit has a sprite or explicit
  overflow count, parties visibly walk, fighting produces and clears a dust
  cloud on combat ticks, damage/deaths remain readable without the log, missing
  art falls back safely, and animation cannot affect deterministic simulation.
- After meaningful implementation stages and again at completion, run
  `.\publish.ps1` with no parameters from this project and report whether it
  passes. Once the completion publish passes, commit all intended files using
  `rust_management/docs/COMMIT_STYLE.md`: Dungeon Core's diegetic subject,
  plain-terms parenthetical tag, honest prose body including verification, and
  the required AI co-author trailer. A passing publish without that standards-
  compliant commit does not complete this goal.

#### Later art following the creature pass

- Trap, upgrade and attunement art; themed room interiors with floor-depth
  theming; the core room as a visual centrepiece.
- Element-distinct VFX beyond the melee slice (fire, frost, poison, trap
  triggers, siege arrival and prestige).
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
- `src/ui/controls.rs::draw_controls` appears fully superseded by `ui/shell.rs`,
  which reuses only `ControlAction`'s `ToggleSpeed` / `ToggleDungeon`. The rest
  of that module looks dead — confirm and delete it.

## Deferred on purpose

- No more monsters, species, or elements until branching layouts and the art
  pass make the existing 47 feel distinct.
- No multiplayer, no deeper combat math, no player-facing equipment-loot economy.
- Party splitting at forks, and species beyond the current eight
  (Plant/Fungal, Insect), are post-launch stretches at best.
