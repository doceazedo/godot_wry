#!/usr/bin/env bash
# Ad-hoc sign the locally-built macOS framework.
#
# macOS 15+ refuses to dlopen unsigned dylibs from outside an app bundle, and
# SIGKILLs the host process (`Code Signature Invalid`, namespace CODESIGNING
# code 2). Production releases get a real Developer ID signature via the
# `ci_sign_macos.ps1` step in `.github/workflows/build.yml`. For local
# development on macOS 15+ run this script after each `just build` /
# `just build-macos-universal` so the Godot editor can load the extension.
#
# Usage:
#   scripts/sign_macos_local.sh

set -euo pipefail

cd "$(dirname "$0")/.."
FRAMEWORK="godot/addons/godot_wry/bin/universal-apple-darwin/libgodot_wry.framework"

if [ ! -d "$FRAMEWORK" ]; then
    echo "Framework not found at $FRAMEWORK" >&2
    echo "Run \`just build-macos-universal\` (or \`just build\` on macOS) first." >&2
    exit 1
fi

codesign --force --sign - --timestamp=none "$FRAMEWORK"
codesign -dv "$FRAMEWORK" 2>&1 | grep -E "Format|Signature"
echo "Done. Framework is ad-hoc signed and Godot should be able to load it now."
