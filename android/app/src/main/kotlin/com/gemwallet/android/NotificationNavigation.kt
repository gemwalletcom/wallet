package com.gemwallet.android

import android.content.Intent
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.ext.toAssetId
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
import com.gemwallet.android.ui.navigation.routes.TransactionDetailsRoute
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemAssetsServiceInterface
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemPushNotification
import uniffi.gemstone.GemPushNotificationService
import uniffi.gemstone.GemPushNotificationServiceInterface
import javax.inject.Inject

class NotificationNavigation @Inject constructor(
    private val getWallet: GetWallet,
    private val createTransaction: CreateTransaction,
    private val navigationService: GemNavigationServiceInterface,
    private val pushNotificationService: GemPushNotificationServiceInterface,
) {
    suspend fun prepareNavigation(intent: Intent): List<NavKey> {
        if (!intent.hasNotificationPayload()) {
            return emptyList()
        }
        val notificationType = intent.getStringExtra(PushNotificationField.Type.key) ?: return emptyList()
        val notification = pushNotificationService.parse(
            notificationType = notificationType,
            data = intent.getStringExtra(PushNotificationField.Data.key),
        ) ?: return emptyList()
        return prepareNavigation(notification)
    }

    internal suspend fun prepareNavigation(notification: GemPushNotification): List<NavKey> {
        if (notification is GemPushNotification.Transaction) {
            storeTransaction(WalletId(notification.walletId), notification.assetId.toAssetId(), notification.transaction.toPrimitives())
        }
        return navigationService.openNotification(notification).routes()
    }

    private suspend fun storeTransaction(walletId: WalletId, assetId: AssetId?, transaction: Transaction) {
        val wallet = getWallet(walletId).firstOrNull() ?: return
        createTransaction.createNotificationTransaction(wallet = wallet, assetId = assetId ?: return, transaction = transaction)
    }
}

internal fun Intent.putNotificationPayload(type: String?, rawData: String?): Intent = apply {
    type?.let { putExtra(PushNotificationField.Type.key, it) }
    rawData?.let { putExtra(PushNotificationField.Data.key, it) }
}

internal fun Intent.hasNotificationPayload(): Boolean = hasExtra(PushNotificationField.Type.key) || hasExtra(PushNotificationField.Data.key)
