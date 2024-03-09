{ pkgs, lib, stdenv, rust }: 
pkgs.mkShell {
  inputsFrom = [ (pkgs.callPackage ./default.nix { inherit rust; }) ];

  buildInputs = with pkgs; [
    trunk
    sqlx-cli
  ];

  # Workaround for https://github.com/NixOS/nixpkgs/issues/166205
  env = lib.optionalAttrs stdenv.cc.isClang { 
    NIX_LDFLAGS = "-l${stdenv.cc.libcxx.cxxabi.libName}"; 
  };
}