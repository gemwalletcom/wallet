package com.gemwallet.android

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemWalletSessionServiceInterface
import javax.inject.Inject

class NotificationNavigation @Inject constructor(private val navigationService: GemNavigationServiceInterface, private val walletSessionService: GemWalletSessionServiceInterface) {
    internal suspend fun prepareNavigation(notificationType: String, data: String?): PendingNavigation.Routes {
        val target = navigationService.openNotification(notificationType = notificationType, data = data)
        target.walletId()?.let { walletId -> withContext(Dispatchers.IO) { walletSessionService.setCurrentWalletId(walletId) } }
        return target.destination()
    }
}
