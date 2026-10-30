//! Our games, on the sandbox. [`games::extend`] is the one hook: both launchers (the native
//! `sandbox` binary and the browser's `wenilla`) hand it to `benilla_app::run_sandbox_with`, so a
//! game added there runs in both.
//!
//! The client is built and the sandbox character spawned before `extend` runs; see
//! `benilla_app`'s `sandbox` module docs for what a game can reach.

pub mod games;
