{
  nixConfig = {
    fallback = true;
  };

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    flake-parts,
    treefmt-nix,
    rust-overlay,
    nixpkgs,
    crane,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      imports = [
        treefmt-nix.flakeModule
      ];

      perSystem = {
        config,
        system,
        self',
        ...
      }: let
        localSystem = system;
        crossSystem.config = "riscv64-unknown-none-elf";

        localPkgs = import nixpkgs {
          inherit localSystem;
          overlays = [(import rust-overlay)];
        };

        crossPkgs = import nixpkgs {
          inherit crossSystem localSystem;
          overlays = [(import rust-overlay)];
        };

        crossEnv = crossPkgs.stdenv;

        inherit (nixpkgs) lib;

        craneLibFor = pkgs:
          (crane.mkLib pkgs).overrideToolchain (p:
            p.rust-bin.nightly.latest.default.override {
              extensions = ["rust-src"];
            });

        enableOpenSBI = false;

        prefixDrv = prefix: drv:
          crossPkgs.runCommandNoCCLocal "prefix-drv" {} ''
            mkdir $out
            cp -r ${drv} $out/${prefix}
          '';

        rust-xv6 = crossPkgs.callPackage ./rust-xv6 {
          craneLib = craneLibFor crossPkgs;
        };

        tar2fs = localPkgs.callPackage ./tar2fs {
          craneLib = craneLibFor localPkgs;
        };

        kernel = crossEnv.mkDerivation {
          pname = "kernel";
          inherit (rust-xv6) version;
          src = (prefixDrv "kernel" ./kernel) + "/kernel";
          inherit nativeBuildInputs;

          cmakeFlags = ["-DRUST_XV6=${rust-xv6}/lib/librust_xv6.a" "-DOPENSBI_ENABLED=${toString enableOpenSBI}"];
          buildFlags = "kernel";
          installPhase = ''
            install -m 0444 kernel $out
          '';
          dontStrip = true;
        };

        programs = crossEnv.mkDerivation ({
            pname = "programs.tar";
            version = "none";
            src = ./.;

            cmakeFlags = ["-DNEWLIB=${NEWLIB}" "-S ../user"];
            buildFlags = "Programs";
            installPhase = ''
              install -m 0444 fs.tar $out
            '';
          }
          // common);

        fsImg = localPkgs.runCommandNoCCLocal "fs.img" {} ''
          ${tar2fs}/bin/tar2fs ${programs} $out
        '';

        newlib = crossPkgs.newlib.override {nanoizeNewlib = true;};
        platform = crossEnv.hostPlatform.config;
        NEWLIB = "${newlib}/${platform}";
        nativeBuildInputs = [
          localPkgs.perl
          localPkgs.fd
          localPkgs.cmake
          localPkgs.unixtools.xxd
        ];
        buildInputs = [newlib];

        common = {
          inherit NEWLIB buildInputs nativeBuildInputs;
          OPENSBI_ENABLED = lib.toUpper (lib.boolToString enableOpenSBI);
        };
      in {
        treefmt.config = import ./treefmt.nix;

        checks = {
          build-test = self'.packages.default;
        };

        devShells.default = (craneLibFor crossPkgs).devShell ({
            inputsFrom = [rust-xv6];
            packages = [
              config.treefmt.build.wrapper
              localPkgs.gnumake
              localPkgs.clang-tools
              localPkgs.guestfs-tools
            ];
          }
          // common);

        packages = {
          inherit rust-xv6 kernel programs tar2fs;
          default = localPkgs.writeScriptBin "qemu-script" ''
            #!/usr/bin/env zsh
            set -e
            FS="$(mktemp -p /tmp fs.XXXXXX.img)"
            cp ${fsImg} "$FS"
            echo "copied fs.img to $FS"
            trap 'rm -vf "$FS"' EXIT

            CPUS="''${CPUS:-3}"
            qemu-system-riscv64                                            \
                -machine virt -m 128M -smp "$CPUS" -nographic              \
                -global virtio-mmio.force-legacy=false                     \
                -drive file="$FS",if=none,format=raw,id=x0                 \
                -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0   \
                -kernel ${kernel} ${lib.optionalString (!enableOpenSBI) "-bios none"} "''${@[@]}"
          '';
        };
      };
    };
}
