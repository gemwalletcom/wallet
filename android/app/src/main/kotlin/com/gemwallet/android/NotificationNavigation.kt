package com.gemwallet.android

import android.content.Intent
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.assets.coordinators.EnsureWalletAssets
import com.gemwallet.android.application.assets.coordinators.GetAssetById
import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.cases.parseNotificationData
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.getAssociatedAssetIds
import com.gemwallet.android.model.PushNotificationData
import com.gemwallet.android.model.PushNotificationField
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.FiatInputRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualPositionRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.gemwallet.android.ui.navigation.routes.SupportRoute
import com.gemwallet.android.ui.navigation.routes.SwapPairRoute
import com.gemwallet.android.ui.navigation.routes.TransactionDetailsRoute
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.firstOrNull
import javax.inject.Inject

class NotificationNavigation @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val walletsRepository: WalletsRepository,
    private val saveTransactions: SaveTransactions,
    private val prefetchAssets: PrefetchAssets,
    private val ensureWalletAssets: EnsureWalletAssets,
    private val getAssetById: GetAssetById,
) {
    suspend fun prepareNavigation(intent: Intent): List<NavKey> {
        if (!intent.hasNotificationPayload()) {
            return emptyList()
        }
        val type = intent.getStringExtra(PushNotificationField.Type.key)
        val rawData = intent.getStringExtra(PushNotificationField.Data.key)
        return prepareNavigation(type = type, data = parseNotificationData(type, rawData))
    }

    internal suspend fun prepareNavigation(type: String?, data: PushNotificationData?): List<NavKey> {
        return when (val payload = data ?: parseNotificationData(type, rawData = null) ?: return emptyList()) {
            is PushNotificationData.Asset -> {
                prepareAssets(payload.assetId)
                listOf(AssetRoute(payload.assetId))
            }
            is PushNotificationData.BuyAsset -> {
                prepareAssets(payload.assetId)
                listOf(FiatInputRoute(payload.assetId))
            }
            is PushNotificationData.WalletAsset -> {
                prepareWallet(payload.walletId, listOf(payload.assetId)) ?: return emptyList()
                listOf(AssetRoute(payload.assetId))
            }
            is PushNotificationData.Stake -> {
                prepareWallet(payload.walletId, listOf(payload.assetId)) ?: return emptyList()
                listOf(AssetRoute(payload.assetId))
            }
            is PushNotificationData.Swap -> {
                prepareAssets(payload.fromAssetId, payload.toAssetId)
                listOf(SwapPairRoute(payload.fromAssetId, payload.toAssetId))
            }
            is PushNotificationData.Transaction -> prepareTransactionRoutes(payload)
            PushNotificationData.Reward -> listOf(ReferralRoute())
            PushNotificationData.Support -> listOf(SupportRoute)
        }
    }

    private suspend fun prepareAssets(vararg assetIds: AssetId) {
        prefetchAssets.prefetchAssets(assetIds.toList())
    }

    private suspend fun prepareTransaction(data: PushNotificationData.Transaction): Boolean {
        val assetIds = (data.transaction.getAssociatedAssetIds() + data.assetId).distinct()
        prepareWallet(data.walletId, assetIds) ?: return false
        saveTransactions.saveTransactions(data.walletId, listOf(data.transaction))
        return true
    }

    private suspend fun prepareTransactionRoutes(data: PushNotificationData.Transaction): List<NavKey> {
        if (!prepareTransaction(data)) {
            return emptyList()
        }
        val transactionRoute = TransactionDetailsRoute(data.transaction.id)
        val asset = getAssetById(data.assetId).firstOrNull() ?: return emptyList()
        if (asset.type != AssetType.PERPETUAL) {
            return listOf(AssetRoute(asset.id), transactionRoute)
        }
        return listOf(PerpetualRoute, PerpetualPositionRoute(data.assetId), transactionRoute)
    }

    private suspend fun prepareWallet(walletId: WalletId, assetIds: List<AssetId>): Wallet? {
        val wallet = walletsRepository.getWallet(walletId).firstOrNull() ?: return null
        prefetchAssets.prefetchAssets(assetIds)
        ensureWalletAssets.ensureWalletAssets(wallet, assetIds)
        if (sessionRepository.session().firstOrNull()?.wallet?.id != wallet.id) {
            sessionRepository.setWallet(wallet)
        }
        return wallet
    }
}

internal fun Intent.putNotificationPayload(type: String?, rawData: String?): Intent = apply {
    type?.let { putExtra(PushNotificationField.Type.key, it) }
    rawData?.let { putExtra(PushNotificationField.Data.key, it) }
}

internal fun Intent.hasNotificationPayload(): Boolean {
    return hasExtra(PushNotificationField.Type.key) || hasExtra(PushNotificationField.Data.key)
}
