#!/usr/bin/env bash
# Generate a set of distinct H.264 test clips for the sampler, using GStreamer's
# videotestsrc patterns. Several patterns animate (ball / pinwheel / spokes /
# circular) so playback and looping are obvious on screen.
#
# Usage:  scripts/gen-test-clips.sh [OUTDIR] [SECONDS]
#   OUTDIR   target directory (default: ./samples)
#   SECONDS  clip length in seconds (default: 5)
#
# Requires: gst-launch-1.0 with x264enc + mp4mux (gstreamer1.0-plugins-ugly).
set -euo pipefail

OUTDIR="${1:-./samples}"
SECS="${2:-5}"
W=640
H=480
FPS=30
NBUF=$((SECS * FPS))

mkdir -p "$OUTDIR"

if ! command -v gst-launch-1.0 >/dev/null; then
    echo "error: gst-launch-1.0 not found" >&2
    exit 1
fi

# slot → "pattern_number:name"
clips=(
    "0:smpte"      # 0 colour bars
    "18:ball"      # 1 bouncing ball   (motion)
    "11:circular"  # 2 concentric zone (motion)
    "1:snow"       # 3 TV static
    "4:red"        # 4 solid red
    "5:green"      # 5 solid green
    "6:blue"       # 6 solid blue
    "10:checkers"  # 7 checkerboard
    "21:pinwheel"  # 8 pinwheel        (motion)
    "22:spokes"    # 9 spokes          (motion)
)

i=0
for entry in "${clips[@]}"; do
    pat="${entry%%:*}"
    name="${entry##*:}"
    out="$OUTDIR/$(printf '%02d' "$i")_${name}.mp4"
    echo "→ $out (pattern=$pat, ${SECS}s)"
    gst-launch-1.0 -e -q \
        videotestsrc pattern="$pat" num-buffers="$NBUF" \
        ! "video/x-raw,width=$W,height=$H,framerate=$FPS/1" \
        ! videoconvert \
        ! x264enc key-int-max="$FPS" bitrate=2000 speed-preset=veryfast \
        ! mp4mux faststart=true \
        ! filesink location="$out"
    i=$((i + 1))
done

echo
echo "Generated $i clips in $OUTDIR:"
ls -1 "$OUTDIR"/*.mp4
