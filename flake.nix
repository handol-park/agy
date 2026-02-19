{
  description = "Development environment for agy (Rust agent framework)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
            rust-analyzer
          ];

          shellHook = ''
            export RUST_BACKTRACE=1
            export CARGO_HOME="$PWD/.cargo"
            echo "Rust dev shell ready. Run: cargo check && cargo test"
          '';
        };
      });
}
