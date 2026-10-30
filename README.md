# wow-minigames

A collection of small World of Warcraft-universe mini-games, built on an open-source, from-scratch
WoW 1.12.1 client engine (Rust + [Bevy](https://bevy.org)) that also compiles to WebAssembly so the
games run in the browser. **No server, no login, no account.**

## Plan

- **Phase 1: proof of concept.** Import the engine and run it in the browser, locally, against a
  WoW 1.12.1 client you supply. This repository is currently at this stage: the engine is imported
  as-is (see [NOTICE.md](NOTICE.md)), and builds and runs the way it did upstream.
- **Phase 2: our own games.** Mini-games made for this project, for example Darkmoon Faire
  mini-games, a Gnomeregan jumping game, and fishing.

## Credits and licenses

**Please read this section before reusing anything here.**

- **Engine:** imported from
  [OutThisLife/azeroth-survivors-re](https://github.com/OutThisLife/azeroth-survivors-re) at commit
  [`72048f4056e2a57d99234e4fb658179e577d1d92`](https://github.com/OutThisLife/azeroth-survivors-re/commit/72048f4056e2a57d99234e4fb658179e577d1d92).
  That repository builds on two upstream projects:
  - [samwhosung/benilla](https://github.com/samwhosung/benilla): the from-scratch WoW 1.12.1 client.
  - [Arnesen/wenilla](https://github.com/Arnesen/wenilla): benilla compiled to WebAssembly, with a
    local host and realm service.
- **Survivors game design:** *Azeroth Survivors* is a free fan game by **Aethus**
  ([wow.aethus.pl](https://wow.aethus.pl/)). Its game design is theirs and is **not** what this
  project ships. The survivors code that came along with the engine import is not a product of
  this project.
- **Licenses:** engine code is licensed under [MIT](LICENSE-MIT) **or** [Apache 2.0](LICENSE-APACHE),
  at your option. Vendored components under `third_party/` keep their own upstream licenses (see the
  `LICENSE*` file in each directory).

The full provenance record is in [NOTICE.md](NOTICE.md).

## Build and run

```bash
git clone https://github.com/okwan/wow-minigames
cd wow-minigames
make build     # fetch game data + install toolchain + build the wasm (~5.5 GB download)
make start     # then open http://127.0.0.1:8090/
```

`make build` chains three re-runnable scripts, each skipping what's already done:

| Step | What it does |
| --- | --- |
| `scripts/fetch-data.sh` | downloads the WoW 1.12.1 client and extracts its MPQs |
| `scripts/web-setup.sh` | installs the wasm toolchain (wasi-sdk, wasm-bindgen, binaryen) |
| `scripts/web-build.sh` | builds the client into `web/dist/` |

**Prerequisites:** [Rust](https://rustup.rs/), `make`, `curl`, `unzip`, and a bash shell. On Windows,
run everything from Git Bash; PowerShell and cmd can't run the scripts.

Run `make` (or `make help`) to list every target. Controls are in [CONTROLS.md](CONTROLS.md).

## The game data (not in this repo)

This repo ships **no Blizzard content**: no art, models, sounds, maps, MPQ contents, or FrameXML.
The game reads a WoW 1.12.1 client's `Data` folder at runtime, which you supply.

`make build` (via `scripts/fetch-data.sh`) downloads the **English WoW 1.12.1 (build 5875)**
client from [archive.org](https://archive.org/details/WoW_1.12.1-5875) (~5.5 GB) and extracts the
MPQs into `wow-data/Data/` (git-ignored, resumable, skipped when already present). Or point at a
client you already have with `--data` / `WOW_DATA`.

### Changing the download source

If the archive.org item moves or you have your own mirror, override the URL without editing
anything:

```bash
make build WOW_DATA_URL=https://your-mirror.example/client.zip
```

Or set it permanently by editing the single clearly-marked `DEFAULT_DATA_URL` line at the top of
`scripts/fetch-data.sh`. Either way the URL must be a direct link to a zip holding the client's
`Data/*.MPQ` files.

## Legal

This project is not affiliated with or endorsed by Blizzard Entertainment, and it ships no Blizzard
game files. World of Warcraft is a trademark of Blizzard Entertainment, Inc.; all game art, sounds
and data you load at runtime from your own client are © Blizzard Entertainment. The engine code in
this repository is licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your
option; the vendored components under `third_party/` keep their own upstream licenses.
