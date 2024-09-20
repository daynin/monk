{ pkgs ? import <nixpkgs> {}
}:
pkgs.mkShell {
  name="dev-environment";
  buildInputs = [
    pkgs.rustup
    pkgs.just
  ];
  shellHook = ''
    export PATH=$PATH:~/.cargo/bin
    echo "Start developing..."
  '';
} 
