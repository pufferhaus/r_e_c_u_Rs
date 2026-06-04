#!/usr/bin/env bash
# deploy.sh — cross-compile r_e_c_u_Rs and push it to the Pi.
#
# Run on the build machine (needs `cross` + Docker):
#     PI=imi@192.168.86.47 ./deploy/deploy.sh
#
# Env overrides:
#     PI         ssh target           (default: imi@recur.local)
#     APP_DIR    install dir on Pi     (default: /home/imi/r_e_c_u_rs)
#     FEATURES   cargo feature         (default: pi5; use pi3 for a Pi 3 B+)
#
# First-time setup on a fresh Pi: after the first deploy, run `./install.sh`
# on the Pi once (provisions packages, boot config, systemd) then reboot.
set -euo pipefail

PI="${PI:-imi@recur.local}"
APP_DIR="${APP_DIR:-/home/imi/r_e_c_u_rs}"
FEATURES="${FEATURES:-pi5}"
TARGET=aarch64-unknown-linux-gnu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/$TARGET/release/r_e_c_u_rs"

echo "== cross-compile ($FEATURES) =="
( cd "$ROOT" && cross build --release --no-default-features --features "$FEATURES" --target "$TARGET" )

echo "== stage dirs on $PI =="
ssh "$PI" "mkdir -p '$APP_DIR/shaders'"

echo "== copy artifacts =="
# `-t` gives sudo a tty to prompt on. Stop the service so the binary isn't busy.
ssh -t "$PI" "sudo systemctl stop recur.service 2>/dev/null || true"
scp "$BIN" "$PI:$APP_DIR/"
scp -r "$ROOT/shaders/." "$PI:$APP_DIR/shaders/"
scp "$ROOT/keymap.toml" "$ROOT/config.toml" "$PI:$APP_DIR/"
scp "$ROOT/deploy/recur.service" "$ROOT/deploy/install.sh" \
    "$ROOT/scripts/gen-test-clips.sh" "$PI:$APP_DIR/"

echo "== (re)start service =="
ssh -t "$PI" "sudo systemctl start recur.service; sleep 2; systemctl --no-pager status recur.service | head -4"

echo "== done =="
