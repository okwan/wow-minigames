//! Where games are registered.
//!
//! A game is a Bevy plugin. Write it in its own module beside [`example`], add it to [`extend`],
//! and both launchers pick it up. Nothing here is a game: with the example off, the sandbox is
//! just a character in the world.

use bevy::prelude::*;

pub mod example;

/// Add every game to the built client. Called once, after the engine and the sandbox are built.
pub fn extend(app: &mut App) {
    // A game of ours:
    //     app.add_plugins(my_game::MyGamePlugin);
    //
    // The example is opt-in (`?example=1`, `WOW_SANDBOX_EXAMPLE=1`), so a plain sandbox stays plain.
    if example::enabled() {
        app.add_plugins(example::ExamplePlugin);
    }
}
