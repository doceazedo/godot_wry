// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Stripped variant of wry's android/kotlin/RustWebChromeClient.kt for
// Godot embedding. The upstream version pulls in:
//   - registerForActivityResult() — needs a real Activity that's a
//     LifecycleOwner registered via the OS lifecycle. Our WryActivity is a
//     plain Java object instantiated by GodotWryPlugin, NOT an OS-managed
//     Activity, so it can't satisfy that contract.
//   - File picker / camera capture — drags in FileProvider + temp files +
//     androidx.activity ActivityResult plumbing, all of which depend on
//     LifecycleOwner state.
//   - JS alert/confirm/prompt dialogs — pulls in AlertDialog tied to the
//     Activity context.
//
// For a Godot HUD overlay, none of those features are needed. We keep just
// `onReceivedTitle` (so wry's `document_title_changed_handler` keeps
// working) and `onConsoleMessage` (so JS console output is visible in
// `adb logcat`). The constructor still takes a `WryActivity` because wry's
// `android_setup()` instantiates RustWebChromeClient via the JNI signature
// `(L<package>/WryActivity;)V` — see wry-0.50.5/src/android/mod.rs:117-122.

package org.godotwry.plugin

import android.webkit.ConsoleMessage
import android.webkit.WebChromeClient
import android.webkit.WebView

class RustWebChromeClient(@Suppress("unused") val activity: WryActivity) : WebChromeClient() {

    override fun onReceivedTitle(view: WebView, title: String) {
        handleReceivedTitle(view, title)
    }

    override fun onConsoleMessage(consoleMessage: ConsoleMessage): Boolean {
        val tag: String = Logger.tags("Console")
        if (consoleMessage.message() != null) {
            val msg = "${consoleMessage.sourceId()}:${consoleMessage.lineNumber()} ${consoleMessage.message()}"
            when (consoleMessage.messageLevel().name.uppercase()) {
                "ERROR" -> Logger.error(tag, msg, null)
                "WARNING" -> Logger.warn(tag, msg)
                "TIP", "DEBUG" -> Logger.debug(tag, msg)
                else -> Logger.info(tag, msg)
            }
        }
        return true
    }

    companion object {
        init {
            System.loadLibrary("godot_wry")
        }
    }

    private external fun handleReceivedTitle(webview: WebView, title: String)
}
