# Notice: provenance of the imported engine

The engine code in this repository was imported on **2026-10-29** from:

- **Upstream repository:** https://github.com/OutThisLife/azeroth-survivors-re
- **Commit:** `72048f4056e2a57d99234e4fb658179e577d1d92`
- **Import commit message:** "Import engine from OutThisLife/azeroth-survivors-re@72048f4"

The whole source tree was copied except `.git` and `docs/screenshots/` (JPG screenshots of
Blizzard art, removed deliberately). No Blizzard files are committed here.

## What was removed

The upstream repository is the engine plus the *Azeroth Survivors* game built on it
(`crates/benilla-app/src/survivors/` and the `warcraft-survivors` launcher). **That game's code was
removed from this repository** after the import: this project is the engine as a base for our own
games, and ships none of Aethus's game. Two small pieces of it were kept and adapted because they
are engine plumbing, not game: how a local unit is spawned so the stock attach and animation systems
dress it (`crates/benilla-app/src/sandbox/units.rs`), and the class-trainer look and weapon tables
that dress a character offline (`.../sandbox/looks.rs`). The game stays in the history (the import
commit and the commits before the removal) and is credited below because the engine reached us
through it. What replaced it is documented in [docs/SANDBOX.md](docs/SANDBOX.md).

## Credited authors and projects

- [OutThisLife/azeroth-survivors-re](https://github.com/OutThisLife/azeroth-survivors-re): the
  repository imported here.
- [samwhosung/benilla](https://github.com/samwhosung/benilla): the from-scratch WoW 1.12.1 client
  (Rust + Bevy) the engine is built on.
- [Arnesen/wenilla](https://github.com/Arnesen/wenilla): the WebAssembly build of benilla ("wenilla"),
  with a local host and realm service.
- **Aethus** ([wow.aethus.pl](https://wow.aethus.pl/)): author of the *Azeroth Survivors* fan game,
  whose design the upstream repository implements.

## Licenses

Engine code: MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`). Components in `third_party/`
keep their own upstream licenses (`third_party/*/LICENSE*`). Those files are unmodified.

World of Warcraft is a trademark of Blizzard Entertainment, Inc. This project is not affiliated
with Blizzard.
