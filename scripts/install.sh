#!/bin/bash

# Ordinator Install Script
# This script installs Ordinator on macOS

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

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install Homebrew if not present
install_homebrew() {
    if ! command_exists brew; then
        print_status "Homebrew not found. Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        
        # Add Homebrew to PATH for current session
        if [[ -f "/opt/homebrew/bin/brew" ]]; then
            eval "$(/opt/homebrew/bin/brew shellenv)"
        elif [[ -f "/usr/local/bin/brew" ]]; then
            eval "$(/usr/local/bin/brew shellenv)"
        fi
        
        print_success "Homebrew installed successfully"
    else
        print_status "Homebrew already installed"
    fi
}

# Function to install Ordinator via Homebrew
install_ordinator_homebrew() {
    print_status "Installing Ordinator via Homebrew..."
    
    # Add the ordinator tap if it doesn't exist
    if ! brew tap | grep -q "ordinators/ordinator"; then
        print_status "Adding ordinator tap..."
        brew tap ordinators/ordinator
    fi
    
    # Install ordinator
    brew install ordinator
    
    print_success "Ordinator installed successfully via Homebrew"
}

# Function to install Ordinator from source
install_ordinator_source() {
    print_status "Installing Ordinator from source..."
    
    # Check if Rust is installed
    if ! command_exists cargo; then
        print_error "Rust is required to build Ordinator from source"
        print_status "Please install Rust first: https://rustup.rs/"
        exit 1
    fi
    
    # Clone and build ordinator
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    print_status "Cloning Ordinator repository..."
    git clone https://github.com/ordinators/ordinator.git
    cd ordinator
    
    print_status "Building Ordinator..."
    cargo build --release
    
    # Install to /usr/local/bin
    sudo cp target/release/ordinator /usr/local/bin/
    
    # Clean up
    cd /
    rm -rf "$temp_dir"
    
    print_success "Ordinator installed successfully from source"
}

# Main installation function
main() {
    print_status "Starting Ordinator installation..."
    
    # Check if we're on macOS
    if [[ "$OSTYPE" != "darwin"* ]]; then
        print_error "This installer is for macOS only"
        exit 1
    fi
    
    # Install Homebrew if needed
    install_homebrew
    
    # Try to install via Homebrew first
    if command_exists brew; then
        if brew tap | grep -q "ordinators/ordinator" || brew search ordinator >/dev/null 2>&1; then
            install_ordinator_homebrew
        else
            print_warning "Ordinator not available in Homebrew, installing from source..."
            install_ordinator_source
        fi
    else
        print_warning "Homebrew installation failed, installing from source..."
        install_ordinator_source
    fi
    
    # Verify installation
    if command_exists ordinator; then
        print_success "Ordinator installation completed successfully!"
        print_status "You can now use 'ordinator --help' to see available commands"
    else
        print_error "Installation failed. Please check the output above for errors."
        exit 1
    fi
}

# Run main function
main "$@" 