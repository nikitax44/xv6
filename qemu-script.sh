#!/bin/sh
set -e
BASE="$(dirname "$0")"
KERNEL="${KERNEL:-$BASE/kernel}"
if [ -z "$FS" ]; then
    FS="$(mktemp fs.XXXXXX.img)"
    cp "$BASE/fs.img" "$FS"
    echo "copied fs.img to $FS"
    trap "rm -vf '$FS'" EXIT
fi
CPUS="${CPUS:-3}"
qemu-system-riscv64                                            \
    -machine virt -m 128M -smp "$CPUS" -nographic              \
    -global virtio-mmio.force-legacy=false                     \
    -drive file="$FS",if=none,format=raw,id=x0                 \
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0   \
    -kernel "$KERNEL" -bios none
