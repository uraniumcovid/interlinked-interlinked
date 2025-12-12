{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
    
    # Build dependencies
    gcc
    pkg-config
    
    # Optional development tools
    git
  ];

  shellHook = ''
    echo "🦀 Rust development environment ready!"
    echo "Available commands:"
    echo "  cargo build    - Build the project"
    echo "  cargo run      - Run the indexer"
    echo "  cargo test     - Run tests"
    echo "  cargo clippy   - Run linter"
    echo ""
    echo "Usage: cargo run [directory] [--json]"
  '';
}