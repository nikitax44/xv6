let
  pkgs = import <nixpkgs> {};
  tpkg = pkgs.pkgsCross.riscv64-embedded;
in
  tpkg.mkShell {
    nativeBuildInputs = [pkgs.gcc pkgs.perl pkgs.gnumake];
    buildInputs = [(tpkg.newlib.override {nanoizeNewlib = true;})];
    TOOLPREFIX = "riscv64-none-elf-";
    # TOOLPREFIX = "riscv64-unknown-linux-gnu-";
  }
