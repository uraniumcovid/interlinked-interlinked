{
  description = "interlinked - global filesystem  plain-text link and tag scanner";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            
            # Build dependencies
            gcc
            pkg-config
            
            # Development tools
            git
          ];

          shellHook = ''
            echo "🦀 Interlinked development environment ready!"
            echo "Rust version: $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build    - Build the project"
            echo "  cargo run      - Run the indexer"
            echo "  cargo test     - Run tests"
            echo "  cargo clippy   - Run linter"
            echo ""
            echo "Usage: cargo run [directory] [--json]"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "interlinked-interlinked";
          version = "0.1.0";
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = with pkgs; [
            gcc
          ];

          meta = with pkgs.lib; {
            description = "File indexer for Obsidian-style links and tags";
            license = licenses.mit;
          };
        };
      });
}
