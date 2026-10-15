# TZ — FoFline sound on EmpiierOS (EliteEW pack)

**For:** EliteEW monorepo session (`powered_by_emperyworld`).
**From:** EmpiierOS 2026-09-20 02:38 +10. Do **not** edit EmpiierOS
from that session. Do **not** add Pulse/PipeWire, a Tauri plugin, or
another OS MIME/CORS inject.

**Pack now:** `1789837366816-w7k51h8btf` on the lab seat
(`include_audio=false`, helper sync 2026-09-20 03:12 +10).
Decoded owner is packed into `contentApi-Bstp_TrW.js`
(`decodeAudioData` + `createBufferSource`; `Ro()` skips
`Range: bytes=0-0` on `foflineNative`). Prior seat
`1789794743865-pza6u2zqzf`. Official EmpiierOS `out/` is not this TZ.

Meet bounds stay
[ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md](ELITEEW-TZ-FOFLINE-MEET-BOUNDS.md).
This TZ is **pack playback and Web Audio**, not a second Meet client.

---

## Operator sentence

Sound in FoFline on the OS seat is **absent**. Cloud/browser and
Tauri play. The OS MusicPlayer spinner never becomes sound. Click
foley and volume on the host are not the product player.

EmpiierOS already wasted a night on custom-scheme CORS, `octet-stream`,
Range strip, and a `blob:` user-script. Those are **crutches**. The
generator is in EliteEW: the pack plays like Chromium over HTTPS+Pulse.

---

## Glass (do not re-litigate)

Seat `192.168.8.10`, GUI `de7be00f…` PID `116418` (HUD clip mux
`20260919T164914Z-shell-18165`). Product WebKit still holds the
previous pack until hide/re-open.

1. `POST /api/v1/content/9df2440c-…/download-ticket` → `200`.
2. `GET …/download-by-ticket` → `200` `1851567` bytes. File **arrives**.
3. Immediately `blob:fofline-spa://localhost/…` `Load request cancelled`.
4. Fallback `GET …/download` → another `200` `1851567`. Spinner stays.
5. No Pulse/PipeWire on the image (law). Analog PCM `hw:0,0` is held
   exclusively by `WebKitWebProcess` (`Subdevices: 0/1`) after the
   cancelled media load. That is why **the whole FoFline surface**
   goes silent, not only one track.
6. Host tile `MusicPlayer` is ALSA `audio.control` (volume). It is not
   this library. `audio-guides` is deferred and is **not** this track.

Web/Tauri works because `<audio src>` is `https://app.eliteew.ru/…`
into Chromium + a sound server. EmpiierOS FoFline origin is
`fofline-spa://localhost`. Media is WebKitGTK → GStreamer → raw ALSA.

---

## Cause (one layer)

On `foflineNative` the now-playing engine still does Chromium media:

1. Prefer **stream**: `createContentDownloadTicket` +
   `<audio src=fofline-spa://…/download-by-ticket>`.
2. GStreamer will not play that custom-scheme URL (Range `206`,
   `audio/*` cancel, or `octet-stream`+nosniff never `canplay`).
3. Fallback `fetchAudioBlob` + `URL.createObjectURL` sets
   `blob:fofline-spa://…`. WebKit **cancels** that load too.
4. The failed `<audio>` keeps the ALSA sink open. Later Play, UI
   sounds, and Meet in the same process have no device.

The OS proxy cannot make GStreamer treat `fofline-spa:` like HTTPS.
Fix the **pack playback owner**.

---

## Required lock (must be in the published pack)

Treat EmpiierOS as
`getOidcRuntimeProfile() === 'foflineNative'`
and/or `location.protocol === 'fofline-spa:'`
and **no** `__TAURI_INTERNALS__`.

### 1. One playback owner on `foflineNative`

All of: MusicPlayer now-playing, `AuthenticatedAudioPreview`,
inline audio, notification / TTS preview that uses `<audio>` or
`new Audio()`.

Do **not** set `HTMLMediaElement.src` (or `<source src>`) to:

- any `fofline-spa:` `/api/v1/content/…/download*` URL
- any `blob:` created from those bodies

Do **not** probe stream readiness with `Range: bytes=0-0` on those
URLs (`qo` in `contentApi`). A `206` slice is not a playable track
on this host.

### 2. Play from memory, not from a media URL

On `foflineNative`:

1. `GET /api/v1/content/{id}/download` (or ticket-then-XHR, same
   bytes) with `credentials` / axios `responseType: blob|arraybuffer`.
   Empty body is `EMPTY_AUDIO_BLOB` — same as today.
2. `arrayBuffer()` → `AudioContext.decodeAudioData`.
3. Play with `AudioBufferSourceNode` (or one shared context + clock).
   Pause / seek / rate / volume are your clock + gain, not
   `HTMLAudioElement`.
4. On stop / track change / error: `source.stop()`, disconnect,
   **and** if any leftover `<audio>` exists,
   `removeAttribute('src'); load()` so GStreamer releases ALSA.

Browser and Tauri arms **keep** today’s `<audio src>` / blob path.
Do not regress cloud.

### 3. Honest failure, not an infinite spinner

If `decodeAudioData` rejects, show the existing decode error
(`AUDIO_DECODE_ERROR` / `playbackErrors`). Do not retry stream URL
on `fofline-spa:`. Do not spin `isLoading` after a terminal decode
fail. Do not remint ticket as a media `src`.

### 4. Do not invent a host sound daemon from EliteEW

No Pulse, no `pactl`, no Tauri `invoke` for playback, no second
Meet/WebRTC stack. If `decodeAudioData` is proven dead on glass
**after** this pack (syslog has no `blob:` / `download-by-ticket`
media `src`, and ALSA is free), **stop** and write that. EmpiierOS
then owns an ALSA helper. Do not guess that helper from EliteEW.

---

## Forbidden

- “Fix OS” CORS / MIME / `__empiierosAudioBlob` / `data:` inject as
  the SoT. Those stay EmpiierOS-side leftovers; the pack must work
  **without** them.
- `include_audio=true` / fetching `audio-guides` (1.5 GiB). User
  content `9df2440c-…` is enough.
- Restoring Tauri `tauri-plugin-updater` or `/desktop-client-updates/`.
- Changing Meet iframe vs bounds (other TZs).
- Claiming DEMO_PASSED / MATCH / product-ready.

---

## Tests (EliteEW tree)

1. Unit: `foflineNative` + play → **no** `HTMLMediaElement.src`
   assignment to `/download-by-ticket` or `blob:`.
2. Unit: `foflineNative` + play → `decodeAudioData` called with a
   non-empty `ArrayBuffer`; stop path clears any `<audio src>`.
3. Unit: `browser` / Tauri profile still uses the living stream/blob
   path (spy, do not delete it).
4. Unit: stream `Range: bytes=0-0` probe is **not** used on
   `foflineNative`.

Name the files you actually change (`useNowPlaying` / music playback
hook / `AuthenticatedAudioPreview`). Do not hide the lock in a minified
pack-only edit without source.

---

## Measure (glass, after a published pack)

Operator: Open FoFline on the lab seat, Play the same track.

Pass:

- syslog: ticket and/or `/download` `200` full bytes
- **no** `blob:fofline-spa://… Load request cancelled`
- **no** `HTMLMediaElement` request to `download-by-ticket` as media
- `aplay -l` analog `Subdevices` not stuck `0/1` after stop
- audible play from the speakers (not host click WAV)

Fail-closed: spinner after a full `200` body, or PCM still held after
Pause/Stop, is **not** “OS CORS again”.

---

## EmpiierOS leftovers (ignore in the EliteEW session)

The kiosk may still strip `Range` and force `octet-stream` on XHR.
That is compatible with step 2. It is **not** acceptance. Do not
build the pack to require `__empiierosAudioBlob`.
