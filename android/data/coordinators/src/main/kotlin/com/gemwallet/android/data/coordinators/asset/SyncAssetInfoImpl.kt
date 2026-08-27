package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.assets.coordinators.SyncAssetInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemAssetsService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemStreamSubscriptionService

class SyncAssetInfoImpl(
    private val assetsService: GemAssetsService,
    private val assetsRepository: AssetsRepository,
    private val streamSubscriptionService: GemStreamSubscriptionService,
    private val prefetchAssets: PrefetchAssets,
    private val sessionRepository: SessionRepository,
) : SyncAssetInfo {

    override suspend fun syncAssetInfo(assetId: AssetId, wallet: Wallet): Unit = withContext(Dispatchers.IO) {
        wallet.getAccount(assetId) ?: return@withContext

        streamSubscriptionService.addPrices(listOf(assetId.toIdentifier()))

        coroutineScope {
            launch {
                ensureWalletAsset(
                    walletId = wallet.id.id,
                    assetId = assetId,
                )
            }
            launch { assetsRepository.updateBalances(assetId) }
            launch {
                val assetFull = syncAssetMetadata(assetId) ?: return@launch
                prefetchAssets.prefetchAssets(assetFull.associations.map { it.assetId })
            }
        }
    }

    private suspend fun ensureWalletAsset(
        walletId: String,
        assetId: AssetId,
    ) = assetsRepository.getAssetInfo(assetId).firstOrNull()
        ?: assetsRepository.getTokenInfo(assetId).firstOrNull()?.also { asset ->
            assetsRepository.linkAssetToWallet(
                walletId = walletId,
                assetId = asset.asset.id,
                visible = asset.metadata?.isBalanceEnabled ?: true,
            )
        }

    private suspend fun syncAssetMetadata(assetId: AssetId) = runCatching {
        assetsService.syncAsset(assetId.toIdentifier(), sessionRepository.getCurrentCurrency().toJson()).decodeJson<AssetFull>()
    }.getOrNull()
}
