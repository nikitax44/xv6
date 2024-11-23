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
        };

        crossPkgs = import nixpkgs {
          inherit crossSystem localSystem;
          overlays = [(import rust-overlay)];
        };

        inherit (nixpkgs) lib;

        craneLib = (crane.mkLib crossPkgs).overrideToolchain (p: p.rust-bin.nightly.latest.default);

        enableOpenSBI = false;

        prefixDrv = prefix: drv:
          crossPkgs.runCommandNoCCLocal "prefix-drv" {} ''
            mkdir $out
            cp -r ${drv} $out/${prefix}
          '';

        rust-xv6 = crossPkgs.callPackage ./rust-xv6 {
          inherit craneLib;
        };

        kernel = crossPkgs.stdenv.mkDerivation {
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

        programs = crossPkgs.stdenv.mkDerivation ({
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

        mkfs = localPkgs.stdenv.mkDerivation ({
            pname = "mkfs";
            version = "none";
            src = ./.;
            buildFlags = "mkfs";
            installPhase = ''
              install -Dm 0555 mkfs.elf $out/bin/mkfs
            '';
          }
          // common);

        newlib = crossPkgs.newlib.override {nanoizeNewlib = true;};
        platform = crossPkgs.stdenv.hostPlatform.config;
        NEWLIB = "${newlib}/${platform}";
        nativeBuildInputs = [
          localPkgs.stdenv.cc
          localPkgs.perl
          localPkgs.fd
          localPkgs.cmake
          localPkgs.unixtools.xxd
          (localPkgs.writeShellScriptBin "get-busybox" "cp ${busybox}/bin/busybox ./_busybox")
        ];
        buildInputs = [newlib];
        busybox = localPkgs.pkgsCross.riscv64.busybox.override {
          enableStatic = true;
          enableAppletSymlinks = false;
          enableMinimal = true;
        };

        common = {
          inherit NEWLIB buildInputs nativeBuildInputs;
          OPENSBI_ENABLED = lib.toUpper (lib.boolToString enableOpenSBI);
        };
      in {
        treefmt.config = import ./treefmt.nix;

        checks = {
          build-test = self'.packages.default;
        };

        devShells.default = craneLib.devShell ({
            inputsFrom = [rust-xv6];
            packages = [
              config.treefmt.build.wrapper
              localPkgs.gnumake
              localPkgs.clang-tools
            ];
          }
          // common);

        packages = {
          inherit rust-xv6 kernel programs mkfs;
          default = crossPkgs.stdenv.mkDerivation ({
              src = ./.;
              pname = "xv6";
              version = "none";
              preBuild = ''
                cp ${kernel} kernel/kernel
                cp ${programs} user/fs.tar
                cp ${mkfs}/bin/mkfs mkfs.elf
                chmod u+w kernel/kernel user/fs.tar mkfs.elf
              '';
              buildFlags = "-i";
              installPhase = ''
                mkdir -p $out/bin
                install -Dm 0444 kernel/kernel fs.img $out/
                install -Dm 0555 qemu-script $out/bin/
              '';
              dontStrip = true;
              meta.mainProgram = "qemu-script";
            }
            // common);
        };
      };
    };
}
