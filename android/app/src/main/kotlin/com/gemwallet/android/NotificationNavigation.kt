package com.gemwallet.android

import android.content.Intent
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.application.assets.cases.SyncMissingAssets
import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.PushNotificationField
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.FiatInputRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualPositionRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.gemwallet.android.ui.navigation.routes.SupportRoute
import com.gemwallet.android.ui.navigation.routes.SwapPairRoute
import com.gemwallet.android.ui.navigation.routes.TransactionDetailsRoute
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemPushNotification
import uniffi.gemstone.GemPushNotificationService
import javax.inject.Inject

class NotificationNavigation @Inject constructor(
    private val getSession: GetSession,
    private val setCurrentWallet: SetCurrentWallet,
    private val getWallet: GetWallet,
    private val createTransaction: CreateTransaction,
    private val syncMissingAssets: SyncMissingAssets,
    private val assetsService: GemAssetsService,
    private val pushNotificationService: GemPushNotificationService,
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
        return when (notification) {
            is GemPushNotification.Asset -> prepareAssetRoute(notification.assetId.toAssetId())
            is GemPushNotification.PriceAlert -> prepareAssetRoute(notification.assetId.toAssetId())
            is GemPushNotification.BuyAsset -> {
                val assetId = notification.assetId.toAssetId() ?: return emptyList()
                prepareAssets(assetId)
                listOf(FiatInputRoute(assetId))
            }
            is GemPushNotification.FiatTransaction -> prepareWalletAssetRoutes(WalletId(notification.walletId), notification.assetId.toAssetId())
            is GemPushNotification.Stake -> prepareWalletAssetRoutes(WalletId(notification.walletId), notification.assetId.toAssetId())
            is GemPushNotification.SwapAsset -> {
                val fromAssetId = notification.fromAssetId.toAssetId() ?: return emptyList()
                val toAssetId = notification.toAssetId.toAssetId() ?: return emptyList()
                prepareAssets(fromAssetId, toAssetId)
                listOf(SwapPairRoute(fromAssetId, toAssetId))
            }
            is GemPushNotification.Transaction -> prepareTransactionRoutes(
                walletId = WalletId(notification.walletId),
                assetId = notification.assetId.toAssetId() ?: return emptyList(),
                transaction = notification.transaction.decodeJson(),
            )
            GemPushNotification.Rewards -> listOf(ReferralRoute())
            GemPushNotification.Support -> listOf(SupportRoute)
            GemPushNotification.Test -> emptyList()
        }
    }

    private suspend fun prepareAssetRoute(assetId: AssetId?): List<NavKey> {
        if (assetId == null) {
            return emptyList()
        }
        prepareAssets(assetId)
        return listOf(AssetRoute(assetId))
    }

    private suspend fun prepareAssets(vararg assetIds: AssetId) {
        syncMissingAssets.syncMissingAssets(assetIds.toList())
    }

    private suspend fun prepareWalletAssetRoutes(walletId: WalletId, assetId: AssetId?): List<NavKey> {
        val assetId = assetId ?: return emptyList()
        val wallet = getWallet(walletId).firstOrNull() ?: return emptyList()
        val asset = assetsService.openWalletAsset(wallet.toJson(), assetId.toIdentifier())?.toPrimitives() ?: return emptyList()
        selectWallet(wallet)
        return listOf(AssetRoute(asset.id))
    }

    private suspend fun prepareTransactionRoutes(walletId: WalletId, assetId: AssetId, transaction: Transaction): List<NavKey> {
        val wallet = getWallet(walletId).firstOrNull() ?: return emptyList()
        val asset = createTransaction.createNotificationTransaction(
            wallet = wallet,
            assetId = assetId,
            transaction = transaction,
        ) ?: return emptyList()
        selectWallet(wallet)
        val transactionRoute = TransactionDetailsRoute(transaction.id)
        if (asset.type != AssetType.PERPETUAL) {
            return listOf(AssetRoute(asset.id), transactionRoute)
        }
        return listOf(PerpetualRoute, PerpetualPositionRoute(asset.id), transactionRoute)
    }

    private suspend fun selectWallet(wallet: Wallet) {
        if (getSession().firstOrNull()?.wallet?.id != wallet.id) {
            setCurrentWallet.setCurrentWallet(wallet.id)
        }
    }
}

internal fun Intent.putNotificationPayload(type: String?, rawData: String?): Intent = apply {
    type?.let { putExtra(PushNotificationField.Type.key, it) }
    rawData?.let { putExtra(PushNotificationField.Data.key, it) }
}

internal fun Intent.hasNotificationPayload(): Boolean {
    return hasExtra(PushNotificationField.Type.key) || hasExtra(PushNotificationField.Data.key)
}
