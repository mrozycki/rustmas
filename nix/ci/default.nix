{ pkgs, lib, stdenv, rust }:
stdenv.mkDerivation rec {
    name = "ci-runner";
    src = ../..;
    nativeBuildInputs = [ rust ];
    PATH = lib.makeBinPath nativeBuildInputs;

    phases = [ "unpackPhase" "buildPhase" "installPhase" ];
    buildPhase = ''
        nix/ci/ci-runner.sh
    '';
    installPhase = ''
        mkdir $out
    '';
}
