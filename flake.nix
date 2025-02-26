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

        craneLib = (crane.mkLib crossPkgs).overrideToolchain (p:
          p.rust-bin.nightly.latest.default.override {
            extensions = ["rust-src"];
          });

        enableOpenSBI = false;
        qemuFlags = lib.optionalString (!enableOpenSBI) "-bios none";

        kernel-headers = ./include;

        rust-xv6 = crossPkgs.callPackage ./rust-xv6 {
          inherit craneLib kernel-headers;
        };

        libkernel = crossPkgs.stdenvNoLibs.mkDerivation {
          name = "libkernel";
          src = ./kernel;
          inherit nativeBuildInputs;

          cmakeFlags = ["-DCMAKE_C_FLAGS=-I${kernel-headers}"];
          buildFlags = "KernelLib";
          installPhase = ''
            install -m 0444 libKernelLib.a $out
          '';
        };

        kernel = crossPkgs.stdenvNoLibs.mkDerivation {
          pname = "kernel";
          inherit (rust-xv6) version;
          src = ./boot;
          inherit nativeBuildInputs;

          cmakeFlags = [
            "-DCMAKE_C_FLAGS=-I${kernel-headers}"
            "-DKERNEL=${libkernel}"
            "-DRUST_XV6=${rust-xv6}/lib/librust_xv6.a"
            "-DOPENSBI_ENABLED=${toString enableOpenSBI}"
          ];
          buildFlags = "kernel";
          installPhase = ''
            install -m 0444 kernel $out
          '';
          dontStrip = true;
        };

        programs = crossPkgs.stdenvNoLibs.mkDerivation ({
            name = "programs.tar";
            src = ./user;

            cmakeFlags = ["-DCMAKE_C_FLAGS=-I${kernel-headers}" "-DNEWLIB=${NEWLIB}"];
            buildFlags = "Programs";
            installPhase = ''
              install -m 0444 fs.tar $out
            '';
          }
          // common);

        mkfs = localPkgs.stdenv.mkDerivation {
          pname = "mkfs";
          version = "none";
          src = ./mkfs;
          buildPhase = ''
            gcc -o mkfs.elf mkfs.c -I${kernel-headers}
          '';
          installPhase = ''
            install -Dm 0555 mkfs.elf $out/bin/mkfs
          '';
        };

        qemu-script = localPkgs.substitute {
          name = "qemu-script";
          src = ./qemu-script.tmpl;
          substitutions = [
            "--subst-var-by"
            "QEMU_FLAGS"
            qemuFlags
          ];
        };

        newlib = crossPkgs.newlib.override {nanoizeNewlib = true;};
        platform = crossPkgs.stdenvNoLibs.hostPlatform.config;
        NEWLIB = "${newlib}/${platform}";
        nativeBuildInputs = [
          localPkgs.stdenvNoLibs.cc
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
              localPkgs.rust-bindgen
            ];
          }
          // common);

        packages = {
          inherit rust-xv6 libkernel kernel programs mkfs qemu-script;
          default =
            localPkgs.runCommandNoCCLocal "xv6" {
              meta.mainProgram = "qemu-script";
            } ''
              tar -xf ${programs}
              cp ${./README} ./README
              ${mkfs}/bin/mkfs fs.img *
              mkdir -p $out/bin
              install -Tm 0444 ${kernel} $out/kernel
              install -Dm 0444 fs.img $out/
              install -Dm 0555 ${qemu-script} $out/bin/qemu-script
            '';
        };
      };
    };
}
