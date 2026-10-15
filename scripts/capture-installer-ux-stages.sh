#!/usr/bin/env bash
# Capture production egui installer screens (one PNG per step) for design reference.
# Output: docs/capture/installer-ux-stages-YYYYMMDD/ (EmpiierOS repo).
set -euo pipefail

EMPIIEROS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TS_ROOT="${ELITEEW_TS_ROOT:-${EMPIIEROS_ROOT}/../powered_by_emperyworld/pack-organization/trusted-substrate}"
INSTALLER_BIN="${ELITEEW_TS_INSTALLER_BIN:-${TS_ROOT}/installer/target/release/eliteew-ts-installer}"
OUT="${1:-${EMPIIEROS_ROOT}/docs/capture/installer-ux-stages-$(date -u +%Y%m%d)}"

if [[ ! -x "$INSTALLER_BIN" ]]; then
  echo "capture-installer-ux-stages: build installer first:" >&2
  echo "  (cd ${TS_ROOT}/installer && cargo build --release)" >&2
  exit 1
fi

command -v xvfb-run >/dev/null || {
  echo "capture-installer-ux-stages: need xvfb-run" >&2
  exit 1
}

mkdir -p "$OUT"

capture_step() {
  local idx="$1"
  local step="$2"
  local slug="$3"
  local png="${OUT}/${idx}-${slug}.png"
  echo "capture-installer-ux-stages: ${step} -> ${png}"
  ELITEEW_TS_INSTALLER_STEP="$step" \
  ELITEEW_TS_CHIME_ALREADY=1 \
  ELITEEW_TS_INSTALLER_CAPTURE_OUT="$png" \
  xvfb-run -a -s "-screen 0 1920x1080x24" \
    timeout 30s "$INSTALLER_BIN" || true
  if [[ ! -s "$png" ]]; then
    echo "capture-installer-ux-stages: FAIL empty ${png}" >&2
    exit 1
  fi
}

capture_step 01 Welcome welcome
capture_step 02 Regional regional
capture_step 03 Network network
capture_step 04 Disk disk
capture_step 05 StateKey state-key
capture_step 06 Confirm confirm-erase
capture_step 07 Progress progress
capture_step 08 Done done-pin-recovery
capture_step 09 Failed failed

python3 - "$OUT" <<'PY'
import sys
from pathlib import Path

out = Path(sys.argv[1])
try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError as exc:
    raise SystemExit(f"Pillow required for LUKS mock: {exc}") from exc

w, h = 1920, 1080
img = Image.new("RGB", (w, h), (6, 10, 16))
draw = ImageDraw.Draw(img)
font_paths = [
    "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",
]
font_lg = font_sm = None
for path in font_paths:
    p = Path(path)
    if p.is_file():
        font_lg = ImageFont.truetype(str(p), 28)
        font_sm = ImageFont.truetype(str(p), 18)
        break
if font_lg is None:
    font_lg = font_sm = ImageFont.load_default()

# Cyan chip + titles (palette v2 / state-unlock-agent)
chip = (46, 196, 198)
ink = (6, 10, 16)
muted = (100, 116, 139)
text = (230, 236, 242)
y = 280
draw.rectangle([420, y, 520, y + 36], fill=chip)
draw.text((432, y + 4), "EmpiierOS", fill=ink, font=font_sm)
draw.text((540, y + 4), "ключ восстановления диска", fill=text, font=font_lg)
y += 48
draw.text((420, y), "Не PIN. Не ключ сети. Только LUKS state.", fill=muted, font=font_sm)
y += 40
draw.text((420, y), "> ••••••••••••••••••••••••••••••••", fill=text, font=font_lg)

path = out / "10-installed-boot-luks-console.png"
img.save(path)
print(f"capture-installer-ux-stages: LUKS mock -> {path}")
PY

cat >"${OUT}/README.md" <<EOF
# Installer UX captures ($(date -u +%Y-%m-%d))

Reference PNGs for redesign planning. **Not** acceptance evidence.

| File | Step |
|------|------|
| \`01-welcome.png\` | Welcome |
| \`02-regional.png\` | Язык, пояс, время |
| \`03-network.png\` | Сеть |
| \`04-disk.png\` | Диск |
| \`05-state-key.png\` | Ключ диска (LUKS recovery) |
| \`06-confirm-erase.png\` | ERASE confirm |
| \`07-progress.png\` | Progress (~62%) |
| \`08-done-pin-recovery.png\` | Done (sample PIN/recovery) |
| \`09-failed.png\` | Failed (sample error) |
| \`10-installed-boot-luks-console.png\` | **Mock** console LUKS prompt (not egui) |

**01–09:** real \`eliteew-ts-installer\` egui (1280×800), captured with
\`EFRAME_SCREENSHOT_TO\` + \`ELITEEW_TS_INSTALLER_STEP\` fixtures.
Regenerate: \`bash scripts/capture-installer-ux-stages.sh\`.

**10:** typography/colors aligned with \`state-unlock-agent\` banner; layout approximate.
EOF

echo "capture-installer-ux-stages: OK -> ${OUT}"
