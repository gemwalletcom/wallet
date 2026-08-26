package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.assets.coordinators.SyncAssetInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.stream.StreamSubscriptionService
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemAssetsService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class SyncAssetInfoImpl(
    private val assetsService: GemAssetsService,
    private val assetsRepository: AssetsRepository,
    private val streamSubscriptionService: StreamSubscriptionService,
    private val prefetchAssets: PrefetchAssets,
) : SyncAssetInfo {

    override suspend fun syncAssetInfo(assetId: AssetId, wallet: Wallet): Unit = withContext(Dispatchers.IO) {
        wallet.getAccount(assetId) ?: return@withContext

        streamSubscriptionService.addAssetIds(listOf(assetId))

        coroutineScope {
            launch {
                ensureWalletAsset(
                    walletId = wallet.id.id,
                    assetId = assetId,
                )
            }
            launch { assetsRepository.updateBalances(assetId) }
            launch {
                val assetFull = loadAssetMetadata(assetId) ?: return@launch
                assetsRepository.saveAssetMetadata(assetFull)
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

    private suspend fun loadAssetMetadata(assetId: AssetId) =
        runCatching { assetsService.getAsset(assetId.toIdentifier()).decodeJson<AssetFull>() }.getOrNull()
}
