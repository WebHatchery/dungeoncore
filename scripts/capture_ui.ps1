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
# branching/deep_board (camera stress), tutorial, and species.

param(
    [string[]]$Scenes = @("gameplay", "tutorial", "species"),
    [int]$Frames = 90,
    [switch]$SkipBuild
)

$shared = Join-Path $PSScriptRoot "..\..\macroquad-toolkit\scripts\capture_ui.ps1"
& $shared -GameDir (Join-Path $PSScriptRoot "..") -Scenes $Scenes -Frames $Frames -SkipBuild:$SkipBuild
