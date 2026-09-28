package com.gemwallet.android

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemPushNotification
import uniffi.gemstone.GemPushNotificationServiceInterface
import uniffi.gemstone.GemWalletSessionServiceInterface
import javax.inject.Inject

class NotificationNavigation @Inject constructor(
    private val navigationService: GemNavigationServiceInterface,
    private val pushNotificationService: GemPushNotificationServiceInterface,
    private val walletSessionService: GemWalletSessionServiceInterface,
) {
    internal suspend fun prepareNavigation(notificationType: String, data: String?): PendingNavigation.Routes {
        val notification = pushNotificationService.parse(notificationType = notificationType, data = data) ?: return PendingNavigation.Routes(emptyList())
        return prepareNavigation(notification)
    }

    internal suspend fun prepareNavigation(notification: GemPushNotification): PendingNavigation.Routes {
        val target = navigationService.openNotification(notification)
        target.walletId()?.let { walletId -> withContext(Dispatchers.IO) { walletSessionService.setCurrentWalletId(walletId) } }
        return target.destination()
    }
}
