let
  pkgs = import <nixpkgs> {};
  tpkg = pkgs.pkgsCross.riscv64-embedded;
  newlib = tpkg.newlib.override {nanoizeNewlib = true;};
in
  tpkg.mkShell {
    nativeBuildInputs = [pkgs.gcc pkgs.perl pkgs.gnumake pkgs.clang-tools];
    buildInputs = [newlib];
    TOOLPREFIX = "riscv64-none-elf-";
    EXTRA_CFLAGS = "-I${newlib}/riscv64-none-elf/include";
    EXTRA_LDFLAGS = "${newlib}/riscv64-none-elf/lib/libc.a";
  }
