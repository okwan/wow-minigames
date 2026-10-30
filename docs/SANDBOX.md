# The sandbox

Sandbox mode is how this repository runs the engine for our own games: the real WoW 1.12.1 world,
one local character standing in it, the engine's own third-person controls and camera, and **no
server, no login, no character select, no game rules**. `make build && make start` opens
`http://127.0.0.1:8090/` straight into it; `make native` does the same in a window.

This file is the investigation behind it (what the networked client does, what the removed
Azeroth Survivors mode did instead, and why the sandbox is built the way it is), the reference for
its parameters, and the list of everything that was removed or deliberately left.

- [Using it](#using-it)
- [Findings: how the engine runs a player](#findings-how-the-engine-runs-a-player)
- [Findings: how Survivors ran server-less](#findings-how-survivors-ran-server-less)
- [Design](#design)
- [Writing a game](#writing-a-game)
- [Risks and what was not verified](#risks-and-what-was-not-verified)
- [What was removed](#what-was-removed)

## Using it

Open the page, or run the native binary, and the loading screen rises, the world streams in, the
character appears and the camera settles behind it.

| Input | Does |
|---|---|
| `W` / `S` (or `Up` / `Down`) | run forward / backpedal |
| `A` / `D` (or `Left` / `Right`) | turn; strafe instead while the right mouse button is held |
| `Q` / `E` | strafe left / right |
| `Space` | jump |
| right mouse drag | steer: the character turns with the camera; hidden, locked cursor |
| left mouse drag | orbit the camera around the character; it swings back behind you when you move |
| both buttons | run forward, steering with the mouse |
| mouse wheel | zoom, from first person out to 30 yards |
| `Num Lock` | autorun |
| `X` / `Z` | sit or stand / sheathe or draw the weapon |
| `F1` | hide or show the help line and the coordinates readout |

Where and who, as query parameters in the browser or `WOW_SANDBOX_<NAME>` variables natively
(`webenv`: the page's `?sandbox_<name>=` and the bare `?<name>=` both work):

| `?param` | native | meaning | default |
|---|---|---|---|
| `map` | `WOW_SANDBOX_MAP` (or `WOW_MAP`) | `Map.dbc` id: `0` Eastern Kingdoms, `1` Kalimdor; or `eastern` / `kalimdor` | `0` |
| `x` `y` `z` | `WOW_SANDBOX_X` … | WoW coordinates. Leave `z` out and the character stands on the terrain at `x`, `y` | Northshire Abbey, `-8949.95 -132.49 84.5` |
| `facing` | `WOW_SANDBOX_FACING` | orientation in radians (`0` faces +x, north; counter-clockwise), or `90deg` | `0` |
| `race` | `WOW_SANDBOX_RACE` | `human` `orc` `dwarf` `nightelf` `undead` `tauren` `gnome` `troll`, or 1-8 | `human` |
| `gender` | `WOW_SANDBOX_GENDER` | `male` / `female` | `male` |
| `class` | `WOW_SANDBOX_CLASS` | `warrior` `paladin` `hunter` `rogue` `priest` `shaman` `mage` `warlock` `druid`, or the id | `mage` |
| `time` | `WOW_SANDBOX_TIME` | the light's time of day, `HH:MM` (there is no server clock to follow) | `13:00` |
| `example` | `WOW_SANDBOX_EXAMPLE` | `1` loads `crates/sandbox/src/games/example.rs` | off |

Examples:

```
http://127.0.0.1:8090/?race=orc&class=warrior&map=1&x=-950&y=-2700        # The Barrens, on the ground
http://127.0.0.1:8090/?race=nightelf&gender=female&class=druid&time=21:30   # Northshire at night
WOW_SANDBOX_RACE=dwarf WOW_SANDBOX_CLASS=hunter make native
```

A race and class combination 1.12 never offered (a Tauren Mage) is dressed in the nearest
trainer look available, and the log says so (`sandbox: no class-8 trainer look …`).

Other places worth starting at (WoW coordinates, from the removed Survivors mode's list of flat,
open ground; heights are as it sampled them, so leave `z` out if one turns out wrong):

| place | map | x | y | z |
|---|---|---|---|---|
| Northshire Abbey (the default; the 1.12 human start) | 0 | -8949.95 | -132.49 | 83.5 |
| Elwynn Forest, west of Goldshire | 0 | -9548.5 | 115.0 | 60.0 |
| Westfall | 0 | -10252.2 | 1483.5 | 41.6 |
| Duskwood | 0 | -10051.8 | -792.5 | 36.5 |
| Redridge Mountains | 0 | -9199.8 | -2767.5 | 91.0 |
| Dun Morogh | 0 | -5400.8 | -739.9 | 396.7 |
| Tirisfal Glades | 0 | 1993.0 | -153.3 | 35.4 |
| The Barrens, south of the Crossroads | 1 | -950.0 | -2700.0 | 92.0 |
| Durotar | 1 | 587.5 | -4311.4 | 19.0 |
| Mulgore | 1 | -1809.9 | -220.6 | -8.4 |
| Teldrassil | 1 | 9737.5 | 649.8 | 1298.5 |

## Findings: how the engine runs a player

Read from `crates/benilla-app/src` (`player.rs`, `player/`, `net/`, `char_select/`, `login/`,
`loading_screen.rs`, `bindings.rs`) before writing any sandbox code.

**Boot and world entry.** `lib.rs::launch` builds the engine (`benilla_world::WorldPlugins`) and the
game (`GamePlugins { connect, start }`). Normally `start` is `ClientState::Login`; `NetPlugin`
spawns the IO thread (a task on the browser's event loop), the login, realm-select and
character-select screens run, and picking a character sends `CMSG_PLAYER_LOGIN`. The server answers
with `SMSG_LOGIN_VERIFY_WORLD` (a `WorldportMessage`: map, position, orientation) and an
`SMSG_UPDATE_OBJECT` carrying our own player's create block. The loading cover is raised by the
`EnteredWorldMessage` and cleared once the world around the body is resident.

**The avatar is a streamed entity.** `net::handlers` turns the create block into an entity with
`Guid`, `NetEntity { kind, display_id, scale }` and `ObjectStore` (the descriptor fields). The
attach, equipment and animation systems dress and animate anything with those three. `SelfGuid`
names which guid is ours; `net::apply::tag_self_player` adds `SelfPlayer` to it;
`player::embody::maintain_embodiment` adds `Embodied` (the body the camera follows) and
`ActiveMover` plus a `MovementState` (the controller feeds it; the animation reads it).

**Taking control.** `player::wire_in::apply_server_moves`, inside the controller, "seizes" when
`Player.active` is false and the embodied body has a pose: it copies the pose into `Player`, sets
`settling` (gravity off) and `world_stale`, and points the camera behind the facing. The stream
follows `Player.pos` (`player::world_focus` publishes `ViewFocus::body`); `release_post_snap_hold`
ends the settle when the terrain and colliders under the body are resident; the loading screen
clears after that.

**The controller** is `player::controller::control`, one system: input to axes (`input.rs`), the
kinematic capsule over avian's `MoveAndSlide` against terrain, WMOs and doodads (`mover.rs`),
swimming (`swim.rs`), jump/fall (`arc.rs`, `land.rs`), animation flags (`flags.rs`, `body_pose.rs`),
and the camera (`camera.rs`, `camera_dynamics.rs`, `camera_zoom.rs`): right-drag steers, left-drag
orbits, both buttons run, the wheel zooms, `FollowStyle` reels a left-drag orbit back in behind a
moving character, collision pulls the camera in. Movement is streamed to the server through
`movement_net`. `control` is gated `not(CaptureMode)` and `InWorldGated`.

**Input does not read keys.** The controller reads `bindings::BindingsState`, never `ButtonInput<KeyCode>`.
`latch_and_dispatch` resolves a key through the chord map built from `Bindings.xml`, runs the bound
command's **Lua body in the UI VM** (`MoveForwardStart()`), and the VM's `BindingInput` calls become
the held and fired inputs in `BindingsState`. Without the UI VM no key moves anything. The mouse is
the exception: `camera::latch_world_mouse` reads the raw buttons.

**The UI** (FrameXML, the Lua VM) loads at `OnEnter(InWorld)` unless `run_mode::CaptureMode` is
present (`ui_script::lifecycle::ui_wanted`).

**Offline.** `NetPlugin { connect: false }` creates every channel and no thread: sends fail
silently, the event stream stays empty, `NetOffline` is inserted. This is how captures run.

## Findings: how Survivors ran server-less

(The code is gone; `git show bc37b7d:crates/benilla-app/src/survivors/…` has it.)

- `run_survivors` set a flag; `launch` then called `world_map::set_start_map(1)`, built
  `GamePlugins { connect: false, start: InWorld }` and added `SurvivorsPlugin`.
- `SurvivorsPlugin` inserted **`CaptureMode`**. That one resource turned off the player UI *and* the
  engine's controller (`control`), camera shake, the self-avatar fade and the posture/ride systems.
- Units were spawned as `Guid + NetEntity + ObjectStore + MovementState + ScriptedFacing`
  (`units.rs`), dressed as class trainers (`CreatureDisplayInfo` ids, race and sex read back from
  the catalog), weapons in the virtual item slots. This is the part the sandbox keeps.
- Movement was its own: `hero.rs::drive_hero` wrote the hero's `Transform` from `WASD` (world-relative
  in the locked view), clamped to the terrain with `terrain_height_under`, with no collision against
  buildings or water, and set `Player.active/pos` every frame so the world streamed. The camera was
  its own too (`place_camera`): **locked top-down at 61 degrees by default, the hero turning to face
  the mouse cursor**, with `C` toggling an orbit mode in which a drag swings it.
- The loading screen cleared on its own backstop ("focus not resident").

That explains the report that the camera looked wrong and the controls did not respond: the default
view is a fixed overhead angle with world-relative `WASD`, the mouse drag does nothing until `C` is
pressed, there is no pointer lock, and none of the standard WoW controls exist. It could not be
reproduced here (no GPU, no game data); this is the code reading.

## Design

**Reuse the engine's controller and camera, fake only what a server would have provided.** Three
seams, each small:

1. **No connection** (`launch`): `connect: false`, start state `InWorld`, `set_start_map`.
2. **The avatar is spawned instead of streamed** (`sandbox/player.rs`). The same recipe as
   Survivors' units (`sandbox/units.rs`), so the dress/animate/weapon systems are the proven path;
   `SelfGuid` is set to its guid and everything from there is stock: tagging, embodiment, seize,
   settle, camera seat. The seize needs the avatar to exist, the avatar needs the creature
   catalog (to find a trainer look for the race and sex), so it appears a moment after boot.
3. **Keys without the UI VM** (`sandbox/input.rs`). The sandbox does not load FrameXML
   (`SandboxMode` is a new reason `ui_wanted` says no, beside `CaptureMode`, and a server-fed UI
   would be half-empty and error-prone). Its Lua is what turns keys into inputs, so the stock
   1.12 default key map is pressed into `BindingsState` through a new `feed_inputs` method, the
   same entry the Lua calls use. The mouse needs nothing.

Smaller edits: `LoadingScreen::raise_for_sandbox` / `sandbox_avatar_placed` (there is no login to
raise the cover, and until the avatar is seized the stream follows the camera, which `park_camera`
keeps over the start); `ZoomLimit` widened to the 1.12 slider's 30 yards; the light pinned to
`?time=` (no server clock).

**Why not adapt `hero.rs` plus a new orbit camera.** Possible, and lower risk of a *startup*
problem, but it would re-implement collision, step-up, slopes, swimming, jump arcs, the camera's
water and wall handling and its auto-follow, none of which the removed mode had in full, and every
game would inherit a controller that differs from the one the engine is verified against. The
engine's controller already runs without a server (captures run it offline); what it needed was a
body, a start state and its keys.

**Why not feed synthetic `SessionEvent`s through the real net handlers** (fake `SMSG_UPDATE_OBJECT`
and `SMSG_LOGIN_VERIFY_WORLD`): the most faithful, and the most code, and the protocol-side
descriptor layout for a *player* (equipment slots, `PLAYER_BYTES`, item queries that gate
`equipment.settled`) is exactly what cannot be tested here. A unit-kind avatar skips all of it.

**Why the avatar is a Unit, not a Player.** A `Player`-kind entity attaches only once its
equipment has "settled", which waits on item-template queries a server answers. A trainer-dressed
`Unit` attaches at once, which Survivors had already proven in the browser.

## Writing a game

A game is a Bevy plugin. `crates/sandbox` is the place:

- `crates/sandbox/src/games/mod.rs`: `extend(app)` adds every game. Both launchers call it (the
  native `sandbox` binary, and `wenilla` for the browser), so a game runs in both.
- `crates/sandbox/src/games/example.rs`: a runnable example (`?example=1`): reads `T` and
  `Backspace`, reads the character's position and facing, asks the world for the ground and spawns
  and removes local units. Copy it.
- `benilla_app::sandbox`'s module docs (`crates/benilla-app/src/sandbox/mod.rs`) list what a game can
  reach: `SandboxPlayer` (the character's entity), `spawn_unit` / `UnitSpec` (local creatures and
  characters), `WorldPoint` (terrain height), `WorldStage` (system ordering), `wow_to_bevy` /
  `bevy_to_wow`, `SandboxParams` (where the run started), `SandboxMode`.

Game rules, HUD and content are the game's. The engine's UI is off; for text and widgets use Bevy
UI (`sandbox/hud.rs` is a minimal example in the client's font, `crate::char_select::wow_font`).

## Risks and what was not verified

The sandbox was written without a GPU or a WoW install: it compiles for native and wasm and its
pure logic is unit-tested, but it has **not been run**. Things to look at first, most likely first:

1. **The avatar never appears** (loading screen up forever, or an empty world). Needs the creature
   catalog (`Creatures`), then a class-trainer display that resolves for the race and sex. Log
   lines: `sandbox: character spawned …` or `sandbox: no class-N trainer look …`. If the cover
   never lifts, `loading screen: waiting …` lines say what it waits on.
2. **Keys do nothing.** The injection is `sandbox/input.rs`; it runs after the dispatch pass
   (`bindings::BindingSet`) and before `WorldStage::Input`. Add a log in `feed_bindings` to see
   what it presses; if it presses and nothing moves, look at `Player.active`/`settling`.
3. **The camera is not behind the character.** Seize sets camera yaw to the character's facing
   (`?facing=`), pitch to the login pitch. If `settling` never ends the controller holds the body
   with gravity off: `release_post_snap_hold` ends it when the tile under the body is resident.
4. **Falls through the world / floats.** The start height is `?z=` (default 84.5, a yard over
   Northshire's ground) or the terrain height plus a yard when `z` is omitted. Terrain height
   comes from the heightfield only: a start inside a building needs an explicit `z`.
5. **A race and class with no trainer look** stands in with another look (logged).
6. **FrameXML-dependent engine features are silent**: targeting frames, chat, the minimap, action
   bars and the world map do not exist; clicking a creature does target it internally.
7. The mouse look needs pointer lock in the browser, as in normal play; the first click may be
   needed to focus the page, and Chrome on Linux needs `--enable-unsafe-webgpu` for the world.

## What was removed

Azeroth Survivors, entirely: `crates/benilla-app/src/survivors/` (game, HUD, menus, minimap,
cards, enemies, abilities), the `warcraft-survivors` launcher and its Cargo.lock entry, the two
post-process shaders (`survivors_grade.wgsl`, `survivors_fill.wgsl`) and their embedded-asset
registration, `run_survivors` and the `survivors::active()` / start-map override in `lib.rs`, the
`cargo survivors` alias, the Makefile's `native` target (now `-p wow-sandbox`), the survivors tips
and title in `web/boot.js` and `web/index.html`, the screenshot script's autopilot URL
(`scripts/ci/shoot.mjs`) and the benilla-formats `arena_scout` example (it only picked Survivors
arena centres). Aethus's game design is credited in `NOTICE.md` and `README.md`; none of his
game's code remains.

Every remaining occurrence of the word "survivor(s)" in the tree, by file, and why it stays:

| where | why |
|---|---|
| `NOTICE.md`, `README.md` | provenance and credit: the engine came from `azeroth-survivors-re`, whose game code was removed |
| `AGENTS.md` (header note) | the imported file's note that it describes the upstream repository |
| `docs/SANDBOX.md` (this file) | the history above |
| `crates/benilla-world/src/particles.rs`, `.../collider/web_budget.rs`, `.../static_gx/mod.rs`, `.../terrain_stream.rs`, `.../rig_palette.rs`, `crates/benilla-protocol/.../probes/aura.rs`, `crates/benilla-app/src/{ui_party,ui_chat,creature_anim,addon_harness,ui_models,ui_script}/…` | the English word (particles that survive, the remaining party member, an expected survivor in a test), nothing to do with the game |
