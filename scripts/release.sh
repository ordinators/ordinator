#!/bin/bash

# Release script for Ordinator
# Usage: ./scripts/release.sh [patch|minor|major]

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

if [ ! -f "Cargo.toml" ]; then
    print_error "This script must be run from the project root directory"
    exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
    print_error "Git working directory is not clean. Please commit or stash your changes."
    exit 1
fi

CURRENT_VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
print_status "Current version: $CURRENT_VERSION"

BUMP_TYPE=${1:-patch}
if [[ ! "$BUMP_TYPE" =~ ^(patch|minor|major)$ ]]; then
    print_error "Invalid bump type. Use: patch, minor, or major"
    exit 1
fi

IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR=${VERSION_PARTS[0]}
MINOR=${VERSION_PARTS[1]}
PATCH=${VERSION_PARTS[2]}

case $BUMP_TYPE in
    patch)
        PATCH=$((PATCH + 1))
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"
print_status "New version: $NEW_VERSION"

echo
print_warning "This will:"
echo "  1. Update version in Cargo.toml to $NEW_VERSION"
echo "  2. Commit the version change"
echo "  3. Create and push tag v$NEW_VERSION"
echo "  4. Trigger GitHub Actions release workflow"
echo
read -p "Continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_status "Release cancelled"
    exit 0
fi

print_status "Updating version in Cargo.toml..."
sed -i.bak "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

print_status "Committing version change..."
git add Cargo.toml
git commit -m "chore: bump version to $NEW_VERSION"

print_status "Creating tag v$NEW_VERSION..."
git tag "v$NEW_VERSION"

print_status "Pushing changes and tag..."
git push origin master
git push origin "v$NEW_VERSION"

print_status "Release process started!"
print_status "GitHub Actions will now:"
echo "  - Build binaries for macOS and Linux"
echo "  - Run all tests and checks"
echo "  - Create GitHub release with binaries"
echo
print_status "Monitor the release at: https://github.com/ceterus/ordinator/actions" 