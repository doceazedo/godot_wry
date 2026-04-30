<a href="https://godotengine.org/asset-library/asset/3426">
  <img src="assets/splash.png" />
</a>

<p align="center">
  <img src="https://img.shields.io/static/v1?label=Godot&message=4.1%2B&color=478CBF&logo=godotengine">
  <img src="https://github.com/doceazedo/godot_wry/actions/workflows/build.yml/badge.svg">
  <a href="https://discord.gg/B9fWw3raZJ">
    <img src="https://img.shields.io/static/v1?label=Need%20help%3F&message=Join%20us%20on%20Discord!&color=5865F2&logo=discord">
  </a>
</p>

# Godot WRY

[WRY](https://github.com/tauri-apps/wry) is a cross-platform webview rendering library. This extension allows you to use the native webview in Godot to build browsers and GUIs with HTML, CSS and JavaScript.

## ✨ Features

- 🍃 Use the native system native webview (no extra dependencies!)
- 🌎 Load website URLs and local `res://` files
- 🧩 JavaScript ⇔ GDScript code integration
- 🚥 Mouse/keyboard input events forwarding

## ⛹️ Demo

<p align="center">
  <img src="assets/demo-cas.gif">
  Demo game UI available at "<a href="godot/addons/godot_wry/examples/character_creator_ui_demo">examples/character_creator_ui_demo</a>".
</p>

<details>
  <summary>📸 Other screenshots</summary>
  
  ![](assets/screenshot-7.png)
  ![](assets/screenshot-6.png)
  ![](assets/screenshot-4.png)
  ![](assets/screenshot-5.png)
  
</details>

## 💾 Installing

### Asset Library

The easiest way to install Godot WRY is through Godot's [Asset Library](https://godotengine.org/asset-library/asset/3426). You can install it via the editor by following these instructions:

1. Open your project in Godot 4.1 or later.
2. Go to the "📥 AssetLib" tab at the top of the editor.
3. Search for "Godot WRY".
4. Click on the Godot WRY extension and click **Download**.
5. In the configuration dialog, click **Install**.

### GitHub releases

1. Go to the [Releases](https://github.com/doceazedo/godot_wry/releases) page.
2. Download the latest release ZIP file (_not_ the source code).
3. Extract the contents into your project's "addons" folder (create one if it doesn't exist yet).

### Build from source

Use [just](https://github.com/casey/just) to build the extension and move the binaries to the Godot project folder:

```sh
$ just build
```

> **macOS 15+ local dev**: after `just build` (or `just build-macos-universal`),
> ad-hoc sign the framework or Godot will refuse to load it (SIGKILL by the
> CODESIGNING namespace). CI release builds use a real Developer ID; local
> builds need:
>
> ```sh
> $ scripts/sign_macos_local.sh
> ```

If you need a more in-depth guide on how to compile the project, check the [Building from source](https://godot-wry.doce.sh/contributing/compiling.html) documentation page.

## 📚 Documentation

Please refer to the [Docs](https://godot-wry.doce.sh) for API reference and in-depth guides on how to use Godot WRY.

## 🎯 Supported platforms

| Platform                        | Support           | Web engine                 |
| ------------------------------- | ----------------- | -------------------------- |
| **Windows (10, 11)**            | ✅ Supported      | WebView2 (Chromium)        |
| **Mac (Intel, Apple Sillicon)** | ✅ Supported      | WebKit                     |
| **Linux (X11)**                 | 🚧 Supported\*    | WebKitGTK                  |
| **Android**                     | ✅ Supported\*\*  | Android WebView (Chromium) |
| **iOS**                         | ✅ Supported      | WKWebView                  |
| **Browser/HTML5**               | ⏳ Planned        | —                          |

### Linux

[WebKitGTK](https://webkitgtk.org) is required for WRY to function on Linux. The package name may differ based on the operating system and Linux distribution.

\* X11 support only. Transparency is currently not supported. See [#17](https://github.com/doceazedo/godot_wry/issues/17).

### Android

\*\* Built and verified on the [`mobile-support`](https://github.com/MacacaGames/godot_wry/tree/mobile-support) branch. Requires Godot 4.2+ on the consumer side. Plugin ships as a v2 Android plugin AAR (`GodotWry-debug.aar` / `GodotWry-release.aar`) under `addons/godot_wry/bin/android/`, plus a `libgodot_wry.so` per ABI (`arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86`). Consumer projects must:

1. Enable **Use Gradle Build** in the Android export preset (Godot's prebuilt APK template skips AAR plugins).
2. Set `[rendering] shader_compiler/shader_cache/enabled=false` in `project.godot` if targeting Android emulator (BlueStacks / Android Studio AVD) — Goldfish OpenGL caches shader state across process restarts and breaks the second launch. Real devices don't need this.

**Upstream-tracking notes:** the Android build currently depends on two unstable behaviors that have fixes in flight upstream:

- **gdext-rust `experimental-threads` feature** (in `rust/Cargo.toml`) is the workaround for [gdext#1423](https://github.com/godot-rust/gdext/issues/1423). The fix landed as [PR #1574](https://github.com/godot-rust/gdext/pull/1574) on 2026-04-30 but isn't in a crates.io release yet (latest is 0.5.2 from 2026-04-28). Once a `>= 0.5.3` is published, drop the feature and pin to that version.
- **`shader_cache/enabled = false`** in the demo project is a workaround for the Goldfish OpenGL emulator shader-cache issue on Godot 4 GL Compatibility renderer (sibling of [godot#82419](https://github.com/godotengine/godot/issues/82419)). Real-device users don't need it.

### iOS

iOS uses `WKWebView` natively. The plugin's `EditorExportPlugin` automatically adds `-framework WebKit -framework UIKit -framework Foundation` to the Xcode link step. Targets `aarch64-apple-ios` (device); simulator slice is intentionally omitted (real-device development is the supported path). Min iOS 13.0.

### Mobile background reading

WRY itself has [mobile support](https://github.com/tauri-apps/wry/blob/dev/MOBILE.md) for the underlying webview. The Godot integration glue (JNI bridge, v2 plugin AAR, EditorExportPlugin scaffold, Vulkan / GL Compatibility caveats) is documented inline in `android/`, `rust/src/lib.rs`, and `godot/addons/godot_wry/godot_wry_export_plugin.gd`.

## ❌ Caveats

- Webview always renders on top
- Different browser engines across platforms
- No automatic dependency checks
- Godot 4.5+ "Embed Game on Next Play" must be **off** (see below)

You can learn more about these caveats on the [Caveats](https://godot-wry.doce.sh/about/caveats.html) documentation page.

### Game embedding (Godot 4.5+)

Godot 4.5 introduced the **"Embed Game on Next Play"** option (Game tab toggle,
or Editor / Project Settings → Run → Window Placement). When enabled, the
running game is hosted inside the editor's UI through `DisplayServerEmbedded`,
which does not own a real OS window — there is no `NSWindow` / `HWND` /
`UIView` to attach a native subview to. `DisplayServer.window_get_native_handle`
returns `0` and the extension panics with `Id<T> should never be null`.

This affects **any** GDExtension that needs to overlay a native view (webview,
native video, native map, etc.), not just godot_wry. The fix is to **turn
embed mode off** so the game opens in its own native window. As an alternative
for modal popups, attach the WebView to a separate `Window` node — that node
opens its own real OS window even while the main game is embedded (see
`addons/godot_wry/examples/example.tscn`).

## 🤝 Contribute

Your help is most welcome regardless of form! Check out the [How to contribute](https://godot-wry.doce.sh/contributing/how-to-contribute.html) page for all ways you can contribute to the project. For example, [suggest a new feature](https://github.com/doceazedo/godot_wry/issues/new?template=feature_request.md), [report a problem/bug](https://github.com/doceazedo/godot_wry/issues/new?template=bug_report.md), [submit a pull request](https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/about-pull-requests), or simply use the project and comment your experience.

See the [Roadmap](https://godot-wry.doce.sh/about/roadmap.html) documentation page for an idea of how the project should evolve.

## 🎫 License

The Godot WRY extension is licensed under [MIT](/LICENSE). WRY is licensed under [Apache-2.0/MIT](https://github.com/tauri-apps/wry/blob/dev/LICENSE.spdx).

## 🧪 Similar projects

Below is a list of interesting similar projects:

- [gdcef](https://github.com/Lecrapouille/gdcef/tree/godot-4.x) — Open-source, powered by Chromium (CEF)
- [Godot-HTML](https://github.com/Decapitated/Godot-HTML) — Open-source, powered by Ultralight (WebKit)
- [godot-webview](https://godotwebview.com/) — Commercial, powered by Qt6 (Chromium)
