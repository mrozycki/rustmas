{ pkgs, lib, stdenv, rust }:
let
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in rustPlatform.buildRustPackage rec {
  pname = "rustmas";
  version = "0.1.0";

  src = ../.;
  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "animation-api-0.1.0" = "sha256-X4Qu0EhN9R6rCBtuoqlQmtkH5V0Cyw9jL9xG70+PgaI=";
    };
  };

  nativeBuildInputs = [
    rustPlatform.bindgenHook
    pkgs.pkg-config
  ];

  buildInputs = [
    pkgs.opencv
  ] ++ lib.optional stdenv.isDarwin [
    pkgs.darwin.apple_sdk.frameworks.AppKit
  ];

  # Workaround for https://github.com/NixOS/nixpkgs/issues/166205
  env = lib.optionalAttrs stdenv.cc.isClang { 
    NIX_LDFLAGS = "-l${stdenv.cc.libcxx.cxxabi.libName}"; 
  };
}