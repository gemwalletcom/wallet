package com.gemwallet.android

import android.content.Intent
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.assets.cases.SyncMissingAssets
import com.gemwallet.android.cases.parseNotificationData
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.PushNotificationData
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
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemAssetsService
import javax.inject.Inject

class NotificationNavigation @Inject constructor(
    private val getSession: GetSession,
    private val setCurrentWallet: SetCurrentWallet,
    private val getWallet: GetWallet,
    private val createTransaction: CreateTransaction,
    private val syncMissingAssets: SyncMissingAssets,
    private val assetsService: GemAssetsService,
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
            is PushNotificationData.WalletAsset -> prepareAssetRoutes(payload.walletId, payload.assetId)
            is PushNotificationData.Stake -> prepareAssetRoutes(payload.walletId, payload.assetId)
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
        syncMissingAssets.syncMissingAssets(assetIds.toList())
    }

    private suspend fun prepareAssetRoutes(walletId: WalletId, assetId: AssetId): List<NavKey> {
        val wallet = getWallet(walletId).firstOrNull() ?: return emptyList()
        val asset = assetsService.openWalletAsset(wallet.toJson(), assetId.toIdentifier())?.decodeJson<Asset>() ?: return emptyList()
        selectWallet(wallet)
        return listOf(AssetRoute(asset.id))
    }

    private suspend fun prepareTransactionRoutes(data: PushNotificationData.Transaction): List<NavKey> {
        val wallet = getWallet(data.walletId).firstOrNull() ?: return emptyList()
        val asset = createTransaction.createNotificationTransaction(
            wallet = wallet,
            assetId = data.assetId,
            transaction = data.transaction,
        ) ?: return emptyList()
        selectWallet(wallet)
        val transactionRoute = TransactionDetailsRoute(data.transaction.id)
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
