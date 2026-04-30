// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
//
// Stamped from wry's android/kotlin/PermissionHelper.kt — verbatim aside from
// the package name. Kept around because the stripped RustWebChromeClient may
// still want hasPermissions() one day; PermissionHelper itself doesn't pull
// in any Activity-state assumptions, so it's free to ship.

package org.godotwry.plugin

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.ActivityCompat
import java.util.ArrayList

object PermissionHelper {
    fun hasPermissions(context: Context?, permissions: Array<String>): Boolean {
        for (perm in permissions) {
            if (ActivityCompat.checkSelfPermission(
                    context!!,
                    perm
                ) != PackageManager.PERMISSION_GRANTED
            ) {
                return false
            }
        }
        return true
    }

    fun hasDefinedPermission(context: Context, permission: String): Boolean {
        val requestedPermissions = getManifestPermissions(context)
        if (!requestedPermissions.isNullOrEmpty()) {
            val requestedPermissionsList = listOf(*requestedPermissions)
            val requestedPermissionsArrayList = ArrayList(requestedPermissionsList)
            if (requestedPermissionsArrayList.contains(permission)) {
                return true
            }
        }
        return false
    }

    private fun getManifestPermissions(context: Context): Array<String>? {
        var requestedPermissions: Array<String>? = null
        try {
            val pm = context.packageManager
            val packageInfo = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                pm.getPackageInfo(
                    context.packageName,
                    PackageManager.PackageInfoFlags.of(PackageManager.GET_PERMISSIONS.toLong())
                )
            } else {
                @Suppress("DEPRECATION")
                pm.getPackageInfo(context.packageName, PackageManager.GET_PERMISSIONS)
            }
            if (packageInfo != null) {
                requestedPermissions = packageInfo.requestedPermissions
            }
        } catch (_: Exception) {
        }
        return requestedPermissions
    }
}
