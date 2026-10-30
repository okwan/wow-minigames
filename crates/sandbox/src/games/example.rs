//! A worked example of the three things every game does, with no game in it: act on the character,
//! read input, spawn local units. Start the sandbox with `?example=1` (browser) or
//! `WOW_SANDBOX_EXAMPLE=1` (native); then
//!
//! - **`T`** stands a friendly Mage trainer six yards in front of the character, facing it;
//! - **`Backspace`** removes the units `T` made;
//! - the log says where the character stood when it appeared.
//!
//! Copy this file, rename the plugin, and replace the systems.

use benilla_app::{
    bevy_to_wow, bytes0, spawn_unit, SandboxEntity, SandboxPlayer, UnitSpec, WorldPoint,
    WorldStage, FACTION_FRIENDLY,
};
use bevy::prelude::*;

/// Is the example asked for? Through `benilla_app::webenv`, which is `std::env::var` natively and
/// the page's query string in a browser, so one switch works on both.
pub fn enabled() -> bool {
    ["WOW_SANDBOX_EXAMPLE", "WOW_EXAMPLE"]
        .into_iter()
        .any(|k| benilla_app::webenv::var(k).is_some_and(|v| v != "0"))
}

/// The units this example spawned, so `Backspace` removes exactly those and not the character.
#[derive(Component)]
struct Dummy;

/// `CreatureDisplayInfo` 1294: a Human Mage trainer.
const TRAINER_DISPLAY: u32 = 1294;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            // After the controller, so `Transform` is this frame's pose.
            (greet, spawn_dummy, clear_dummies).after(WorldStage::Input),
        );
    }
}

/// The character appears a moment after the loading screen goes up (once the creature data is
/// in), not at startup: `Added` is how a system waits for it.
fn greet(arrived: Query<&Transform, Added<SandboxPlayer>>) {
    for t in &arrived {
        let wow = bevy_to_wow(t.translation);
        info!("example: the character appeared at {wow:?}");
    }
}

/// Read input, read where the character stands and faces, ask the world for the ground, spawn.
fn spawn_dummy(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    me: Query<&Transform, With<SandboxPlayer>>,
    world: WorldPoint,
    mut serial: Local<u64>,
) {
    if !keys.just_pressed(KeyCode::KeyT) {
        return;
    }
    let Ok(me) = me.single() else { return };
    // A model faces -Z at rotation zero, so "in front" is the rotation applied to -Z.
    let mut at = me.translation + me.rotation * Vec3::NEG_Z * 6.0;
    // Local units stand where they are put: nothing grounds them, so ask the world.
    if let Some(ground) = world.terrain_height_under(at) {
        at.y = ground;
    }
    *serial += 1;
    let e = spawn_unit(
        &mut commands,
        &UnitSpec {
            // Unique among local units; 1 is the character's.
            serial: 1000 + *serial,
            display: TRAINER_DISPLAY,
            scale: 1.0,
            level: 60,
            faction: FACTION_FRIENDLY,
            bytes0: bytes0(1, 8, 0, 0),
            weapons: &[],
            sheath: 0,
        },
        at,
        // Turned around to face the character.
        me.rotation.to_euler(EulerRot::YXZ).0 + std::f32::consts::PI,
    );
    commands.entity(e).insert(Dummy);
}

fn clear_dummies(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    dummies: Query<Entity, (With<Dummy>, With<SandboxEntity>)>,
) {
    if keys.just_pressed(KeyCode::Backspace) {
        for e in &dummies {
            commands.entity(e).despawn();
        }
    }
}
