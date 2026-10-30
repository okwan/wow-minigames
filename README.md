# wow-minigames

The base for our own World of Warcraft mini-games: an open-source, from-scratch WoW 1.12.1 client
engine (Rust + [Bevy](https://bevy.org)) that also compiles to WebAssembly, so a game runs in the
browser. **No server, no login, no account.** The engine is the product here; the games are ours to
write on top of it.

What you get today is the **sandbox**: the real world (Eastern Kingdoms and Kalimdor), one local
character of any race, gender and class standing in it, and the engine's own third-person controls
and camera: run, strafe, jump, swim, right-drag to steer, left-drag to orbit, wheel to zoom. No game
rules, no quests, no enemies. A game is a Bevy plugin added in one place
([`crates/sandbox/src/games`](crates/sandbox/src/games/mod.rs)); how is in
[docs/SANDBOX.md](docs/SANDBOX.md).

## Plan

- **Phase 1: the engine and the sandbox.** Import the engine, run it in the browser against a WoW
  1.12.1 client you supply, drop everything that needed a server, and give games a place to start.
  This repository is at this stage.
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
- **Azeroth Survivors:** the imported repository also carried *Azeroth Survivors*, a free fan game by
  **Aethus** ([wow.aethus.pl](https://wow.aethus.pl/)). Its game design is theirs. **We removed that
  game from this repository**: its game logic, HUD and content are gone (two small engine-plumbing
  pieces were kept and adapted, listed in [NOTICE.md](NOTICE.md)), and it is not what this project
  ships. It
  is credited because the engine came to us through it. See [NOTICE.md](NOTICE.md).
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

Then `make start` serves it, and `http://127.0.0.1:8090/` opens straight into the sandbox: a loading
screen, then your character in Northshire. `?race=orc&class=warrior&map=1&x=-950&y=-2700` picks who
and where (every parameter is in [docs/SANDBOX.md](docs/SANDBOX.md)). Sound is muted by default:
`?mute=0` turns it on, or press `M` in game. `make native` runs the same thing as a desktop window. Run `make` (or `make help`) to list every target. Controls are in
[CONTROLS.md](CONTROLS.md).

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

## CI

`.github/workflows/wenilla.yml` downloads the client data (cached in the Actions cache, never
committed), builds the wasm client, runs the host tests, and drives the client in headless Chromium
for screenshots (uploaded as the `ci-out` artifact). Repository variables (Settings > Secrets and
variables > Actions > Variables):

- `WOW_DATA_URLS`: optional whitespace/newline separated list of mirror URLs for the 1.12.1 client
  zip, tried in order (default: the archive.org item). Same variable works locally with
  `scripts/fetch-data.sh`.
- `PAGES_ENABLED`: any non-empty value turns on the GitHub Pages deploy job (client shell only; no
  game data is ever published, so the world does not load there). The site is marked `noindex`.
  Leave it unset to skip the deploy; the `site` artifact is still built for inspection.

## Legal

This project is not affiliated with or endorsed by Blizzard Entertainment, and it ships no Blizzard
game files. World of Warcraft is a trademark of Blizzard Entertainment, Inc.; all game art, sounds
and data you load at runtime from your own client are © Blizzard Entertainment. The engine code in
this repository is licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your
option; the vendored components under `third_party/` keep their own upstream licenses.
