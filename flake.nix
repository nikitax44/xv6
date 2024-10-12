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

        craneLib = (crane.mkLib crossPkgs).overrideToolchain (p: p.rust-bin.nightly.latest.default);

        rust-xv6 = crossPkgs.callPackage ./kernel/rust {inherit craneLib;};

        newlib = crossPkgs.newlib.override {nanoizeNewlib = true;};
        platform = crossPkgs.stdenv.hostPlatform.config;

        TOOLPREFIX = "${crossPkgs.stdenv.cc}/bin/${platform}-";
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
      in {
        treefmt.config = import ./treefmt.nix;

        checks = {
          build-test = self'.packages.default;
          inherit rust-xv6;
        };

        devShells.default = craneLib.devShell {
          inputsFrom = [rust-xv6];
          packages = [
            config.treefmt.build.wrapper
            localPkgs.gnumake
            localPkgs.clang-tools
          ];
          inherit NEWLIB TOOLPREFIX buildInputs nativeBuildInputs;
        };

        packages.default = crossPkgs.stdenv.mkDerivation {
          src = ./.;
          pname = "xv6";
          version = "none";
          preBuild = ''
            cp ${rust-xv6}/lib/librust_xv6.a kernel/
          '';
          inherit NEWLIB TOOLPREFIX buildInputs nativeBuildInputs;
          installPhase = ''
            mkdir -p $out/bin
            install -Dm 0444 kernel/kernel fs.img $out/
            install -Dm 0555 qemu-script $out/bin/
          '';
          OPENSBI_ENABLED = false;
          RUST_KALLOC_ENABLE = false;
          meta.mainProgram = "qemu-script";
        };
      };
    };
}
