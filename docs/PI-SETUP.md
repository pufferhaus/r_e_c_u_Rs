# Raspberry Pi 5 — Setup & Deploy

End-to-end procedure to reproduce a working r_e_c_u_Rs install on a Pi 5. Scripts
live in `deploy/`. The verified target is **Pi 5 / Debian 13 (trixie) / vc4-kms**.

## Hardware

- Raspberry Pi 5
- 3.5" SPI TFT LCD, **ILI9486 / MPI3501** clone (control UI), wired:
  reset → GPIO25, dc → GPIO24, led → GPIO18, on SPI0 CE0
- HDMI display for video output (or an HDMI dummy plug for headless)
- USB numpad (19-key) for control — see [NUMPAD.md](NUMPAD.md)

## Build machine prerequisites

```bash
cargo install cross          # needs Docker running
```

The Pi binary is cross-compiled; nothing is built on the Pi.

## First-time provisioning

1. **Flash** Raspberry Pi OS (64-bit) and enable SSH. Note the host/IP.

2. **Deploy the binary + scripts** from the build machine:
   ```bash
   PI=imi@<pi-ip> ./deploy/deploy.sh
   ```
   (Override `FEATURES=pi3` for a Pi 3 B+.)

3. **Provision the Pi** — run once on the Pi (installs packages, boot config,
   systemd service):
   ```bash
   ssh imi@<pi-ip>
   cd ~/r_e_c_u_rs && ./install.sh
   sudo reboot
   ```

After the reboot the `recur.service` starts automatically on boot.

## Updating

Re-run the deploy script; it rebuilds, stops the service, copies the new
binary/shaders/config, and restarts:

```bash
PI=imi@<pi-ip> ./deploy/deploy.sh
```

## Sample clips (optional, for testing)

```bash
ssh imi@<pi-ip>
cd ~/r_e_c_u_rs && ./seed-samples.sh
sudo systemctl restart recur.service
```

Generates 10 distinct test-pattern clips and pre-maps them into bank A slots
0–9 (several animate, so playback/looping is obvious).

## What the setup actually does (and why)

These were the non-obvious hurdles getting it running — captured so they don't
have to be rediscovered:

| Thing | Why |
|---|---|
| `dtoverlay=fbtft,...ili9486` | Drives the SPI LCD as `/dev/fb0`. The app mmaps it. |
| `spidev.bufsiz=524288` | A full 320×480 RGB565 frame is ~300 KB; the default SPI buffer is too small. |
| `hdmi_force_hotplug=1` + `video=HDMI-A-1:...` | Pi 5 has no composite jack. Many monitors / capture cards / KVMs present **no EDID**, so KMS reports 0 modes and the app can't take the display. Forcing the connector exposes fallback modes. |
| Service `ExecStartPre` unbinds **fbcon** | The kernel console binds to `/dev/fb0` (the SPI LCD) and draws the login prompt there, fighting the app for the framebuffer — this caused both the "login prompt on the LCD" and the "column flicker". Unbinding it gives the app the LCD exclusively. |
| Service `ExecStartPre` forces HDMI connector `on` | Same EDID-less display issue, applied at runtime right before launch. |

## Service control

```bash
systemctl status recur.service
sudo systemctl restart recur.service
journalctl -u recur.service -b           # logs since boot
```

The app runs **SPI-LCD-only** (logs a warning) if no HDMI display is found, so
the control UI still works without a monitor attached.

## Troubleshooting

- **LCD blank / shows login prompt** — fbcon still bound. Check
  `cat /sys/class/vtconsole/vtcon*/bind`; the service should have unbound the
  "frame buffer device" one. Restart the service.
- **`no display modes available`** — HDMI has no EDID and wasn't forced on.
  Confirm `hdmi_force_hotplug=1` is in `config.txt` and reboot, or attach an
  HDMI dummy plug.
- **No video on trigger** — verify a clip decodes:
  `gst-launch-1.0 filesrc location=<clip>.mp4 ! decodebin ! fakesink`.
- **Cross build fails** — Docker not running, or `cross` not installed.
