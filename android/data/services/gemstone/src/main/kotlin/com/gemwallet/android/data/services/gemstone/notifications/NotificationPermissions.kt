package com.gemwallet.android.data.services.gemstone.notifications

import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import com.gemwallet.android.application.notifications.NotificationPermissionRequests
import com.gemwallet.android.model.NotificationsAvailable
import uniffi.gemstone.GemNotificationPermissions
import uniffi.gemstone.GemNotificationPrompt
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPreferencesServiceInterface

class GemstoneNotificationPermissions(
    private val context: Context,
    private val requests: NotificationPermissionRequests,
    private val preferences: GemPreferencesServiceInterface,
    private val notificationsAvailable: NotificationsAvailable,
) : GemNotificationPermissions {

    override fun isAvailable(): Boolean = notificationsAvailable

    override suspend fun requestPermissionsOrOpenSettings(): Boolean = when (preferences.notificationPrompt(isGranted())) {
        GemNotificationPrompt.ENABLE -> true

        GemNotificationPrompt.REQUEST -> requests.request()

        GemNotificationPrompt.OPEN_SETTINGS -> {
            openSettings()
            false
        }
    }

    private fun isGranted(): Boolean = NotificationManagerCompat.from(context).areNotificationsEnabled() && hasNotificationPermission()

    private fun openSettings() {
        context.startActivity(notificationSettingsIntent(context.packageName, notificationSettingsTarget(hasNotificationPermission())))
    }

    private fun hasNotificationPermission(): Boolean = Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
        ContextCompat.checkSelfPermission(context, android.Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED
}

internal enum class NotificationSettingsTarget {
    AppDetails,
    NotificationSettings,
}

internal fun notificationSettingsTarget(permissionGranted: Boolean): NotificationSettingsTarget = if (permissionGranted) {
    NotificationSettingsTarget.NotificationSettings
} else {
    NotificationSettingsTarget.AppDetails
}

internal fun notificationSettingsIntent(packageName: String, target: NotificationSettingsTarget): Intent {
    val intent = when (target) {
        NotificationSettingsTarget.NotificationSettings -> Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).putExtra(Settings.EXTRA_APP_PACKAGE, packageName)
        NotificationSettingsTarget.AppDetails -> Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.fromParts("package", packageName, null))
    }
    return intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
}
