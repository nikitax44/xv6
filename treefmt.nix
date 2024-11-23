{
  projectRootFile = ".git/config";
  settings = {
    formatter.cmake-format = {
      options = ["--dangle-parens" "--enable-sort" "--line-width" "120"];
    };
  };
  programs = {
    # nix
    alejandra.enable = true;
    deadnix.enable = true;
    statix.enable = true;

    # C
    clang-format.enable = true;

    # CMake
    cmake-format.enable = true;

    # rust
    rustfmt.enable = true;

    # workflow
    actionlint.enable = true;
    yamlfmt.enable = true;

    # json
    jsonfmt.enable = true;

    # general
    keep-sorted.enable = true;
  };
}
