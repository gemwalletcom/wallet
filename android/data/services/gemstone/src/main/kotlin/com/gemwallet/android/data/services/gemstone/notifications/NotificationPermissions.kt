package com.gemwallet.android.data.services.gemstone.notifications

import android.content.Context
import android.content.Intent
import android.provider.Settings
import androidx.core.app.NotificationManagerCompat
import uniffi.gemstone.GemNotificationPermissions

class GemstoneNotificationPermissions(
    private val context: Context,
) : GemNotificationPermissions {
    override suspend fun requestPermissionsOrOpenSettings(): Boolean {
        if (NotificationManagerCompat.from(context).areNotificationsEnabled()) {
            return true
        }
        context.startActivity(
            Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
                .putExtra(Settings.EXTRA_APP_PACKAGE, context.packageName)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        )
        return false
    }
}
