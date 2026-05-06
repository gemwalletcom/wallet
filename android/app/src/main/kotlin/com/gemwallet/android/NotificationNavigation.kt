package com.gemwallet.android

import android.content.Intent
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.assets.coordinators.EnsureWalletAssets
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
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.gemwallet.android.ui.navigation.routes.SupportRoute
import com.gemwallet.android.ui.navigation.routes.SwapPairRoute
import com.gemwallet.android.ui.navigation.routes.TransactionDetailsRoute
import com.wallet.core.primitives.AssetId
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
        val payload = data ?: parseNotificationData(type, rawData = null) ?: return emptyList()
        if (!prepare(payload)) {
            return emptyList()
        }
        return payload.toRoutes()
    }

    private suspend fun prepare(data: PushNotificationData): Boolean {
        return when (data) {
            is PushNotificationData.Asset -> prepareAssets(data.assetId)
            is PushNotificationData.BuyAsset -> prepareAssets(data.assetId)
            is PushNotificationData.WalletAsset -> prepareWallet(data.walletId, listOf(data.assetId)) != null
            is PushNotificationData.Stake -> prepareWallet(data.walletId, listOf(data.assetId)) != null
            is PushNotificationData.Swap -> prepareAssets(data.fromAssetId, data.toAssetId)
            is PushNotificationData.Transaction -> prepareTransaction(data)
            PushNotificationData.Reward -> true
            PushNotificationData.Support -> true
        }
    }

    private suspend fun prepareAssets(vararg assetIds: AssetId): Boolean {
        prefetchAssets.prefetchAssets(assetIds.toList())
        return true
    }

    private suspend fun prepareTransaction(data: PushNotificationData.Transaction): Boolean {
        val assetIds = (data.transaction.getAssociatedAssetIds() + data.assetId).distinct()
        val wallet = prepareWallet(data.walletId, assetIds) ?: return false
        saveTransactions.saveTransactions(wallet.id, listOf(data.transaction))
        return true
    }

    private suspend fun prepareWallet(walletId: WalletId, assetIds: List<AssetId>): Wallet? {
        val wallet = walletsRepository.getWallet(walletId.id).firstOrNull() ?: return null
        prefetchAssets.prefetchAssets(assetIds)
        ensureWalletAssets.ensureWalletAssets(wallet, assetIds)
        if (sessionRepository.session().firstOrNull()?.wallet?.id != wallet.id) {
            sessionRepository.setWallet(wallet)
        }
        return wallet
    }
}

private fun PushNotificationData.toRoutes(): List<NavKey> {
    return when (this) {
        is PushNotificationData.Asset -> listOf(AssetRoute(assetId))
        is PushNotificationData.BuyAsset -> listOf(FiatInputRoute(assetId))
        is PushNotificationData.WalletAsset -> listOf(AssetRoute(assetId))
        is PushNotificationData.Stake -> listOf(AssetRoute(assetId))
        is PushNotificationData.Transaction -> listOf(AssetRoute(assetId), TransactionDetailsRoute(transaction.id))
        is PushNotificationData.Swap -> listOf(SwapPairRoute(fromAssetId, toAssetId))
        PushNotificationData.Reward -> listOf(ReferralRoute())
        PushNotificationData.Support -> listOf(SupportRoute)
    }
}

internal fun Intent.putNotificationPayload(type: String?, rawData: String?): Intent = apply {
    type?.let { putExtra(PushNotificationField.Type.key, it) }
    rawData?.let { putExtra(PushNotificationField.Data.key, it) }
}

internal fun Intent.hasNotificationPayload(): Boolean {
    return hasExtra(PushNotificationField.Type.key) || hasExtra(PushNotificationField.Data.key)
}
