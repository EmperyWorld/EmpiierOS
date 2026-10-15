#!/usr/bin/env bash
# Write a prepared LinkornEW rootfs to one removable USB disk.
# Refuses NVMe and any disk that is not TRAN=usb RM=1.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dev=""
rootfs=""

usage() {
  printf 'usage: %s --dev /dev/sdX --rootfs DIR\n' "$0" >&2
  exit 2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dev) dev=$2; shift 2 ;;
    --rootfs) rootfs=$2; shift 2 ;;
    *) usage ;;
  esac
done

[[ -n "$dev" && -n "$rootfs" && -d "$rootfs/etc/empiieros" ]] || usage
profile=$(tr -d '\n' < "$rootfs/etc/empiieros/seat-profile")
[[ "$profile" == "linkornew" ]] || { printf 'linkornew-flash: rootfs profile is not linkornew\n' >&2; exit 1; }
[[ -f "$rootfs/boot/bzImage" ]] || { printf 'linkornew-flash: rootfs has no kernel\n' >&2; exit 1; }
# User extract of rootfs.tar.gz drops setuid and uid 0. Runtime then dies:
# state_store_open_failed (directory owner does not match runtime euid).
# Factory /usr/bin/login is 0755; dbus-daemon-launch-helper is the setuid canary.
if [[ ! -u "$rootfs/usr/libexec/dbus-daemon-launch-helper" ]]; then
  printf 'linkornew-flash: refusing: dbus helper is not setuid; extract factory tar as root\n' >&2
  exit 1
fi
state_uid=$(stat -c '%u' "$rootfs/var/lib/empiieros-runtime")
if [[ "$state_uid" != 0 ]]; then
  printf 'linkornew-flash: refusing: state dir uid=%s (want 0)\n' "$state_uid" >&2
  exit 1
fi
[[ -b "$dev" ]] || { printf 'linkornew-flash: not a block device\n' >&2; exit 1; }
[[ "$dev" != /dev/nvme* ]] || { printf 'linkornew-flash: refusing nvme\n' >&2; exit 1; }

tran=$(lsblk -ndo TRAN "$dev" | tr -d ' ')
rmflag=$(lsblk -ndo RM "$dev" | tr -d ' ')
if [[ "$tran" != "usb" || "$rmflag" != "1" ]]; then
  printf 'linkornew-flash: refusing %s tran=%s rm=%s\n' "$dev" "$tran" "$rmflag" >&2
  exit 1
fi

if [[ "$(id -u)" -ne 0 ]]; then
  exec sudo -n "$0" --dev "$dev" --rootfs "$rootfs"
fi

while read -r part; do
  umount "$part" 2>/dev/null || true
done < <(lsblk -lnpo NAME "$dev" | tail -n +2)

# Fourth CSV field is GPT UUID, not a name. A token there is "unsupported command".
sfdisk --wipe always --wipe-partitions always "$dev" <<'EOF'
label: gpt
,256MiB,U,
,,L
EOF
partprobe "$dev" || true
sleep 1
efi=${dev}1
ext=${dev}2
if [[ ! -b "$efi" || ! -b "$ext" ]]; then
  efi=${dev}p1
  ext=${dev}p2
fi
mkfs.vfat -n LINKEFI "$efi"
mkfs.ext4 -F -L LINKORNEW "$ext"
partuuid=$(blkid -s PARTUUID -o value "$ext")
if [[ ! "$partuuid" =~ ^[0-9a-fA-F-]{36}$ ]]; then
  printf 'linkornew-flash: missing PARTUUID on %s\n' "$ext" >&2
  exit 1
fi
mnt=$(mktemp -d)
efimnt=$(mktemp -d)
mount "$ext" "$mnt"
mount "$efi" "$efimnt"
tar -C "$rootfs" --owner=0 --group=0 --numeric-owner -cf - . | tar -C "$mnt" --numeric-owner -xf -
mkdir -p "$efimnt/EFI/BOOT" "$efimnt/boot"
cp "$rootfs/boot/bzImage" "$efimnt/boot/bzImage"
factory_efi="${ELITEEW_TS_ROOT:-$ROOT/../powered_by_emperyworld/pack-organization/trusted-substrate}/out/efi-part/EFI/BOOT/bootx64.efi"
cp "$factory_efi" "$efimnt/EFI/BOOT/bootx64.efi"
# Kernel name_to_dev_t accepts PARTUUID=/PARTLABEL=/dev/…, not fs LABEL=.
# LABEL= disables rootwait ("root is invalid") and panics on unknown-block(0,0)
# before USB storage appears. Factory grub uses /dev/sda1; this stick is p2.
cat > "$efimnt/EFI/BOOT/grub.cfg" <<EOF
set default="0"
set timeout="3"

menuentry "LinkornEW" {
	linux /boot/bzImage root=PARTUUID=${partuuid} rootwait console=tty1
}
EOF
sync
umount "$efimnt" "$mnt"
printf 'linkornew-flash: wrote %s profile=linkornew\n' "$dev"
