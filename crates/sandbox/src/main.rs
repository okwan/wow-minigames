//! The native sandbox launcher: the build stamp, handed to the sandbox with our games attached.
//!
//! ```text
//! WOW_DATA=wow-data cargo run --release -p wow-sandbox        # or: make native
//! WOW_SANDBOX_RACE=orc WOW_SANDBOX_CLASS=warrior WOW_SANDBOX_MAP=1 \
//!   WOW_SANDBOX_X=-950 WOW_SANDBOX_Y=-2700 cargo run --release -p wow-sandbox
//! ```

use benilla_app::BuildId;

fn main() -> benilla_app::AppExit {
    benilla_app::run_sandbox_with(
        BuildId {
            version: env!("CARGO_PKG_VERSION"),
            describe: env!("BENILLA_GIT_DESCRIBE"),
            sha: env!("BENILLA_GIT_SHA"),
            short: env!("BENILLA_GIT_SHORT"),
            date: env!("BENILLA_GIT_DATE"),
            profile: env!("BENILLA_PROFILE"),
            project_dir: env!("BENILLA_PROJECT_DIR"),
            ..Default::default()
        },
        wow_sandbox::games::extend,
    )
}
