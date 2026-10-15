# PRODUCT-WAVE-SEAT-CHROME-PANEL

**Status:** In source. Lab syslog observed 2026-09-20 04:13 +10.
Not official `out/`. Not metal MATCH.
**Date:** 2026-09-20
**Scope lock:** Always-on native HUD + per-icon flyouts. No OS A/B OTA client.
No `audio.action` enable. No pack music library. No `autoOpened`.

## Cause

Icon clicks were not flyouts. Launcher and Shield jumped straight into
the app. EliteEW dispatched `workspace=eliteew` without `activator`
while the host was already there (no-op, no sound). Volume aliased
the sensor panel. Click spawn preferred `paplay` on a Pulse-less seat
so only the launcher *app* path sounded like a click.

## Lock

1. HUD is always present (not FoFline-only). It docks on the **right
   edge** (`GTK_ALIGN_END` / `GTK_ALIGN_CENTER`), icons vertical,
   fully visible. No peek / hover / scrolled clip. Flyouts open left.
   `hide_product` does not hide the HUD.
2. Five non-modal `GtkPopover` flyouts (`chrome_toggle_flyout`). Each
   icon: summary + quick actions. Open is a button inside the flyout.
   EliteEW Open = `product.open` + allowlisted `show_product`.
   Panel = mic/cam/GPS observe. Volume = ALSA `audio.control` observe.
   Shield = status/events/`observe` + Open. CSS stays
   `GTK_STYLE_PROVIDER_PRIORITY_USER`.
3. `play_click` is `aplay -q` first. `on_volume_clicked` must not call
   `on_panel_clicked`. Shield icon must not call `open_host_workspace`.
4. Gates require `movable no-peek` / `chrome_hide_flyouts` /
   `Открыть EliteEW` and forbid `chrome_schedule_peek` /
   `chrome_peek_height` / `gtk_scrolled_window_new` / `-48px`. Peek
   stays forbidden. HUD may be dragged; default remains right-center.

## Honesty (not this wave)

- Unified OS OTA client: **absent**. FoFline pack OTA is `launcher.spa_sync`.
  Image A/B is frozen P93. `update.apply` stays honesty/staging.
- FoFline MusicPlayer spinner: `include_audio=false`, `audio-guides` deferred.
  Host tile `MusicPlayer` is ALSA volume, not a library player.

## Measure

`scripts/preflight-foundation.sh` exit 0. `scripts/bootstrap-phase-smoke.sh`
exit 0. Live GUI `b94fa6be…` PID `22262` (mux after icon regen):
`dock=end no-peek`, splash `revealed hud` 04:13:14, then 04:13:44–04:13:48
flyouts `launcher` / `eliteew` / `panel` / `shield` / `volume` each with
`seat chrome click alsa`. Analog PCM free after (`Subdevices: 1/1`).
Official `out/` unchanged. Not `verified-real`.
