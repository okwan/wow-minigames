#!/usr/bin/env bash
# Fetch the game data — a World of Warcraft 1.12.1 (build 5875) client's Data folder — from
# archive.org (or a mirror). This repo ships no Blizzard content; the data is downloaded at setup
# time and only the MPQs are extracted.
#
# Re-runnable: skips when the MPQs are already present, and resumes an interrupted download.
# Requires: curl, unzip.
#
# Environment:
#   WOW_DATA_URL    one URL (kept for compatibility; wins over WOW_DATA_URLS)
#   WOW_DATA_URLS   whitespace/newline separated mirror URLs, tried in order
#   WOW_DATA_DIR    where wow-data lives (default: <repo>/wow-data)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA_DIR="${WOW_DATA_DIR:-$ROOT/wow-data}"
DATA="$DATA_DIR/Data"

# --- Data source ---------------------------------------------------------------
# Where the game data is downloaded from. Each URL must be a direct link to a zip that holds the
# client's Data/*.MPQ files. Override per-run with WOW_DATA_URL (one) or WOW_DATA_URLS (a list,
# tried in order), or edit the default below if the archive.org item ever moves.
DEFAULT_DATA_URL="https://archive.org/download/WoW_1.12.1-5875/1.12.1-5875_enUS.zip"
if [ -n "${WOW_DATA_URL:-}" ]; then
    URL_LIST="$WOW_DATA_URL"
elif [ -n "${WOW_DATA_URLS:-}" ]; then
    URL_LIST="$WOW_DATA_URLS"
else
    URL_LIST="$DEFAULT_DATA_URL"
fi
# shellcheck disable=SC2206  # word splitting on whitespace/newlines is the point
URLS=($URL_LIST)
if [ "${#URLS[@]}" -eq 0 ]; then
    echo "fetch-data: no data URL configured" >&2
    exit 1
fi

# The two MPQs the game cannot boot without are the canary: both present = data is in.
if [ -f "$DATA/terrain.MPQ" ] && [ -f "$DATA/patch.MPQ" ]; then
    echo "game data already present at $DATA"
    echo "  --data $DATA"
    exit 0
fi

mkdir -p "$DATA_DIR"
ZIP="$DATA_DIR/wow-client.zip"
TRIED=()

zip_ok() { [ -s "$1" ] && unzip -tq "$1" >/dev/null 2>&1; }

if zip_ok "$ZIP"; then
    echo "using already-downloaded $ZIP"
else
    for url in "${URLS[@]}"; do
        TRIED+=("$url")
        echo "downloading the game data (a WoW 1.12.1 client, ~5.5 GB)…"
        echo "  $url"
        # -C - resumes a partial $ZIP from an earlier attempt or mirror of the same file.
        if curl --fail --location --retry 5 --retry-delay 10 -C - -o "$ZIP" "$url" && zip_ok "$ZIP"; then
            echo "  download verified (valid zip, $(du -h "$ZIP" | cut -f1))"
            break
        fi
        echo "  failed or not a valid zip: $url" >&2
        # A corrupt/partial file from one mirror must not poison resume against the next one.
        rm -f "$ZIP"
    done
fi

if ! zip_ok "$ZIP"; then
    {
        echo
        echo "fetch-data: could not download a valid game-data zip."
        echo "URLs tried, in order:"
        for u in "${TRIED[@]}"; do echo "  - $u"; done
        echo
        echo "What to do:"
        echo "  - Locally: WOW_DATA_URLS='https://mirror-a/x.zip https://mirror-b/x.zip' ./scripts/fetch-data.sh"
        echo "    (or WOW_DATA_URL=<one url>). Each must be a direct link to a zip holding Data/*.MPQ."
        echo "  - In CI: set the repository variable WOW_DATA_URLS (Settings > Secrets and variables >"
        echo "    Actions > Variables) to one or more mirror URLs, whitespace or newline separated."
        echo "  - Or copy a WoW 1.12.1 client's Data/ folder to $DATA yourself."
    } >&2
    rm -f "$ZIP"
    exit 1
fi

echo "extracting MPQs into ${DATA}…"
mkdir -p "$DATA"
unzip -o "$ZIP" 'Data/*.MPQ' -d "$DATA_DIR"

# The MPQs are what the game reads; drop the zip to reclaim ~5.5 GB.
rm -f "$ZIP"

for f in terrain.MPQ patch.MPQ; do
    if [ ! -f "$DATA/$f" ]; then
        echo "fetch-data: extraction finished but $DATA/$f is missing — the zip does not look like a" >&2
        echo "  WoW 1.12.1 client (expected Data/$f inside it, matched case-sensitively)." >&2
        exit 1
    fi
done

echo "MPQ sizes:"
ls -l "$DATA"/*.MPQ

echo "game data ready at $DATA"
echo "  --data $DATA"
