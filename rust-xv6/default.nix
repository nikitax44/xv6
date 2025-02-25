{
  libiconv,
  lib,
  fd,
  pkg-config,
  stdenvNoLibs,
  craneLib,
  kernel-headers,
  rust-bindgen,
  CARGO_PROFILE ? "release",
}: let
  xv6-bindgen = stdenvNoLibs.mkDerivation {
    name = "xv6-binds";
    src = kernel-headers;
    nativeBuildInputs = [fd rust-bindgen];
    configurePhase = ''
      fd -e h --base-directory . --strip-cwd-prefix -x echo "#include \"{}\"" > everything.c
    '';
    buildPhase = ''
      bindgen everything.c -o xv6_binds.rs \
          --use-core --wrap-static-fns --wrap-unsafe-ops --emit-diagnostics \
          --experimental --no-derive-copy --no-debug trapframe \
          -- -D __ASSEMBLER__ -I . -nostdinc
    '';
    installPhase = ''
      mkdir $out
      mv xv6_binds.rs $out/
    '';
  };

  commonArgs = {
    src = craneLib.cleanCargoSource ./.;
    strictDeps = true;

    postUnpack = ''
      cp ${xv6-bindgen}/xv6_binds.rs source/
    '';

    # See: https://github.com/NixOS/nixpkgs/pull/146583
    depsBuildBuild = [];

    nativeBuildInputs =
      [
        pkg-config
        stdenvNoLibs.cc
        fd
      ]
      ++ lib.optionals stdenvNoLibs.buildPlatform.isDarwin [
        libiconv
      ];

    buildInputs = [
    ];

    inherit CARGO_PROFILE;

    # See: https://doc.rust-lang.org/cargo/reference/config.html#target
    CARGO_TARGET_RISCV64GC_UNKNOWN_NONE_ELF_LINKER = "${stdenvNoLibs.cc.targetPrefix}ld";

    cargoExtraArgs = "--target riscv64gc-unknown-none-elf"; # --features talc";
    cargoCheckExtraArgs = "";

    HOST_CC = "${stdenvNoLibs.cc.nativePrefix}cc";
    TARGET_CC = "${stdenvNoLibs.cc.targetPrefix}cc";
  };
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
  clippy = craneLib.cargoClippy (commonArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "-- --deny warnings";
    });
  package = craneLib.buildPackage (commonArgs // {cargoArtifacts = clippy;});
in
  package
