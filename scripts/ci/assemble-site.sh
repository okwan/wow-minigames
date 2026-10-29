#!/usr/bin/env bash
#
# assemble-site.sh - build the GitHub Pages payload `site/` from `web/dist/` (client only).
#
#   scripts/ci/assemble-site.sh [dist-dir] [site-dir]      (defaults: web/dist, site)
#
# NOTE / TODO (deliberate omission): the world needs the WoW 1.12.1 client's MPQ data, served as
# /data/* by wenilla-host. Serving that from Pages would mean redistributing Blizzard's assets
# from a public site, so this script NEVER copies game data and the deploy does not upload any.
# The Pages copy is therefore the client shell only; it cannot load the world on its own, and
# the page says so (a small notice injected below). Do not "fix" this by adding data here.
#
# The site is meant to stay unlisted: noindex meta + /robots.txt "Disallow: /" (Pages cannot set
# an X-Robots-Tag header). Do not link the deployed URL anywhere in this repo.
#
# Precompressed siblings (*.br, *.gz) are left out: they exist for wenilla-host's
# precompressed_br(), and Pages would serve them as opaque files, not as Content-Encoding
# variants. The plain .wasm is shipped (Pages compresses text-like types on the fly at its
# discretion); it must stay under Pages' 100 MB-per-file limit, which this script checks.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

DIST="${1:-web/dist}"
SITE="${2:-site}"
LIMIT=$((100 * 1000 * 1000))   # Pages: 100 MB per file (decimal MB, conservative)

[ -f "$DIST/index.html" ] || { echo "assemble-site: $DIST/index.html missing — run scripts/web-build.sh" >&2; exit 1; }

rm -rf "$SITE"
mkdir -p "$SITE"
# Everything from dist except the precompressed siblings, and defensively any game data.
(cd "$DIST" && find . -type f \
  ! -name '*.br' ! -name '*.gz' ! -iname '*.mpq' ! -path './data/*' ! -path './Data/*' -print0 \
  | tar --null -T - -cf -) | tar -xf - -C "$SITE"

printf 'User-agent: *\nDisallow: /\n' > "$SITE/robots.txt"
: > "$SITE/.nojekyll"

# Inject the noindex meta and a small notice into index.html.
python3 - "$SITE/index.html" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
meta = '<meta name="robots" content="noindex,nofollow">\n'
notice = (
    '<div id="pages-notice" style="position:fixed;left:8px;bottom:8px;z-index:99999;'
    'max-width:34em;padding:6px 10px;font:12px/1.4 sans-serif;color:#ddd;background:rgba(0,0,0,.75);'
    'border:1px solid #555;border-radius:4px">'
    'Client build only: game data is not hosted on this site, so the world cannot load here. '
    'Run it locally with your own WoW 1.12.1 client data.</div>\n'
)
marker = '<meta charset="utf-8">'
if marker in s:
    s = s.replace(marker, marker + "\n" + meta, 1)
else:
    s = meta + s
low = s.lower()
i = low.rfind("</body>")
s = s[:i] + notice + s[i:] if i != -1 else s + "\n" + notice
open(p, "w", encoding="utf-8").write(s)
PY
grep -q 'name="robots" content="noindex,nofollow"' "$SITE/index.html" || { echo "assemble-site: noindex injection failed" >&2; exit 1; }

# Guards: no game data, no oversized files.
if find "$SITE" -iname '*.mpq' | grep -q .; then echo "assemble-site: MPQ found in site — refusing" >&2; exit 1; fi
over=0
while IFS= read -r -d '' f; do
  sz=$(stat -c %s "$f")
  if [ "$sz" -gt "$LIMIT" ]; then echo "assemble-site: $f is $sz bytes, over the Pages 100 MB file limit" >&2; over=1; fi
done < <(find "$SITE" -type f -print0)
[ "$over" = 0 ] || exit 1

echo "site assembled in $SITE:"
( cd "$SITE" && find . -type f -printf '%10s  %p\n' | sort -k2 )
echo "total: $(du -sb "$SITE" | cut -f1) bytes (Pages site limit ~1 GB)"
