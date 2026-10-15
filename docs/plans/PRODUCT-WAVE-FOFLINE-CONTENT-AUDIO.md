# PRODUCT-WAVE-FOFLINE-CONTENT-AUDIO

**Status:** Pack `1789837366816-w7k51h8btf` on disk. Play after
re-open is the remaining measure. Not official `out/`.
**Date:** 2026-09-20
**Scope lock:** FoFline `/api/v1/content` download bodies on
`fofline-spa:`. No OS A/B OTA. No `audio-guides` fetch.

## Cause

MusicPlayer stream mode sets `<audio src>` to a
`download-by-ticket` URL. A 206 Range slice or `audio/*` on
`fofline-spa:` cancels the load. A full `200` `octet-stream` with
`nosniff` never reaches `canplay`, so the spinner stays. XHR blob
fallback still cannot use `audio/*` on the custom scheme.

## Lock

Proxy strips `Range`, echoes `fofline-spa` CORS + credentials, keeps
XHR bodies as `octet-stream`, and sniffs media-Range clients to a real
audio MIME. A document-start script rewrites those media `src` values
to a typed `blob:` and calls `load()`. That inject is a crutch:
WebKit cancels `blob:fofline-spa:`. The pack lock is
[ELITEEW-TZ-FOFLINE-AUDIO-ON-OS.md](ELITEEW-TZ-FOFLINE-AUDIO-ON-OS.md).

## Measure

Helper sync 03:12 +10: `spa_state=ready`
`spa_app_build_id=1789837366816-w7k51h8btf`. Operator must hide/re-open
FoFline, then Play. Pass is `/download` 200 without `blob:` cancel or
ticket media `src`, and free analog PCM after Stop. If those are clean
and there is still no sound — EmpiierOS ALSA helper, not this pack.
Official `out/` unchanged. Runtime not remuxed.
