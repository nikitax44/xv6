{
  craneLib,
  CARGO_PROFILE ? "release",
}: let
  commonArgs = {
    src = ./.;
    strictDeps = true;

    # See: https://github.com/NixOS/nixpkgs/pull/146583
    depsBuildBuild = [];

    inherit CARGO_PROFILE;
    cargoCheckExtraArgs = "";
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
