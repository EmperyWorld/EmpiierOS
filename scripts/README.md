# scripts

Foundation and lab tooling. Not acceptance by themselves.

| Script | Role |
|--------|------|
| `preflight-foundation.sh` | EmpiierOS machine gate — hooks, Runtime tests, IPC smoke, Shell build |
| `hooks-selftest.sh` | Policy-guard adversarial cases |
| `runtime-health-smoke.sh` | IPC peer deny / framing / connection bound |
| `bootstrap-phase-smoke.sh` | Shell transport / locale smoke |
| `package-shell-gui-rootfs.sh` | Deterministic overlay + optional `--build-image --resume` |
| `metal-live.sh` | Pinned lab probe / Runtime+Shell deploy / rollback |
| `trash-move.sh` | Soft-delete into `trash/<UTC-stamp>/` |

EmpiierOS preflight is not `trusted-substrate/scripts/selftest-secure-reset.sh`.
Flash only USB (`TRAN=usb` `RM=1`). Do not flash or install against a
polygon you still need as SSH unless the operator named that wipe.
`192.168.8.10/30` is the lab polygon, not a product Settings default.
