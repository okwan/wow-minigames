//! The local character: spawned as a unit, seized by the stock player controller.

use benilla_world::view::WorldCamera;
use bevy::prelude::*;

use super::looks::{self, Match};
use super::params::SandboxParams;
use super::units::{self, UnitSpec};
use crate::net::SelfGuid;

/// The local character. Its `Transform` is where it stands, in Bevy space.
#[derive(Component)]
pub struct SandboxPlayer;

/// The serial of the character's guid.
const PLAYER_SERIAL: u64 = 1;

/// A yard above the ground at a start whose height was not given, so the body settles onto it.
const GROUND_LIFT: f32 = 1.0;

/// Frames to wait for the creature catalog to know the class's trainers before dressing the
/// character in whatever resolves (about fifteen seconds at 60 fps).
const LOOK_PATIENCE: u32 = 900;

/// The ground at the start, once the terrain there has loaded. `None` until then, `Some` at once
/// for a start whose height was given.
fn start_height(params: &SandboxParams, ground: Option<f32>) -> Option<f32> {
    if params.snap_to_ground {
        ground.map(|h| h + GROUND_LIFT)
    } else {
        Some(params.position[2])
    }
}

/// Startup: raise the loading cover. There is no login to raise it, and until the character is
/// seized the stream follows the camera, so the first resident frame would clear it on a world
/// with nobody in it.
pub(super) fn raise_cover(
    mut screen: ResMut<crate::loading_screen::LoadingScreen>,
    params: Res<SandboxParams>,
    time: Res<Time<Real>>,
) {
    screen.raise_for_sandbox(Some(params.map), time.elapsed_secs());
}

/// Startup: allow the full 1.12 zoom range (the client's default 15 yd is the options slider's
/// low end; its slider tops out at twice that).
pub(super) fn widen_zoom(mut limit: ResMut<crate::player::camera_zoom::ZoomLimit>) {
    limit.set_distance_max(30.0);
}

/// Before the character is seized the controller free-flies the camera. Keep it over the start
/// instead, looking down at it, so the stream (which follows the camera until the character is
/// ours) loads the right place.
pub(super) fn park_camera(
    params: Res<SandboxParams>,
    mut cams: Query<&mut Transform, With<WorldCamera>>,
) {
    let at = benilla_assets::coords::wow_to_bevy(params.position);
    let eye = at + Vec3::new(0.0, 6.0, 14.0);
    for mut t in &mut cams {
        *t = Transform::from_translation(eye).looking_at(at + Vec3::Y, Vec3::Y);
    }
}

/// Dress and place the character once the creature catalog is in (and, for a start without a
/// height, the terrain under it). Everything after that is the stock flow a login ends in: the
/// avatar is tagged ours through `SelfGuid`, embodied, seized by the controller, held on its
/// settle until the ground is resident.
pub(super) fn spawn_player(
    mut commands: Commands,
    params: Res<SandboxParams>,
    creatures: Option<Res<crate::entities::Creatures>>,
    world: benilla_world::world_point::WorldPoint,
    mut self_guid: ResMut<SelfGuid>,
    mut waited: Local<u32>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Some(creatures) = creatures else {
        return;
    };
    let Some(class) = looks::class_look(params.class) else {
        // Unreachable through `params::parse`, which only yields known classes.
        error!("sandbox: no look for class {}", params.class);
        *done = true;
        return;
    };
    let at = benilla_assets::coords::wow_to_bevy([params.position[0], params.position[1], 0.0]);
    let Some(z) = start_height(&params, world.terrain_height_under(at)) else {
        return;
    };

    *waited += 1;
    let chosen = looks::choose(class, params.race, params.gender, |d| {
        creatures.display_race_sex(d)
    });
    let (look, race, sex) = match chosen {
        Some((look, race, sex, how)) => {
            if how != Match::Exact {
                warn!(
                    "sandbox: no class-{} trainer look for race {} sex {}; wearing display {} \
                     (race {}, sex {}, {:?})",
                    params.class, params.race, params.gender, look, race, sex, how
                );
            }
            (look, race, sex)
        }
        None if *waited < LOOK_PATIENCE => return,
        None => {
            warn!(
                "sandbox: the creature catalog knows none of class {}'s trainer displays; \
                 using display {} unchecked",
                params.class, class.trainers[0]
            );
            (class.trainers[0], params.race, params.gender)
        }
    };

    let pos = benilla_assets::coords::wow_to_bevy([params.position[0], params.position[1], z]);
    let e = units::spawn_unit(
        &mut commands,
        &UnitSpec {
            serial: PLAYER_SERIAL,
            display: look,
            scale: 1.0,
            level: 60,
            faction: units::FACTION_FRIENDLY,
            bytes0: units::bytes0(race, class.id, sex, class.power),
            weapons: class.weapons,
            sheath: class.sheath,
        },
        pos,
        params.facing,
    );
    commands.entity(e).insert(SandboxPlayer);
    self_guid.0 = Some(units::creature_guid(PLAYER_SERIAL));
    info!(
        "sandbox: character spawned at {:?} facing {:.2} (display {look}, race {race}, \
         gender {sex}, class {})",
        params.position, params.facing, class.id
    );
    *done = true;
}

/// Lift the loading cover's wait once the controller has seized the character. From there the
/// stock rules clear it: the cover holds while the body is settling, until its ground is resident.
pub(super) fn release_cover(
    mut screen: ResMut<crate::loading_screen::LoadingScreen>,
    player: Res<crate::player::Player>,
    me: Query<(), With<SandboxPlayer>>,
    mut released: Local<bool>,
) {
    if !*released && player.active && !me.is_empty() {
        screen.sandbox_avatar_placed();
        *released = true;
    }
}

/// Pin the light to the start's time of day: with no server there is no game clock to follow.
pub(super) fn hold_time_of_day(
    params: Res<SandboxParams>,
    debug: Option<ResMut<benilla_world::dev_state::DebugState>>,
) {
    let Some(mut debug) = debug else { return };
    if debug.lighting.follow_server_time || debug.lighting.manual_minute != params.minute {
        debug.lighting.follow_server_time = false;
        debug.lighting.manual_minute = params.minute;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_given_height_is_used_as_is() {
        let p = SandboxParams::default();
        assert_eq!(start_height(&p, None), Some(84.5));
        assert_eq!(start_height(&p, Some(10.0)), Some(84.5));
    }

    #[test]
    fn an_unknown_height_waits_for_the_ground_then_stands_a_yard_above_it() {
        let p = SandboxParams {
            snap_to_ground: true,
            ..SandboxParams::default()
        };
        assert_eq!(start_height(&p, None), None);
        assert_eq!(start_height(&p, Some(92.0)), Some(93.0));
    }
}
