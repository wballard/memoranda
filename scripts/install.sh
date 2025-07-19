#!/bin/bash
set -e

# Memoranda Installation Script
# This script downloads and installs the latest version of memoranda

# Configuration
REPO_OWNER="wballard"
REPO_NAME="memoranda"
BINARY_NAME="memoranda"
INSTALL_DIR="$HOME/.local/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
error() {
    echo -e "${RED}Error: $1${NC}" >&2
    exit 1
}

info() {
    echo -e "${BLUE}Info: $1${NC}"
}

success() {
    echo -e "${GREEN}Success: $1${NC}"
}

warn() {
    echo -e "${YELLOW}Warning: $1${NC}"
}

# Detect OS and architecture
detect_platform() {
    local os
    local arch
    
    case "$(uname -s)" in
        Darwin)
            os="apple-darwin"
            ;;
        Linux)
            os="unknown-linux-gnu"
            ;;
        MINGW* | MSYS* | CYGWIN*)
            os="pc-windows-msvc"
            ;;
        *)
            error "Unsupported operating system: $(uname -s)"
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64 | amd64)
            arch="x86_64"
            ;;
        arm64 | aarch64)
            arch="aarch64"
            ;;
        *)
            error "Unsupported architecture: $(uname -m)"
            ;;
    esac
    
    echo "${arch}-${os}"
}

# Get the latest release version
get_latest_version() {
    local api_url="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"
    
    if command -v curl >/dev/null 2>&1; then
        curl -s "$api_url" | grep '"tag_name"' | cut -d '"' -f 4
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$api_url" | grep '"tag_name"' | cut -d '"' -f 4
    else
        error "Neither curl nor wget is available. Please install one of them."
    fi
}

# Download and install memoranda
install_memoranda() {
    local platform="$1"
    local version="$2"
    local download_url
    local archive_name
    
    # Determine file extension
    if [[ "$platform" == *"windows"* ]]; then
        archive_name="${BINARY_NAME}-${platform}.zip"
        binary_name="${BINARY_NAME}.exe"
    else
        archive_name="${BINARY_NAME}-${platform}.tar.gz"
        binary_name="${BINARY_NAME}"
    fi
    
    download_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${version}/${archive_name}"
    
    info "Downloading memoranda ${version} for ${platform}..."
    info "Download URL: ${download_url}"
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # Download the archive
    if command -v curl >/dev/null 2>&1; then
        curl -L -o "$archive_name" "$download_url" || error "Failed to download memoranda"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$archive_name" "$download_url" || error "Failed to download memoranda"
    else
        error "Neither curl nor wget is available. Please install one of them."
    fi
    
    # Extract the archive
    info "Extracting archive..."
    if [[ "$archive_name" == *.zip ]]; then
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$archive_name" || error "Failed to extract archive"
        else
            error "unzip is not available. Please install it."
        fi
    else
        tar -xzf "$archive_name" || error "Failed to extract archive"
    fi
    
    # Create installation directory
    mkdir -p "$INSTALL_DIR"
    
    # Move binary to installation directory
    info "Installing memoranda to ${INSTALL_DIR}..."
    mv "$binary_name" "$INSTALL_DIR/" || error "Failed to install memoranda"
    chmod +x "${INSTALL_DIR}/${binary_name}"
    
    # Cleanup
    cd - >/dev/null
    rm -rf "$temp_dir"
    
    success "memoranda ${version} installed successfully!"
}

# Check if memoranda is in PATH
check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "The installation directory $INSTALL_DIR is not in your PATH."
        echo "To add it to your PATH, run one of the following commands:"
        echo
        echo "For bash/zsh:"
        echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
        echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.zshrc"
        echo
        echo "For fish:"
        echo "  fish_add_path $INSTALL_DIR"
        echo
        echo "Then restart your shell or run 'source ~/.bashrc' (or ~/.zshrc)"
    fi
}

# Verify installation
verify_installation() {
    if [[ -x "${INSTALL_DIR}/${BINARY_NAME}" ]]; then
        success "Installation verified!"
        info "Run '${INSTALL_DIR}/${BINARY_NAME} --version' to check the version"
        info "Run '${INSTALL_DIR}/${BINARY_NAME} doctor' to verify setup"
        echo
        echo "To get started:"
        echo "  1. Run 'memoranda doctor' to check your setup"
        echo "  2. Add memoranda to your MCP configuration"
        echo "  3. Start using memo tools in Claude Code!"
    else
        error "Installation verification failed"
    fi
}

# Main installation flow
main() {
    echo "🚀 Memoranda Installation Script"
    echo "=============================="
    echo
    
    # Detect platform
    local platform
    platform=$(detect_platform)
    info "Detected platform: ${platform}"
    
    # Get latest version
    info "Fetching latest release information..."
    local version
    version=$(get_latest_version)
    
    if [[ -z "$version" ]]; then
        error "Failed to get latest version information"
    fi
    
    info "Latest version: ${version}"
    
    # Check if already installed
    if command -v memoranda >/dev/null 2>&1; then
        local current_version
        current_version=$(memoranda --version 2>/dev/null | cut -d ' ' -f 2 || echo "unknown")
        if [[ "$current_version" == "${version#v}" ]]; then
            success "memoranda ${version} is already installed and up to date!"
            exit 0
        else
            info "Updating from version ${current_version} to ${version}"
        fi
    fi
    
    # Install memoranda
    install_memoranda "$platform" "$version"
    
    # Check PATH
    check_path
    
    # Verify installation
    verify_installation
    
    echo
    success "Installation complete! 🎉"
}

# Allow script to be sourced for testing
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi