{
  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    flake-parts,
    treefmt-nix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      imports = [
        treefmt-nix.flakeModule
      ];

      perSystem = {
        config,
        pkgs,
        self',
        ...
      }: let
        tpkg = pkgs.pkgsCross.riscv64-embedded;
        newlib = tpkg.newlib.override {nanoizeNewlib = true;};
        platform = tpkg.stdenv.hostPlatform.config;

        qemu-script = pkgs.writeScript "qemu-script" ''
          #!/bin/sh
          set -e
          BASE="$(dirname "$0")"
          KERNEL="''${KERNEL:-$BASE/kernel}"
          if [ -z "$FS" ]; then
            FS="$(mktemp fs.XXXXXX.img)"
            cp "$BASE/fs.img" "$FS"
            echo "copied fs.img to $FS"
            trap "rm -vf '$FS'" EXIT
          fi
          CPUS="''${CPUS:-$(nproc)}"
          qemu-system-riscv64 \
            -machine virt -bios none -m 128M -smp "$CPUS" -nographic \
            -global virtio-mmio.force-legacy=false                     \
            -drive file="$FS",if=none,format=raw,id=x0                 \
            -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0   \
            -kernel "$KERNEL"
        '';

        TOOLPREFIX = "${platform}-";
        NEWLIB = "${newlib}/${platform}";
        nativeBuildInputs = [
          pkgs.stdenv.cc
          pkgs.perl
          pkgs.fd
          pkgs.unixtools.xxd
          (pkgs.writeShellScriptBin "get-busybox" "cp ${busybox}/bin/busybox ./_busybox")
        ];
        buildInputs = [newlib];
        busybox = pkgs.pkgsCross.riscv64.busybox.override {
          enableStatic = true;
          enableAppletSymlinks = false;
          enableMinimal = true;
        };
      in {
        treefmt.config = import ./treefmt.nix;

        checks = {
          build-test = self'.packages.default;
        };

        devShells.default = tpkg.mkShell {
          packages = [
            config.treefmt.build.wrapper
            pkgs.pkgsCross.riscv64.stdenv.cc # not tpkg.stdenv.cc
            pkgs.gnumake
            pkgs.clang-tools
          ];
          inherit NEWLIB TOOLPREFIX buildInputs nativeBuildInputs;
        };

        packages.default = tpkg.stdenv.mkDerivation {
          src = ./.;
          pname = "xv6";
          version = "none";
          preBuild = ''
            make clean
          '';
          inherit NEWLIB TOOLPREFIX buildInputs nativeBuildInputs;
          installPhase = ''
            mkdir $out
            cp kernel/kernel $out/
            cp fs.img $out/
            cp ${qemu-script} $out/qemu-script
          '';
        };
      };
    };
}
