# Audio on the seat (living)

**Status:** Living contract. Verified against `runtime/src/core.rs`,
`runtime/src/audio_control.rs`, `runtime/helpers/empiieros-audio-volume.c`,
and `runtime/shell-gui/product_chrome.c` on 2026-09-20.
**Plane:** Session + Runtime + native kiosk. **Not:** Pulse/PipeWire.
**Not:** FoFline MusicPlayer. **Not:** `audio.action`.

Historical P74 HTTP `GET /v0/audio/status` / `POST /v0/audio/action` and the
Pulse/PipeWire mixer zoo live only as archive in [`CONTRACT.md`](CONTRACT.md).
They are not production.

## Production path

1. **One wired op.** `audio.control` `{ "action": "set_volume", "percent": 10-100, "name"?: mixer }`.
   Runtime starts allowlisted `empiieros-audio-volume.service`. Percent
   below 10 is rejected. Snapshot audio is `--observe` (no mixer write).
2. **ALSA only.** No Pulse, no PipeWire, no `pactl`. Host click foley is
   `aplay -q` first (`product_chrome.c` `play_click`). Boot splash uses
   GStreamer `alsasink` at volume 0.25.
3. **`audio.action` disabled.** Source select, input mute/volume, and
   profile switch return `not_available` / `capability_not_available`.
   The HUD panel lists `/proc/asound/cards` and `/dev/snd` names; combo
   changes syslog `audio.action not wired`. Mic/cam switches are session
   preference only (`no hardware kill-switch`). GPS stays off.
4. **Host tile `MusicPlayer`.** Shell ALSA volume control. It is not the
   FoFline library player. `audio-guides` is deferred (1.5 GiB).
5. **FoFline content.** Pack playback is EliteEW `decodeAudioData` on
   `foflineNative`. Native proxy/inject leftovers are documented in
   [`SPA-PACK.md`](SPA-PACK.md) and [`SEAT-FACE.md`](SEAT-FACE.md). They
   are not this mixer contract.

## Analog speaker amp (Dell ALC3204)

The lab seat (`1028:09C1` / codec `0x10ec0236`) already runs from-source
`snd_hda_intel` (built-in). Mixer IPC and PCM to `hw:0,0` can succeed while
the analog amp stays off: mainline `patch_realtek.c` has no `0x09c1` quirk,
so AFG GPIO stays `enable=0`. Allowlisted `empiieros-audio-volume` writes
GPIO mask/dir/data `0x07` through `/dev/snd/hwC0D0` on that vendor+SSID
only. Boot unit `empiieros-audio-amp.service` runs `--observe` after
`sound.target`. Do not ship Pulse/PipeWire or a prebuilt `hda-verb`.

A later image bake should add `SND_PCI_QUIRK(0x1028, 0x09c1, …)` in the
in-tree Realtek patch so resume/reconfig do not depend on the helper.

## Non-goals

- Spawning a sound server or executing `alsamixer` / `pactl` from Shell.
- Claiming a capture kill-switch or default-sink picker.
- Treating host click WAV or splash AAC as product MusicPlayer.
- Enabling `audio.action` without a new scoped wave + substrate policy.
