//! Keyboard and wheel input for the engine's own player controller.
//!
//! The controller never reads keys. In the client every key resolves through `Bindings.xml`, whose
//! Lua bodies call `MoveForwardStart()` and friends, and `crate::bindings` turns those calls into
//! the held and fired inputs the controller reads. Sandbox mode loads no UI VM, so no body runs:
//! this system presses the same inputs straight from the keys, through the same entry
//! ([`BindingsState::feed_inputs`]). The mouse needs nothing here: right-drag steering, left-drag
//! orbit and the both-button run are the camera's own latch on the raw buttons.
//!
//! The map is the stock 1.12 default set:
//!
//! | input | key |
//! |---|---|
//! | forward / backward | `W` or `Up` / `S` or `Down` |
//! | turn left / right | `A` or `Left` / `D` or `Right` (strafe instead while the right button is held) |
//! | strafe left / right | `Q` / `E` |
//! | jump | `Space` |
//! | autorun | `Num Lock` |
//! | sit or stand / sheathe | `X` / `Z` |
//! | zoom | wheel |
//! | mute or unmute all sound | `M` |

use benilla_ui::script::{BindingInput, FiredInput, HeldInput};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use crate::bindings::BindingsState;

/// Held inputs and the keys that hold them.
const HELD: &[(HeldInput, &[KeyCode])] = &[
    (HeldInput::MoveForward, &[KeyCode::KeyW, KeyCode::ArrowUp]),
    (
        HeldInput::MoveBackward,
        &[KeyCode::KeyS, KeyCode::ArrowDown],
    ),
    (HeldInput::TurnLeft, &[KeyCode::KeyA, KeyCode::ArrowLeft]),
    (HeldInput::TurnRight, &[KeyCode::KeyD, KeyCode::ArrowRight]),
    (HeldInput::StrafeLeft, &[KeyCode::KeyQ]),
    (HeldInput::StrafeRight, &[KeyCode::KeyE]),
];

/// One-shots and the key whose press edge fires them.
const FIRED: &[(KeyCode, FiredInput)] = &[
    (KeyCode::Space, FiredInput::Jump),
    (KeyCode::NumLock, FiredInput::ToggleAutoRun),
    (KeyCode::KeyX, FiredInput::SitOrStand),
    (KeyCode::KeyZ, FiredInput::ToggleSheath),
];

/// What to tell the binding state this frame, given the keys down (`pressed`), the keys that went
/// down this frame (`just_pressed`), which held inputs are set now (`holds`), whether a modifier
/// is down (browser and OS chords such as `Ctrl+W` are not movement) and the wheel in notches
/// (positive is up, toward the screen).
///
/// Held inputs are reconciled, not edge-detected: a key held through a loading cover (which clears
/// every bit) or a lost-focus release presses again once the cover lifts, and a key that went up
/// where the page never saw it is released.
pub(crate) fn plan(
    pressed: impl Fn(KeyCode) -> bool,
    just_pressed: impl Fn(KeyCode) -> bool,
    holds: impl Fn(HeldInput) -> bool,
    modifier: bool,
    wheel: f32,
) -> Vec<BindingInput> {
    let mut out = Vec::new();
    for &(held, keys) in HELD {
        let want = !modifier && keys.iter().any(|&k| pressed(k));
        match (want, holds(held)) {
            (true, false) => out.push(BindingInput::Start(held)),
            (false, true) => out.push(BindingInput::Stop(held)),
            _ => {}
        }
    }
    if !modifier {
        for &(key, fired) in FIRED {
            if just_pressed(key) {
                out.push(BindingInput::Fire(fired));
            }
        }
    }
    if wheel > 0.0 {
        out.push(BindingInput::Fire(FiredInput::CameraZoomIn(wheel)));
    } else if wheel < 0.0 {
        out.push(BindingInput::Fire(FiredInput::CameraZoomOut(-wheel)));
    }
    out
}

/// `M` flips [`SoundConfig::muted`](crate::sound::SoundConfig), the runtime side of `?mute=`. The
/// sound plugin opens the audio device the first time it is unmuted.
pub(super) fn toggle_mute(
    keys: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<crate::sound::SoundConfig>,
) {
    let modifier = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    if !modifier && keys.just_pressed(KeyCode::KeyM) {
        config.muted = !config.muted;
    }
}

/// Press this frame's inputs. Runs after the dispatch pass, which has just cleared the frame's
/// edges, and before the controller reads them.
pub(super) fn feed_bindings(
    keys: Res<ButtonInput<KeyCode>>,
    scroll: Res<AccumulatedMouseScroll>,
    mut state: ResMut<BindingsState>,
) {
    let modifier = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    let inputs = plan(
        |k| keys.pressed(k),
        |k| keys.just_pressed(k),
        |h| state.holds(h),
        modifier,
        crate::bindings::wheel_lines(scroll.unit, scroll.delta.y),
    );
    if !inputs.is_empty() {
        state.feed_inputs(inputs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nothing(_: KeyCode) -> bool {
        false
    }

    #[test]
    fn a_key_going_down_starts_its_input_once() {
        let down = |k| k == KeyCode::KeyW;
        let started = plan(down, nothing, |_| false, false, 0.0);
        assert_eq!(started, vec![BindingInput::Start(HeldInput::MoveForward)]);
        // Next frame the input is set: nothing to say.
        let held = plan(down, nothing, |h| h == HeldInput::MoveForward, false, 0.0);
        assert!(held.is_empty());
    }

    #[test]
    fn letting_go_stops_it() {
        let stopped = plan(nothing, nothing, |h| h == HeldInput::TurnLeft, false, 0.0);
        assert_eq!(stopped, vec![BindingInput::Stop(HeldInput::TurnLeft)]);
    }

    #[test]
    fn an_arrow_and_a_letter_are_one_input() {
        let both = |k| matches!(k, KeyCode::KeyW | KeyCode::ArrowUp);
        let plan_ = plan(both, nothing, |_| false, false, 0.0);
        assert_eq!(plan_, vec![BindingInput::Start(HeldInput::MoveForward)]);
    }

    #[test]
    fn opposite_keys_press_both_and_the_controller_nets_them() {
        let down = |k| matches!(k, KeyCode::KeyW | KeyCode::KeyS);
        let plan_ = plan(down, nothing, |_| false, false, 0.0);
        assert!(plan_.contains(&BindingInput::Start(HeldInput::MoveForward)));
        assert!(plan_.contains(&BindingInput::Start(HeldInput::MoveBackward)));
    }

    #[test]
    fn jump_fires_on_the_press_edge_only() {
        let edge = |k| k == KeyCode::Space;
        assert_eq!(
            plan(nothing, edge, |_| false, false, 0.0),
            vec![BindingInput::Fire(FiredInput::Jump)]
        );
        // Held but not new: no second jump.
        assert!(plan(edge, nothing, |_| false, false, 0.0).is_empty());
    }

    #[test]
    fn a_modifier_makes_every_key_a_chord_and_releases_what_was_held() {
        let down = |k| k == KeyCode::KeyW;
        let space = |k| k == KeyCode::Space;
        assert!(plan(down, space, |_| false, true, 0.0).is_empty());
        let released = plan(down, nothing, |h| h == HeldInput::MoveForward, true, 0.0);
        assert_eq!(released, vec![BindingInput::Stop(HeldInput::MoveForward)]);
    }

    #[test]
    fn the_wheel_zooms_by_its_notches() {
        assert_eq!(
            plan(nothing, nothing, |_| false, false, 2.0),
            vec![BindingInput::Fire(FiredInput::CameraZoomIn(2.0))]
        );
        assert_eq!(
            plan(nothing, nothing, |_| false, false, -0.5),
            vec![BindingInput::Fire(FiredInput::CameraZoomOut(0.5))]
        );
    }

    /// The system end to end against the real binding state: what the controller reads.
    #[test]
    fn the_system_sets_and_clears_the_inputs_the_controller_reads() {
        use crate::bindings::Input;
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<AccumulatedMouseScroll>()
            .init_resource::<BindingsState>()
            .add_systems(Update, feed_bindings);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.update();
        let state = app.world().resource::<BindingsState>();
        assert!(state.pressed(Input::MoveForward));
        assert!(state.just_pressed(Input::MoveForward));
        assert!(!state.pressed(Input::MoveBackward));

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyW);
        app.update();
        assert!(!app
            .world()
            .resource::<BindingsState>()
            .pressed(Input::MoveForward));
    }
}
