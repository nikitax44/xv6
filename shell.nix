let
  pkgs = import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/3b05df1d13c1b315cecc610a2f3180f6669442f0.tar.gz";
    sha256 = "sha256:1dr7kfdl4wvxhml4hd9k77xszl55vbjbb6ssirs2qv53mgw8c24w";
  }) {};
in
  pkgs.pkgsCross.riscv64-embedded.mkShell {
    nativeBuildInputs = [pkgs.gcc pkgs.perl pkgs.gnumake];
    TOOLPREFIX = "riscv64-none-elf-";
    # TOOLPREFIX = "riscv64-unknown-linux-gnu-";
  }
