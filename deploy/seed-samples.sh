#!/usr/bin/env bash
# seed-samples.sh — generate test clips and pre-map them into bank A.
#
# Run ON the Pi:
#     cd ~/r_e_c_u_rs && ./seed-samples.sh
#
# Generates 10 GStreamer test-pattern clips into $APP_DIR/samples, then writes
# banks.toml (clips mapped into bank A slots 0-9) and paths.toml (browser root)
# into the app state dir so they're ready to trigger on next launch.
set -euo pipefail

APP_DIR="${APP_DIR:-$(cd "$(dirname "$0")" && pwd)}"
SAMPLES="$APP_DIR/samples"
STATE="$APP_DIR/.config/recur"
SECS="${SECS:-5}"

echo "--- Generating clips into $SAMPLES"
"$APP_DIR/gen-test-clips.sh" "$SAMPLES" "$SECS"

echo "--- Writing state files into $STATE"
mkdir -p "$STATE"

{
    echo "[[banks]]"
    echo
    i=0
    for f in "$SAMPLES"/*.mp4; do
        name=$(basename "$f" .mp4)
        echo "[[banks.slots]]"
        echo "index = $i"
        echo "name = \"$name\""
        echo "source = { kind = \"file\", value = \"$f\" }"
        echo
        i=$((i + 1))
    done
} > "$STATE/banks.toml"

echo "roots = [\"$SAMPLES\"]" > "$STATE/paths.toml"

echo "--- Done. $(grep -c 'banks.slots' "$STATE/banks.toml") slots mapped into bank A."
echo "    Restart the service to pick them up:  sudo systemctl restart recur.service"
