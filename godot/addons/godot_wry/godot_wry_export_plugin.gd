@tool
extends EditorExportPlugin


func _get_name() -> String:
	return "GodotWry"


func _supports_platform(platform: EditorExportPlatform) -> bool:
	return platform.get_os_name() in ["iOS", "Android"]


func _export_begin(features: PackedStringArray, _is_debug: bool, _path: String, _flags: int) -> void:
	if not features.has("ios"):
		return
	# wry's iOS path uses WKWebView (objc2-web-kit) and UIKit. Without these flags,
	# Xcode link of the iOS Godot app fails with "Undefined symbol _OBJC_CLASS_$_WKWebView" etc.
	add_ios_linker_flags("-framework WebKit -framework UIKit -framework Foundation")


func _get_android_libraries(platform: EditorExportPlatform, debug: bool) -> PackedStringArray:
	# Godot's `getAddonsDirectory` fileTree(*.aar) only scans the addons-folder
	# top level (non-recursive `include` glob), AND Godot's "copy addons into
	# build" step explicitly drops `bin/` subdirectories. Either way the AAR
	# under addons/godot_wry/bin/android/ would be silently skipped. Declaring
	# it here pushes the path through `plugins_local_binaries` project property
	# → `implementation files(...)` in the gradle build, which bypasses the
	# addons-folder copy entirely (the AAR is referenced by abs path from
	# res://). Use Gradle Build must still be enabled in the export preset
	# for ANY of this to run.
	#
	# Debug / release split (matches the m4gr3d template convention):
	# `just build-android-aar` produces both GodotWry-debug.aar and
	# GodotWry-release.aar. The `debug` flag is the export preset's debug
	# selection (`--export-debug` vs `--export-release`); pick the matching
	# AAR so `BuildConfig.DEBUG` and any debug-only behavior in the plugin
	# (chromium remote-debugging via WebView, extra logcat verbosity) flow
	# through correctly.
	if platform.get_os_name() != "Android":
		return PackedStringArray()
	var aar_name := "GodotWry-debug.aar" if debug else "GodotWry-release.aar"
	return PackedStringArray(["res://addons/godot_wry/bin/android/" + aar_name])


func _get_android_dependencies(platform: EditorExportPlatform, _debug: bool) -> PackedStringArray:
	# Maven coords for AAR transitive deps. Godot consumes our AAR via
	# `implementation files(...)` (a flat-file include) which does NOT pull
	# in `implementation` dependencies declared in the AAR's POM —
	# AndroidX Webkit / AppCompat / Activity etc would otherwise be missing
	# at runtime. Symptom: NoClassDefFoundError for `androidx/webkit/WebViewFeature`
	# when the wry RustWebView constructor tries to call
	# `WebViewFeature.isFeatureSupported(DOCUMENT_START_SCRIPT)` during init.
	# Versions track plugin/build.gradle's `dependencies {}` block.
	if platform.get_os_name() != "Android":
		return PackedStringArray()
	return PackedStringArray([
		"androidx.appcompat:appcompat:1.7.0",
		"androidx.webkit:webkit:1.12.1",
		"androidx.activity:activity:1.9.3",
	])
