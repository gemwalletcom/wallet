package com.gemwallet.android.data.services.gemstone.notifications

import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.provider.Settings
import androidx.core.content.ContextCompat
import androidx.core.app.NotificationManagerCompat
import com.gemwallet.android.application.notifications.NotificationPermissionRequests
import uniffi.gemstone.GemNotificationPermissions
import uniffi.gemstone.GemNotificationPrompt
import uniffi.gemstone.GemPreferencesService

class GemstoneNotificationPermissions(
    private val context: Context,
    private val requests: NotificationPermissionRequests,
    private val preferences: GemPreferencesService,
) : GemNotificationPermissions {

    override suspend fun requestPermissionsOrOpenSettings(): Boolean =
        when (preferences.notificationPrompt(isGranted())) {
            GemNotificationPrompt.ENABLE -> true
            GemNotificationPrompt.REQUEST -> requests.request()
            GemNotificationPrompt.OPEN_SETTINGS -> {
                openSettings()
                false
            }
        }

    private fun isGranted(): Boolean = NotificationManagerCompat.from(context).areNotificationsEnabled() &&
        (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
            ContextCompat.checkSelfPermission(context, android.Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED)

    private fun openSettings() {
        context.startActivity(
            Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
                .putExtra(Settings.EXTRA_APP_PACKAGE, context.packageName)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        )
    }
}
