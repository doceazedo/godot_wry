#!/usr/bin/env just --justfile

os := if os() == "macos" { "macos" } else if os() == "windows" { "windows" } else { "linux" }
target := if os == "macos" { arch() + "-apple-darwin" } else if os == "windows" { arch() + "-pc-windows-msvc" } else { arch() + "-unknown-linux-gnu" }

default: build

set working-directory := 'rust'

build-all: build-macos-universal build-linux build-windows build-ios build-android build-android-aar

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

# Android: cross-compile from any host (macOS/Linux) via cargo-ndk. Requires
# `cargo install cargo-ndk` and an NDK 25.x install with ANDROID_NDK_HOME set.
# Produces one libgodot_wry.so per ABI laid out under
# ../godot/addons/godot_wry/bin/android/<abi>/ — Godot's GDExtension loader
# expects exactly that directory naming (arm64-v8a, armeabi-v7a, x86_64, x86).
build-android:
	@echo "Building Android .so for arm64-v8a, armeabi-v7a, x86_64, x86..."
	cargo ndk \
		-t arm64-v8a \
		-t armeabi-v7a \
		-t x86_64 \
		-t x86 \
		-o ./target/android/jniLibs \
		build --locked --release
	mkdir -p ../godot/addons/godot_wry/bin/android/arm64-v8a
	mkdir -p ../godot/addons/godot_wry/bin/android/armeabi-v7a
	mkdir -p ../godot/addons/godot_wry/bin/android/x86_64
	mkdir -p ../godot/addons/godot_wry/bin/android/x86
	cp ./target/android/jniLibs/arm64-v8a/libgodot_wry.so      ../godot/addons/godot_wry/bin/android/arm64-v8a/
	cp ./target/android/jniLibs/armeabi-v7a/libgodot_wry.so    ../godot/addons/godot_wry/bin/android/armeabi-v7a/
	cp ./target/android/jniLibs/x86_64/libgodot_wry.so         ../godot/addons/godot_wry/bin/android/x86_64/
	cp ./target/android/jniLibs/x86/libgodot_wry.so            ../godot/addons/godot_wry/bin/android/x86/

# Android plugin .aar — the Kotlin/Java side of the GodotPlugin contract.
# Builds godot_wry/android/plugin into both a Debug AAR and a Release AAR
# matching the m4gr3d template convention. Output filenames come from
# `archivesBaseName = pluginName` in plugin/build.gradle:
#   GodotWry-debug.aar   — used when exporting an Android debug APK
#   GodotWry-release.aar — used when exporting an Android release APK
# Both copied into ../godot/addons/godot_wry/bin/android/. The split lets
# `_get_android_libraries(platform, debug)` in the EditorExportPlugin pick
# the right variant (which, in turn, makes Godot ship the right
# `BuildConfig.DEBUG` flag and any debug-only logging in the plugin).
#
# Each AAR contains:
#   - jni/<abi>/libgodot_wry.so     (gradle pulls from godot/addons/godot_wry/bin/android/<abi>/)
#   - assets/addons/godot_wry/WRY.gdextension  (gradle preBuild copy task)
#   - classes.jar  (compiled Kotlin from src/main/kotlin/...)
#   - AndroidManifest.xml  (with `org.godotengine.plugin.v2.GodotWry` tag)
#
# The user MUST enable "Use Gradle Build" in their Android export preset for
# the plugin discovery to work — Godot's prebuilt APK templates skip user
# .aar files entirely. Requires JDK 17 + Android SDK Build-Tools.
build-android-aar:
	@echo "Building Android plugin AAR (Debug + Release)..."
	cd ../android && JAVA_HOME="${JAVA_HOME:-/opt/homebrew/opt/openjdk@17}" ./gradlew :plugin:assembleDebug :plugin:assembleRelease --no-daemon
	mkdir -p ../godot/addons/godot_wry/bin/android
	cp ../android/plugin/build/outputs/aar/GodotWry-debug.aar   ../godot/addons/godot_wry/bin/android/GodotWry-debug.aar
	cp ../android/plugin/build/outputs/aar/GodotWry-release.aar ../godot/addons/godot_wry/bin/android/GodotWry-release.aar
	@echo "Plugin AARs ->"
	@echo "  ../godot/addons/godot_wry/bin/android/GodotWry-debug.aar"
	@echo "  ../godot/addons/godot_wry/bin/android/GodotWry-release.aar"