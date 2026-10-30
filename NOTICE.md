# Notice: provenance of the imported engine

The engine code in this repository was imported on **2026-10-29** from:

- **Upstream repository:** https://github.com/OutThisLife/azeroth-survivors-re
- **Commit:** `72048f4056e2a57d99234e4fb658179e577d1d92`
- **Import commit message:** "Import engine from OutThisLife/azeroth-survivors-re@72048f4"

The whole source tree was copied except `.git` and `docs/screenshots/` (JPG screenshots of
Blizzard art, removed deliberately). No Blizzard files are committed here.

## Credited authors and projects

- [OutThisLife/azeroth-survivors-re](https://github.com/OutThisLife/azeroth-survivors-re): the
  repository imported here.
- [samwhosung/benilla](https://github.com/samwhosung/benilla): the from-scratch WoW 1.12.1 client
  (Rust + Bevy) the engine is built on.
- [Arnesen/wenilla](https://github.com/Arnesen/wenilla): the WebAssembly build of benilla ("wenilla"),
  with a local host and realm service.
- **Aethus** ([wow.aethus.pl](https://wow.aethus.pl/)): author of the *Azeroth Survivors* fan game,
  whose design the upstream repository implements. It is not what this project ships.

## Licenses

Engine code: MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`). Components in `third_party/`
keep their own upstream licenses (`third_party/*/LICENSE*`). Those files are unmodified.

World of Warcraft is a trademark of Blizzard Entertainment, Inc. This project is not affiliated
with Blizzard.
