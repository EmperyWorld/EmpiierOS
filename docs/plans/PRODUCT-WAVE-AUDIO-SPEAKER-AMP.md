# Product wave: Dell ALC3204 speaker-amp GPIO

**Date:** 2026-09-18
**Status:** Lab helper muxed and GPIO restore measured; not metal-ready; official `out/` unchanged.
**Plane:** Runtime helper + session contract.

## Cause

Mixer IPC and `aplay` to `hw:0,0` already succeeded on `192.168.8.10`, but
the seat stayed silent. `snd_hda_intel` is from-source and built-in. The
Dell SSID `0x1028:09c1` / Realtek `0x10ec0236` has no mainline quirk, so
AFG GPIO stayed all-off and the analog amp never enabled.

## Lock

- `runtime/helpers/hda_speaker_amp.c` writes GPIO mask/dir/data `0x07`
  through `/dev/snd/hwC0D0` only when sysfs vendor+SSID match.
- `empiieros-audio-volume` calls that on `--observe` and on set.
- Boot oneshot `empiieros-audio-amp.service` after `sound.target`.
- No Pulse, no PipeWire, no foreign `hda-verb`.

A later image bake should add `SND_PCI_QUIRK(0x1028, 0x09c1, …)` in the
in-tree Realtek patch so resume does not depend on the helper.

## Out of scope

Official `out/` remux. Runtime/GUI remux. Meet/call surface.
