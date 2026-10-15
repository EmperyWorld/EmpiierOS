# LinkornEW Chromium pin

**Status:** Source build identified. Hash matches the pin.
The lab overlay copies `out/Default` runtime files (ELF, paks, ICU,
Angle `.so` from that build) plus ninja `about_credits.html`.
It does **not** copy Fedora/AppImage `.so` files or host `ld-linux` / glibc.
**Not** planted on official `out/`. **Not** MATCH. **Not** in this git.
The FoFline packager does not copy it.

`plant-chromium.sh` treats the dump as present when
`gen/components/resources/about_credits.html` (or
`licenses/about_credits.html`) is larger than 100 KiB. The stub
`licenses/SHIPPED.txt` that only reprints `licenses.py --help` is
not the dump. `install` still refuses when that HTML is missing.

The chrome ELF needs guest `libnss3`, `libnssutil3`, `libsmime3`,
`libnspr4`, and `libcups.so.2`. Those are **not** in the factory
Buildroot defconfig (GTK/WebKit/OpenSSL only). Staff overlay builds
them from Mozilla NSS 3.101 + NSPR 4.35 (MPL-2.0) and OpenPrinting
CUPS 2.4.11 (Apache-2.0) via `build-guest-nss-cups.sh`. Binaries stay
out of this git. The script refuses Fedora `/usr/lib64` blobs and any
`.so` that needs GLIBC newer than 2.41. Official `out/` is unchanged.
Not MATCH until the factory rebake lists the same packages.

| Field | Value |
|-------|--------|
| Build | station-chromium, from Chromium sources plus ungoogled-chromium |
| Version string | `EliteEW 154.0.8037.57` |
| Tag | `154.0.8037.57-1` |
| sha256 | `98307979de788ecf6d728038865b3947e2bd89bcacea78128dd17a2d394736c8` |
| Staff path | `/usr/lib/station-chromium/chrome` (wrapper → `chrome.bin`) |
| Profile dir | `/var/lib/empiieros-runtime/linkorn/chrome` |
| Widevine | `enable_widevine=false` in that tree's `gn/args.gn` |
| Codecs | `proprietary_codecs=true`, `ffmpeg_branding="Chrome"` |

Check:

```text
runtime/linkornew-seat/plant-chromium.sh check <chrome>
```
