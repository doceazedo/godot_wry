// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
//
// Stamped from wry's android/kotlin/RustWebView.kt — package + library name
// concretized, no behavioral changes. {{class-init}} placeholder left empty
// (we have no per-instance init beyond what wry sets).

@file:Suppress("unused", "SetJavaScriptEnabled")

package org.godotwry.plugin

import android.annotation.SuppressLint
import android.webkit.*
import android.content.Context
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature
import kotlin.collections.Map

@SuppressLint("RestrictedApi")
class RustWebView(
    context: Context,
    val initScripts: Array<String>,
    val id: String
) : WebView(context) {
    val isDocumentStartScriptEnabled: Boolean

    init {
        settings.javaScriptEnabled = true
        settings.domStorageEnabled = true
        settings.setGeolocationEnabled(true)
        settings.databaseEnabled = true
        settings.mediaPlaybackRequiresUserGesture = false
        settings.javaScriptCanOpenWindowsAutomatically = true

        // Set transparent background up front so the very first composition
        // can show Godot's 3D scene through the HUD's empty regions. wry's
        // MainPipe also calls `setBackgroundColor((0,0,0,0))` later when the
        // node was built with `transparent = true`, but that runs after the
        // first frame on Android and has produced an opaque flash on some
        // chromium versions. The hardware layer hint helps the WebView's
        // chromium-side compositor produce a real RGBA backing buffer that
        // composites over Godot's GL Compatibility SurfaceView — without
        // it, the WebView ended up with no rendered surface at all and the
        // whole window went black.
        setBackgroundColor(android.graphics.Color.TRANSPARENT)
        setLayerType(android.view.View.LAYER_TYPE_HARDWARE, null)

        if (WebViewFeature.isFeatureSupported(WebViewFeature.DOCUMENT_START_SCRIPT)) {
            isDocumentStartScriptEnabled = true
            for (script in initScripts) {
                WebViewCompat.addDocumentStartJavaScript(this, script, setOf("*"))
            }
        } else {
            isDocumentStartScriptEnabled = false
        }
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        // Bring this WebView in front of Godot's GL Compatibility GLSurfaceView,
        // which by default punches a hole through the host window. With a
        // high translationZ + bringToFront + elevation we force the
        // hardware compositor to honour our overlay.
        translationZ = 1000f
        elevation = 1000f
        bringToFront()
        (parent as? android.view.ViewGroup)?.requestLayout()
        Logger.info("RustWebView attached: w=$width h=$height vis=$visibility parent=${parent?.javaClass?.simpleName} alpha=$alpha translationZ=$translationZ elevation=$elevation")
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        Logger.info("RustWebView size changed: ${oldw}x${oldh} -> ${w}x${h}")
    }

    fun loadUrlMainThread(url: String) {
        post { loadUrl(url) }
    }

    fun loadUrlMainThread(url: String, additionalHttpHeaders: Map<String, String>) {
        post { loadUrl(url, additionalHttpHeaders) }
    }

    override fun loadUrl(url: String) {
        if (!shouldOverride(url)) {
            super.loadUrl(url)
        }
    }

    override fun loadUrl(url: String, additionalHttpHeaders: Map<String, String>) {
        if (!shouldOverride(url)) {
            super.loadUrl(url, additionalHttpHeaders)
        }
    }

    fun loadHTMLMainThread(html: String) {
        post {
            super.loadData(html, "text/html", null)
        }
    }

    fun evalScript(id: Int, script: String) {
        post {
            super.evaluateJavascript(script) { result ->
                onEval(id, result)
            }
        }
    }

    fun clearAllBrowsingData() {
        try {
            super.getContext().deleteDatabase("webviewCache.db")
            super.getContext().deleteDatabase("webview.db")
            super.clearCache(true)
            super.clearHistory()
            super.clearFormData()
        } catch (ex: Exception) {
            Logger.error("Unable to clear browsing data: " + ex.message)
        }
    }

    fun getCookies(url: String): String {
        val cookieManager = CookieManager.getInstance()
        return cookieManager.getCookie(url)
    }

    private external fun shouldOverride(url: String): Boolean
    private external fun onEval(id: Int, result: String)
}
