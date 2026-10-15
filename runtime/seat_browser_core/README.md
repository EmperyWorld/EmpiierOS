# seat_browser_core

Prep-bay **pure** Seat Browser logic for EmpiierOS.

- Product SoT: `docs/apps/SeatBrowser/EMPIIEROS-SEAT-BROWSER.md`
- Harvest: EliteEW EBrowser `ebrowser_nav_policy` (lockstep modes / HTTPS deny / TTL)

```bash
cargo test --manifest-path seat_browser_core/Cargo.toml --lib
```

**Not:** WebKit host, Tauri stage, EmpiierOS Runtime daemon.
