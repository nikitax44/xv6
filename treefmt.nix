{
  projectRootFile = ".git/config";
  settings = {
  };
  programs = {
    # nix
    alejandra.enable = true;
    deadnix.enable = true;
    statix.enable = true;

    # C
    clang-format.enable = true;

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
