{ pkgs ? import <nixpkgs> {}
}:
pkgs.mkShell {
  name="dev-environment";
  buildInputs = [
    pkgs.rustup
    pkgs.just
  ];
  shellHook = ''
    echo "Start developing..."
  '';
} 
