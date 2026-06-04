#!/usr/bin/env bash
# install.sh — one-time provisioning for r_e_c_u_Rs on a Raspberry Pi 5.
#
# Run this ON the Pi (it uses sudo internally and will prompt):
#     cd ~/r_e_c_u_rs && ./install.sh
#
# It is idempotent — safe to re-run. A reboot is required afterwards for the
# boot-config (SPI overlay + HDMI) changes to take effect.
#
# Captures the verified working setup:
#   - GStreamer + DRM/GBM/EGL runtime packages
#   - SPI ILI9486 LCD overlay + DMA buffer
#   - HDMI forced-on (for EDID-less displays / capture cards / KVMs)
#   - systemd service (unbinds fbcon + forces HDMI before launch)
set -euo pipefail

DEPLOY_DIR="$(cd "$(dirname "$0")" && pwd)"
BOOT_CFG="/boot/firmware/config.txt"
CMDLINE="/boot/firmware/cmdline.txt"

echo "=== r_e_c_u_Rs install ==="

# ── 1. Runtime packages ────────────────────────────────────────────────────
echo "--- Installing runtime packages (GStreamer + DRM/GBM/EGL)"
sudo apt-get update -q
sudo apt-get install -y --no-install-recommends \
    gstreamer1.0-tools \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav \
    libgstreamer1.0-0 \
    libdrm2 libgbm1 libegl1

# ── 2. SPI ILI9486 LCD overlay ─────────────────────────────────────────────
if ! grep -q "ili9486" "$BOOT_CFG" 2>/dev/null; then
    echo "--- Adding SPI + fbtft overlay to $BOOT_CFG"
    sudo tee -a "$BOOT_CFG" > /dev/null << 'EOF'

# r_e_c_u_Rs: SPI TFT LCD (3.5" ILI9486). fps capped low; the app only writes
# the framebuffer on change (fbcon is unbound by the service), so this is plenty.
dtparam=spi=on
dtoverlay=fbtft,spi0-0,ili9486,reset_pin=25,dc_pin=24,led_pin=18,width=320,height=480,rotate=0,bgr=1,speed=32000000,fps=10
EOF
else
    echo "--- fbtft overlay already present, skipping"
fi

# ── 3. Force HDMI on (Pi 5 has no composite jack; many displays/KVMs/capture
#       cards present no EDID, so KMS reports 0 modes without this) ──────────
if ! grep -q "hdmi_force_hotplug" "$BOOT_CFG" 2>/dev/null; then
    echo "--- Forcing HDMI hotplug in $BOOT_CFG"
    echo -e "\n# r_e_c_u_Rs: drive HDMI even with no detected display\nhdmi_force_hotplug=1" \
        | sudo tee -a "$BOOT_CFG" > /dev/null
else
    echo "--- hdmi_force_hotplug already present, skipping"
fi

# ── 4. Kernel cmdline: SPI DMA buffer + a forced HDMI mode ──────────────────
if ! grep -q "spidev.bufsiz" "$CMDLINE" 2>/dev/null; then
    echo "--- Adding spidev.bufsiz=524288 to $CMDLINE"
    sudo sed -i 's/$/ spidev.bufsiz=524288/' "$CMDLINE"
fi
if ! grep -q "video=HDMI" "$CMDLINE" 2>/dev/null; then
    echo "--- Adding video=HDMI-A-1:1280x720@60 to $CMDLINE"
    sudo sed -i 's/$/ video=HDMI-A-1:1280x720@60/' "$CMDLINE"
fi

# ── 5. systemd service ─────────────────────────────────────────────────────
echo "--- Installing + enabling systemd service"
sudo cp "$DEPLOY_DIR/recur.service" /etc/systemd/system/recur.service
sudo systemctl daemon-reload
sudo systemctl enable recur.service

echo ""
echo "=== Done. Reboot to activate the SPI overlay and HDMI: ==="
echo "    sudo reboot"
