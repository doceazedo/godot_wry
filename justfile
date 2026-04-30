#!/usr/bin/env just --justfile

os := if os() == "macos" { "macos" } else if os() == "windows" { "windows" } else { "linux" }
target := if os == "macos" { arch() + "-apple-darwin" } else if os == "windows" { arch() + "-pc-windows-msvc" } else { arch() + "-unknown-linux-gnu" }

default: build

set working-directory := 'rust'

build-all: build-macos-universal build-linux build-windows build-ios

build: 
	@echo "Building for {{os}} ({{target}})..."
	@just _build-{{os}}
	@just _copy-to-godot-{{os}}

copy-to-godot: build
	@echo "Copying files to Godot project..."
	@just _copy-to-godot-{{os}}

clean:
	cargo clean

_build-macos:
	cargo build --target {{target}} --locked --release
	mkdir -p ./target/{{target}}/release/libgodot_wry.framework/Resources
	mv ./target/{{target}}/release/libgodot_wry.dylib ./target/{{target}}/release/libgodot_wry.framework/libgodot_wry.dylib
	cp ../assets/Info.plist ./target/{{target}}/release/libgodot_wry.framework/Resources/Info.plist

_build-linux:
	cargo build --target {{target}} --locked --release

_build-windows:
	cargo build --target {{target}} --locked --release

_copy-to-godot-macos:
	mkdir -p ../godot/addons/godot_wry/bin/{{target}}
	cp -R ./target/{{target}}/release/libgodot_wry.framework ../godot/addons/godot_wry/bin/{{target}}

_copy-to-godot-linux:
	mkdir -p ../godot/addons/godot_wry/bin/{{target}}
	cp ./target/{{target}}/release/libgodot_wry.so ../godot/addons/godot_wry/bin/{{target}}/

_copy-to-godot-windows:
	mkdir -p ../godot/addons/godot_wry/bin/{{target}}
	cp ./target/{{target}}/release/godot_wry.dll ../godot/addons/godot_wry/bin/{{target}}/	

build-macos-universal:
	@echo "Building universal macOS binary..."
	cargo build --target aarch64-apple-darwin --locked --release
	cargo build --target x86_64-apple-darwin --locked --release
	mkdir -p ./target/release/libgodot_wry.framework/Resources
	lipo -create -output ./target/release/libgodot_wry.dylib ./target/aarch64-apple-darwin/release/libgodot_wry.dylib ./target/x86_64-apple-darwin/release/libgodot_wry.dylib
	mv ./target/release/libgodot_wry.dylib ./target/release/libgodot_wry.framework/libgodot_wry.dylib
	cp ../assets/Info.plist ./target/release/libgodot_wry.framework/Resources/Info.plist
	mkdir -p ../godot/addons/godot_wry/bin/universal-apple-darwin
	cp -R ./target/release/libgodot_wry.framework ../godot/addons/godot_wry/bin/universal-apple-darwin

build-linux:
	@echo "Building for Linux..."
	just os="linux" build

build-windows:
	@echo "Building for Windows..."
	just os="windows" build

# iOS: cross-compile from macOS only — iOS is never the host OS, so it does
# not flow through the `_build-{{os}}` / `_copy-to-godot-{{os}}` host-driven
# path. Static libs inside xcframework do NOT load on iOS — Godot's
# GDExtension uses dlopen at runtime and only finds dynamic frameworks under
# <App>.app/Frameworks/. The cdylib is wrapped in a real
# `libgodot_wry.framework` (dylib + Info.plist + @rpath install_name) and
# bundled into an xcframework with a single device slice.
build-ios:
	@echo "Building iOS xcframework..."
	cargo build --target aarch64-apple-ios --locked --release
	mkdir -p ./target/aarch64-apple-ios/release/libgodot_wry.framework
	cp ./target/aarch64-apple-ios/release/libgodot_wry.dylib ./target/aarch64-apple-ios/release/libgodot_wry.framework/libgodot_wry
	install_name_tool -id "@rpath/libgodot_wry.framework/libgodot_wry" ./target/aarch64-apple-ios/release/libgodot_wry.framework/libgodot_wry
	cp ../assets/Info.ios.plist ./target/aarch64-apple-ios/release/libgodot_wry.framework/Info.plist
	plutil -convert binary1 ./target/aarch64-apple-ios/release/libgodot_wry.framework/Info.plist
	xcodebuild -create-xcframework \
		-framework ./target/aarch64-apple-ios/release/libgodot_wry.framework \
		-output ./target/aarch64-apple-ios/release/libgodot_wry.xcframework
	mkdir -p ../godot/addons/godot_wry/bin/aarch64-apple-ios
	cp -R ./target/aarch64-apple-ios/release/libgodot_wry.xcframework ../godot/addons/godot_wry/bin/aarch64-apple-ios/