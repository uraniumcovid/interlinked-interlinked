#!/bin/bash

# Interlinked Installation Script
# This script builds and installs the interlinked-interlinked binary

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo is not installed!"
    print_status "Please install Rust from: https://rustup.rs/"
    exit 1
fi

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -f "src/main.rs" ]]; then
    print_error "This script must be run from the interlinked-interlinked project directory!"
    exit 1
fi

print_status "Building interlinked-interlinked..."

# Build the project in release mode
cargo build --release

# Check if build succeeded
if [[ $? -ne 0 ]]; then
    print_error "Build failed!"
    exit 1
fi

# Determine installation directory
if [[ "$EUID" -eq 0 ]]; then
    # Running as root - install system-wide
    INSTALL_DIR="/usr/local/bin"
    print_status "Installing system-wide to $INSTALL_DIR (requires root)"
else
    # Install to user's local bin
    INSTALL_DIR="$HOME/.local/bin"
    print_status "Installing to user directory: $INSTALL_DIR"
    
    # Create local bin directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"
fi

# Copy the binary
BINARY_NAME="interlinked"
SOURCE_BINARY="target/release/interlinked-interlinked"
TARGET_BINARY="$INSTALL_DIR/$BINARY_NAME"

print_status "Copying binary to $TARGET_BINARY..."
cp "$SOURCE_BINARY" "$TARGET_BINARY"

# Make it executable
chmod +x "$TARGET_BINARY"

print_success "Binary installed successfully!"

# Check if the installation directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    print_warning "Installation directory $INSTALL_DIR is not in your PATH!"
    
    # Determine shell config file
    SHELL_CONFIG=""
    if [[ "$SHELL" == *"zsh"* ]]; then
        SHELL_CONFIG="$HOME/.zshrc"
    elif [[ "$SHELL" == *"bash"* ]]; then
        if [[ -f "$HOME/.bashrc" ]]; then
            SHELL_CONFIG="$HOME/.bashrc"
        else
            SHELL_CONFIG="$HOME/.bash_profile"
        fi
    elif [[ "$SHELL" == *"fish"* ]]; then
        SHELL_CONFIG="$HOME/.config/fish/config.fish"
    fi
    
    if [[ -n "$SHELL_CONFIG" ]]; then
        print_status "To add $INSTALL_DIR to your PATH, run:"
        if [[ "$SHELL" == *"fish"* ]]; then
            echo -e "${BLUE}    echo 'set -gx PATH \$PATH $INSTALL_DIR' >> $SHELL_CONFIG${NC}"
        else
            echo -e "${BLUE}    echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> $SHELL_CONFIG${NC}"
        fi
        print_status "Then restart your shell or run: source $SHELL_CONFIG"
    else
        print_status "Add $INSTALL_DIR to your PATH manually"
    fi
else
    print_success "Installation directory is already in PATH!"
fi

# Test installation
if command -v "$BINARY_NAME" &> /dev/null; then
    print_success "Installation verified! You can now run: $BINARY_NAME"
    print_status "Try these commands:"
    echo -e "  ${BLUE}$BINARY_NAME --help${NC}                 # Show help"
    echo -e "  ${BLUE}$BINARY_NAME ~/Documents -i${NC}         # Interactive mode"
    echo -e "  ${BLUE}$BINARY_NAME ~/Notes --cache-info${NC}    # Check cache status"
else
    print_warning "Command '$BINARY_NAME' not found in PATH. You may need to restart your shell."
fi

print_success "Installation complete!"