let
  pkgs = import <nixpkgs> {};
  tpkg = pkgs.pkgsCross.riscv64-embedded;
  newlib = tpkg.newlib.override {nanoizeNewlib = true;};

  qemu-script = ''
    #!/bin/sh
    BASE="$(dirname "$0")"
    KERNEL="''${KERNEL:-$BASE/kernel}"
    if [ -z "$FS" ]; then
      FS="$(mktemp)"
      cp "$BASE/fs.img" "$FS"
    fi
    CPUS="''${CPUS:-$(nproc)}"
    qemu-system-riscv64 \
      -machine virt -bios none -m 128M -smp "$CPUS" -nographic \
      -global virtio-mmio.force-legacy=false                     \
      -drive file="$FS",if=none,format=raw,id=x0                 \
      -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0   \
      -kernel "$KERNEL"
  '';
in
  pkgs.pkgsCross.riscv64-embedded.stdenv.mkDerivation {
    src = ./.;
    pname = "xv6";
    version = "none";
    preBuild = "make clean";
    buildFlags = ["kernel/kernel fs.img"];
    nativeBuildInputs = [pkgs.gcc pkgs.perl];
    buildInputs = [newlib];
    makeFlags = [
      "TOOLPREFIX=riscv64-none-elf-"
      "EXTRA_CFLAGS=-I${newlib}/riscv64-none-elf/include"
      "EXTRA_LDFLAGS=${newlib}/riscv64-none-elf/lib/libc.a"
    ];
    installPhase = ''
      mkdir $out
      cp kernel/kernel $out/
      cp fs.img $out/
      cp ${pkgs.writeScript "qemu-script" qemu-script} $out/qemu-script
    '';
  }
