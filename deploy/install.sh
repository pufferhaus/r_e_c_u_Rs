#!/usr/bin/env bash
# install.sh — provision r_e_c_u_Rs on Pi 5 (run as imi, uses sudo internally)
set -euo pipefail

DEPLOY_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_DIR="/home/imi/r_e_c_u_rs"

echo "=== r_e_c_u_Rs install ==="

# 1. Enable SPI + fbtft overlay if not already present
BOOT_CFG="/boot/firmware/config.txt"
if ! grep -q "fb_ili9486\|fbtft" "$BOOT_CFG" 2>/dev/null; then
    echo "--- Adding SPI + fbtft overlay to $BOOT_CFG"
    sudo tee -a "$BOOT_CFG" > /dev/null << 'EOF'

# r_e_c_u_Rs: SPI TFT LCD (3.5" ILI9486)
dtparam=spi=on
dtoverlay=fbtft,spi0-0,ili9486,reset_pin=25,dc_pin=24,led_pin=18,width=320,height=480,rotate=0,bgr=1,speed=32000000,fps=30
EOF
    echo "    Added."
else
    echo "--- fbtft overlay already in $BOOT_CFG, skipping"
fi

# 2. Increase SPI DMA buffer in cmdline.txt
CMDLINE="/boot/firmware/cmdline.txt"
if ! grep -q "spidev.bufsiz" "$CMDLINE" 2>/dev/null; then
    echo "--- Adding spidev.bufsiz=524288 to $CMDLINE"
    sudo sed -i 's/$/ spidev.bufsiz=524288/' "$CMDLINE"
    echo "    Added."
else
    echo "--- spidev.bufsiz already in $CMDLINE, skipping"
fi

# 3. Install systemd service
echo "--- Installing systemd service"
sudo cp "$DEPLOY_DIR/recur.service" /etc/systemd/system/recur.service
sudo systemctl daemon-reload
sudo systemctl enable recur.service
echo "    Enabled recur.service"

echo ""
echo "=== Done. Reboot to activate SPI overlay and start service on boot. ==="
echo "    sudo reboot"
