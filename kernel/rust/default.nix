{
  libiconv,
  lib,
  pkg-config,
  stdenv,
  craneLib,
}:
craneLib.buildPackage {
  src = craneLib.cleanCargoSource ./.;
  strictDeps = true;

  # See: https://github.com/NixOS/nixpkgs/pull/146583
  depsBuildBuild = [];

  nativeBuildInputs =
    [
      pkg-config
      stdenv.cc
    ]
    ++ lib.optionals stdenv.buildPlatform.isDarwin [
      libiconv
    ];

  buildInputs = [
  ];

  # See: https://doc.rust-lang.org/cargo/reference/config.html#target
  CARGO_TARGET_RISCV64GC_UNKNOWN_NONE_ELF_LINKER = "${stdenv.cc.targetPrefix}ld";

  cargoExtraArgs = "--target riscv64gc-unknown-none-elf";
  cargoCheckExtraArgs = "";

  HOST_CC = "${stdenv.cc.nativePrefix}cc";
  TARGET_CC = "${stdenv.cc.targetPrefix}cc";
}
