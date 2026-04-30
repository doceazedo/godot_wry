// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
//
// Stamped from wry's android/kotlin/Ipc.kt — verbatim aside from package name
// and the {{library}} placeholder being concretized to "godot_wry" (matches
// the cdylib output name from cargo-ndk).

@file:Suppress("unused")

package org.godotwry.plugin

import android.webkit.JavascriptInterface

class Ipc(val webViewClient: RustWebViewClient) {
    @JavascriptInterface
    fun postMessage(message: String?) {
        message?.let { m ->
            // Track URL via the webview client (page-load updates currentUrl)
            // because WebView::getUrl() must run on the main thread.
            this.ipc(webViewClient.currentUrl, m)
        }
    }

    companion object {
        init {
            System.loadLibrary("godot_wry")
        }
    }

    private external fun ipc(url: String, message: String)
}
