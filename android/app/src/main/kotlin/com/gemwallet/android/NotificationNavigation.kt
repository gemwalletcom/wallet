package com.gemwallet.android

import android.content.Intent
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.PushNotificationField
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualPositionRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.gemwallet.android.ui.navigation.routes.SupportRoute
import com.gemwallet.android.ui.navigation.routes.SwapPairRoute
import com.gemwallet.android.ui.navigation.routes.TransactionRoute
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemAssetsServiceInterface
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemPushNotification
import uniffi.gemstone.GemPushNotificationService
import uniffi.gemstone.GemPushNotificationServiceInterface
import uniffi.gemstone.GemWalletSessionServiceInterface
import javax.inject.Inject

class NotificationNavigation @Inject constructor(
    private val navigationService: GemNavigationServiceInterface,
    private val pushNotificationService: GemPushNotificationServiceInterface,
    private val walletSessionService: GemWalletSessionServiceInterface,
) {
    internal suspend fun prepareNavigation(intent: Intent): PendingNavigation.Routes {
        if (!intent.hasNotificationPayload()) {
            return PendingNavigation.Routes(emptyList())
        }
        val notificationType = intent.getStringExtra(PushNotificationField.Type.key) ?: return PendingNavigation.Routes(emptyList())
        val notification = pushNotificationService.parse(
            notificationType = notificationType,
            data = intent.getStringExtra(PushNotificationField.Data.key),
        ) ?: return PendingNavigation.Routes(emptyList())
        return prepareNavigation(notification)
    }

    internal suspend fun prepareNavigation(notification: GemPushNotification): PendingNavigation.Routes {
        val target = navigationService.openNotification(notification)
        target.walletId()?.let { walletId -> withContext(Dispatchers.IO) { walletSessionService.setCurrentWalletId(walletId) } }
        return target.destination()
    }
}

internal fun Intent.putNotificationPayload(type: String?, rawData: String?): Intent = apply {
    type?.let { putExtra(PushNotificationField.Type.key, it) }
    rawData?.let { putExtra(PushNotificationField.Data.key, it) }
}

internal fun Intent.hasNotificationPayload(): Boolean = runCatching { hasExtra(PushNotificationField.Type.key) || hasExtra(PushNotificationField.Data.key) }.getOrDefault(false)
