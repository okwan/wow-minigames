//! Sandbox mode: the engine as a base for games, with no server, no login and no game rules.
//!
//! [`crate::run_sandbox`] boots the client straight into the world on the map and place
//! [`SandboxParams`] names, with one local character standing there that the **engine's own
//! player controller and camera** drive: the 1.12 third-person feel (right-drag steers, left-drag
//! orbits, the wheel zooms, the camera swings back behind a moving character), collision against
//! terrain, buildings and water, the walk, run, backpedal, strafe, jump, fall and swim
//! animations. Nothing here re-implements movement; the only seams cut into the client are small:
//!
//! - **no connection** ([`crate::net::NetOffline`]): the net channels exist and every send is a
//!   no-op, so the controller's `MSG_MOVE_*` stream goes nowhere;
//! - **the avatar is spawned, not streamed** ([`player`]): a unit built exactly as the net layer
//!   builds one ([`units::spawn_unit`]) and named as ours through [`crate::net::SelfGuid`], after
//!   which the stock embodiment, seize-control, settle-on-ground and camera-seat run unchanged;
//! - **keys reach the controller without the UI VM** ([`input`]): the client's player UI is not
//!   loaded ([`SandboxMode`]), and the UI's Lua is what turns keys into movement inputs, so the
//!   stock key map is pressed directly into `BindingsState`.
//!
//! # Start-up parameters
//!
//! In a browser they are the query string, natively `WOW_SANDBOX_<NAME>` variables (see
//! [`params`]):
//!
//! | `?param` | `WOW_SANDBOX_` | meaning | default |
//! |---|---|---|---|
//! | `map` | `MAP` | `Map.dbc` id, or `eastern` / `kalimdor` | `0` |
//! | `x`, `y`, `z` | `X`, `Y`, `Z` | WoW coordinates; leave `z` out to stand on the ground | Northshire Abbey |
//! | `facing` | `FACING` | orientation in radians, or `90deg` | `0` |
//! | `race` | `RACE` | `human`, `orc`, `dwarf`, `nightelf`, `undead`, `tauren`, `gnome`, `troll`, or 1-8 | `human` |
//! | `gender` | `GENDER` | `male` / `female` | `male` |
//! | `class` | `CLASS` | `warrior` … `druid`, or the class id | `mage` |
//! | `time` | `TIME` | the light's time of day, `HH:MM` | `13:00` |
//!
//! # Writing a game on top
//!
//! A game is a Bevy plugin. Hand it to [`crate::run_sandbox_with`] from a launcher (the `sandbox`
//! crate's `games` module is the place this repository keeps them, shared by the native and the
//! browser launcher):
//!
//! ```ignore
//! fn main() -> benilla_app::AppExit {
//!     benilla_app::run_sandbox_with(build_id(), |app| {
//!         app.add_plugins(MyGame);
//!     })
//! }
//! ```
//!
//! Inside the plugin:
//!
//! - **Systems** go in `Update`. Run them `.in_set(WorldStage::Input)` (re-exported here) to act
//!   in the same stage as the controller, `.after(WorldStage::Input)` to see this frame's
//!   movement.
//! - **The character** is the entity with [`SandboxPlayer`]; its `Transform` is where it stands
//!   (Bevy space; [`wow_to_bevy`] and [`bevy_to_wow`] convert), its rotation's Y angle is its
//!   facing. `Res<crate::player::Player>` is the controller's state, not public: read the
//!   transform. Wait for the entity to exist (`Added<SandboxPlayer>`): it appears once the
//!   creature data has loaded, a moment after the loading screen goes up.
//! - **Input** is Bevy's: `Res<ButtonInput<KeyCode>>`, `ButtonInput<MouseButton>`. The movement
//!   keys, `Space`, `Z`, `X` and `Num Lock` already mean something ([`input`]); everything else
//!   is free. Mouse buttons are the camera's: a click that is not a drag is a world click.
//! - **Local units**: [`spawn_unit`] makes a creature or another character from a display id and
//!   it gets its model, animations, weapons and shadows from the stock systems. Despawn it like
//!   any entity. Queries for them use [`SandboxEntity`].
//! - **The ground**: [`WorldPoint`] is a `SystemParam` answering `terrain_height_under`.
//!
//! The `sandbox` crate's `games/example.rs` is a runnable version of all of the above.

mod hud;
mod input;
mod looks;
mod params;
mod player;
mod units;

use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;

pub use benilla_assets::coords::{bevy_to_wow, wow_to_bevy};
pub use benilla_world::schedule::WorldStage;
pub use benilla_world::world_point::WorldPoint;
pub use looks::Weapon;
pub use params::SandboxParams;
pub use player::SandboxPlayer;
pub use units::{
    bytes0, creature_guid, spawn_unit, SandboxEntity, UnitSpec, FACTION_FRIENDLY, FACTION_MONSTER,
};

/// Set by [`crate::run_sandbox`] before the app builds. wasm32 has no process environment, so the
/// flag lives here; `WOW_SANDBOX` (`?sandbox=1` on the web) still turns it on from outside.
pub(crate) static ACTIVE: AtomicBool = AtomicBool::new(false);

pub(crate) fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed) || crate::webenv::var("WOW_SANDBOX").is_some()
}

/// Present in a sandbox run: the client has no server and no player UI. Anything that would wait
/// for either can ask for this resource.
#[derive(Resource)]
pub struct SandboxMode;

/// The sandbox, as one plugin: the local character, its input, the start-up cover and the help
/// line. [`crate::run_sandbox`] adds it; add it yourself only to a custom launcher.
pub struct SandboxPlugin {
    pub params: SandboxParams,
}

impl SandboxPlugin {
    pub fn new(params: SandboxParams) -> Self {
        Self { params }
    }

    /// With the parameters this process was started with ([`SandboxParams::from_env`]).
    pub fn from_env() -> Self {
        Self::new(SandboxParams::from_env())
    }
}

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SandboxMode)
            .insert_resource(self.params.clone())
            .add_systems(
                Startup,
                (
                    player::raise_cover,
                    hud::setup,
                    player::widen_zoom.after(crate::cvars::CvarLoad),
                ),
            )
            .add_systems(
                Update,
                (
                    // Keys into the controller's inputs: after the dispatch pass cleared the
                    // frame's edges, before the controller reads them.
                    input::feed_bindings
                        .after(crate::bindings::BindingSet)
                        .before(WorldStage::Input)
                        .in_set(crate::char_select::InWorldGated),
                    // Until the controller owns the camera, keep it over the start, where the
                    // world streams in.
                    player::park_camera
                        .in_set(WorldStage::Input)
                        .after(crate::player::PlayerControlSet)
                        .run_if(|p: Res<crate::player::Player>| !p.active),
                    player::spawn_player,
                    player::release_cover
                        .after(WorldStage::Input)
                        .before(WorldStage::Present),
                    player::hold_time_of_day,
                    hud::update,
                ),
            );
    }
}
