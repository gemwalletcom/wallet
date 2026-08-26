package com.gemwallet.android.data.repositories.gemstone

import android.content.Context
import androidx.core.app.NotificationManagerCompat
import uniffi.gemstone.GemNotificationPermissions

class GemstoneNotificationPermissions(
    private val context: Context,
) : GemNotificationPermissions {
    override suspend fun requestPermissionsOrOpenSettings(): Boolean =
        NotificationManagerCompat.from(context).areNotificationsEnabled()
}
