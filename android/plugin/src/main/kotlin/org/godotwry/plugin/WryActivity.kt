// Custom WryActivity for Godot embedding.
//
// wry's stock WryActivity (wry-0.50.5/src/android/kotlin/WryActivity.kt) is
// an `abstract class WryActivity : AppCompatActivity()` that the host app is
// expected to subclass as its main entry-point Activity. That model assumes
// you control the Activity — wry calls `activity.setContentView(webview)` to
// take over the entire screen, and the upstream RustWebChromeClient calls
// `activity.registerForActivityResult(...)` which only works on a real
// OS-managed LifecycleOwner.
//
// In Godot's case the OS-managed Activity is `GodotActivity` (we don't
// control its class), and we want the WebView to be an overlay above the
// game's OpenGL surface, NOT a replacement.
//
// Solution: make WryActivity a plain object that wraps GodotActivity via
// `ContextWrapper`. wry's JNI calls into us only require:
//   - `setWebView(RustWebView)`         — wry hands us the webview ref
//   - `setContentView(View)`            — wry asks us to install it as content
//   - `getAppClass(String): Class<*>`   — wry asks for plugin classpath lookup
//   - `getVersion(): String`            — wry asks for the WebView version
// Everything else (RustWebView / RustWebViewClient constructors take a
// Context — we satisfy that via ContextWrapper inheritance) is delegated
// to the host GodotActivity through ContextWrapper.
//
// We override `setContentView` to call `hostActivity.addContentView(view, …)`
// which adds the WebView ABOVE GodotActivity's existing content (the
// SurfaceView Godot draws into) instead of replacing it.

@file:Suppress("unused")

package org.godotwry.plugin

import android.annotation.SuppressLint
import android.app.Activity
import android.content.ContextWrapper
import android.os.Build
import android.view.View
import android.webkit.WebView
import android.widget.FrameLayout

class WryActivity(private val hostActivity: Activity) : ContextWrapper(hostActivity) {
    @Suppress("MemberVisibilityCanBePrivate")
    var webView: RustWebView? = null
        private set

    fun setWebView(webView: RustWebView) {
        this.webView = webView
    }

    fun getAppClass(name: String): Class<*> {
        // wry's mod.rs `find_class` calls this with names like
        // "org/godotwry/plugin/RustWebView" (slash-form) — note that
        // wry replaces slashes with dots before invoking, so by the time
        // we receive `name` it's already in dot-form.
        return Class.forName(name)
    }

    fun setContentView(view: View) {
        // Run on the UI thread because wry's MainPipe callback may be on
        // its own looper-fd thread (see wry-0.50.5 src/android/mod.rs:132-145).
        hostActivity.runOnUiThread {
            val params = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
            hostActivity.addContentView(view, params)
        }
    }

    val version: String
        @SuppressLint("WebViewApiAvailability", "ObsoleteSdkInt")
        get() {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                return WebView.getCurrentWebViewPackage()?.versionName ?: ""
            }
            // Pre-O fallback — Godot 4.6 minSdk is 24 so this branch is dead in
            // practice, but kept for parity with upstream WryActivity.
            return ""
        }

    // Lifecycle hooks invoked by GodotWryPlugin's onMainPause/onMainResume/
    // onMainDestroy. wry doesn't strictly require these (the desktop wry
    // doesn't have explicit suspend/resume hooks either) but Android WebView
    // leaks JS timers and audio/video continues if you don't call onPause.
    fun onPause() {
        hostActivity.runOnUiThread { webView?.onPause() }
    }

    fun onResume() {
        hostActivity.runOnUiThread { webView?.onResume() }
    }

    fun onDestroy() {
        hostActivity.runOnUiThread {
            webView?.destroy()
            webView = null
        }
    }
}
