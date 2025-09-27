#!/bin/bash

# Release script for Local Code Browser
# This script helps create new version releases

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Local Code Browser Release Script${NC}"
echo

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}❌ Error: Not in a git repository${NC}"
    exit 1
fi

# Check if there are uncommitted changes
if ! git diff --quiet || ! git diff --staged --quiet; then
    echo -e "${YELLOW}⚠️  Warning: You have uncommitted changes${NC}"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep -o 'version = "[^"]*"' src-tauri/Cargo.toml | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+')
echo -e "${BLUE}📦 Current version: ${CURRENT_VERSION}${NC}"

# Ask for new version
read -p "Enter new version (e.g., 0.1.5): " NEW_VERSION

if [ -z "$NEW_VERSION" ]; then
    echo -e "${RED}❌ Error: Version cannot be empty${NC}"
    exit 1
fi

# Validate version format (basic check)
if ! echo "$NEW_VERSION" | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' > /dev/null; then
    echo -e "${RED}❌ Error: Invalid version format. Use format: x.y.z${NC}"
    exit 1
fi

echo -e "${BLUE}📝 New version: ${NEW_VERSION}${NC}"

# Confirm release
read -p "Create release v${NEW_VERSION}? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}ℹ️  Release cancelled${NC}"
    exit 0
fi

echo -e "${BLUE}🔄 Updating version numbers...${NC}"

# Update version in src-tauri/Cargo.toml
sed -i.bak "s/version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/g" src-tauri/Cargo.toml

# Update version in web/package.json
sed -i.bak "s/\"version\": \"${CURRENT_VERSION}\"/\"version\": \"${NEW_VERSION}\"/g" web/package.json

# Update version in root Cargo.toml (workspace)
sed -i.bak "s/version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/g" Cargo.toml

echo -e "${GREEN}✅ Version updated in all files${NC}"

# Commit changes
echo -e "${BLUE}💾 Committing version changes...${NC}"
git add src-tauri/Cargo.toml web/package.json Cargo.toml
git commit -m "chore: bump version to ${NEW_VERSION}"

# Create and push tag
echo -e "${BLUE}🏷️  Creating tag v${NEW_VERSION}...${NC}"
git tag "v${NEW_VERSION}"
git push origin main
git push origin "v${NEW_VERSION}"

echo -e "${GREEN}🎉 Release v${NEW_VERSION} created successfully!${NC}"
echo -e "${BLUE}📤 Pushed to origin with tag v${NEW_VERSION}${NC}"
echo -e "${YELLOW}ℹ️  GitHub Actions will now build and release the new version${NC}"
echo -e "${BLUE}🔗 Check releases at: https://github.com/NeuralEmpowerment/local-code-browser/releases${NC}"
