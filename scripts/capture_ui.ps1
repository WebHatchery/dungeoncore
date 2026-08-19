# Per-game screenshot wrapper for Dungeon Core.
#
# Builds the game and captures one PNG per scene into docs\verification\ via the
# shared macroquad_toolkit harness. The game reads DUNGEON_CORE_CAPTURE_* env
# vars (see src/main.rs: render_playing_frame + seed_capture_scene).
#
# Usage (from the dungeon_core directory):
#   & .\scripts\capture_ui.ps1                       # all default scenes
#   & .\scripts\capture_ui.ps1 -Scenes gameplay      # one scene
#   & .\scripts\capture_ui.ps1 -SkipBuild            # reuse the current build
#
# Scenes include opening (fresh dungeon), gameplay (mid-raid dungeon),
# branching/deep_board (camera stress), tutorial, species, title, new_game,
# settings, save_slots, overwrite, codex, controls, confirmation, and log.

param(
    [string[]]$Scenes = @(
        "title", "save_slots", "new_game", "settings", "species",
        "opening", "tutorial", "build", "defenders", "traps", "variants", "placement",
        "gameplay", "transit", "branching", "deep_board", "journal", "rival",
        "swap", "codex", "coretree", "goals", "controls", "log", "pause",
        "summary", "siege", "prestige_vfx", "vfx_showcase", "confirmation",
        "overwrite", "gameover"
    ),
    [int]$Frames = 90,
    [int]$WindowWidth = 0,
    [int]$WindowHeight = 0,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild,
    [switch]$Narrow
)

if ($Narrow) {
    if ($WindowWidth -le 0) { $WindowWidth = 390 }
    if ($WindowHeight -le 0) { $WindowHeight = 844 }
    $Scenes = @($Scenes | ForEach-Object { "${_}_narrow" })
}

$shared = Join-Path $PSScriptRoot "..\..\macroquad-toolkit\scripts\capture_ui.ps1"
& $shared -GameDir (Join-Path $PSScriptRoot "..") -Scenes $Scenes -Frames $Frames `
    -WindowWidth $WindowWidth -WindowHeight $WindowHeight -OutputDir $OutputDir `
    -SkipBuild:$SkipBuild
