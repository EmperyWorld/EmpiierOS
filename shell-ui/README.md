# shell-ui

**Plane:** L-shell — EliteEW OS Shell (React session UI).
**Owns:** OS Shell UI source (scoped product waves).
**Must not:** privileged ops execution; Tauri host; wholesale monorepo FE clone.

**Status:** React 19 + TypeScript 7 + Vite 8. Production face is the Secure
Shell (`src/secure/`) plus the seat launcher host (`src/desktop/`).
The host is Unlock → Activate → Portal → Workspace → Sign-in → Open
(`foflineLauncherStage.ts`), not a second EliteEW catalog. Native bridge
only; no raw Tauri commands. `product.open` stays fail-closed. Verdict
`NOT_READY`. Native HUD (always-on right-edge dock) is GTK
(`runtime/shell-gui/product_chrome.c`), not React. Media copies in
`public/product-chrome/` keep `npm run build` in `dist/`; HUD SoT remains
`runtime/shell-gui/product-chrome/`. Seat Browser is workspace `browser`
(key 6) plus the native overlay. Glass map:
[`../session/SEAT-FACE.md`](../session/SEAT-FACE.md). The host canvas
harvests EliteEW visual DNA only — not pack apps.

**Provenance:** [`ORIGIN.md`](ORIGIN.md).

## Dev

```bash
npm ci --prefix shell-ui
npm run dev --prefix shell-ui
npm run build --prefix shell-ui
```

`npm run dev` rewrites CSP in memory so a desktop browser can load
`http://127.0.0.1:5173/`. Production `dist/index.html` keeps
`script-src empiier:` for the kiosk. Do not put the local CSP in
`index.html`.

Layout: `src/secure/SecureApp.tsx` · desktop host:
`src/desktop/FoflineDesktop.tsx` · bridge: `src/secure/nativeTransport.ts`.
Historical chrome/session helpers may remain on disk; they are not the
production face.
