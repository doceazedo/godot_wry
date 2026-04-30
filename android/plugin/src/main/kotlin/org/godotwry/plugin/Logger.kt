// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
//
// Stamped from wry's android/kotlin/Logger.kt template — only changes are the
// concrete package and using a hard-coded debug flag (we don't have wry's
// BuildConfig.DEBUG injection here, and verbose log spew in release would be
// a regression vs the upstream desktop UX).

@file:Suppress("unused", "MemberVisibilityCanBePrivate")

package org.godotwry.plugin

import android.text.TextUtils
import android.util.Log

class Logger {
    companion object {
        private const val LOG_TAG_CORE = "GodotWry"

        fun tags(vararg subtags: String): String {
            return if (subtags.isNotEmpty()) {
                LOG_TAG_CORE + "/" + TextUtils.join("/", subtags)
            } else LOG_TAG_CORE
        }

        fun verbose(message: String) = Log.v(LOG_TAG_CORE, message)
        fun verbose(tag: String, message: String) = Log.v(tag, message)

        fun debug(message: String) = Log.d(LOG_TAG_CORE, message)
        fun debug(tag: String, message: String) = Log.d(tag, message)

        fun info(message: String) = Log.i(LOG_TAG_CORE, message)
        fun info(tag: String, message: String) = Log.i(tag, message)

        fun warn(message: String) = Log.w(LOG_TAG_CORE, message)
        fun warn(tag: String, message: String) = Log.w(tag, message)

        fun error(message: String) = Log.e(LOG_TAG_CORE, message, null)
        fun error(message: String, e: Throwable?) = Log.e(LOG_TAG_CORE, message, e)
        fun error(tag: String, message: String, e: Throwable?) = Log.e(tag, message, e)
    }
}
