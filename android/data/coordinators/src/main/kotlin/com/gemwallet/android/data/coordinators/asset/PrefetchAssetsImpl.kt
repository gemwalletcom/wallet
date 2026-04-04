package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet

class PrefetchAssetsImpl(
    private val gemApiClient: GemApiClient,
    private val assetsRepository: AssetsRepository,
) : PrefetchAssets {

    override suspend fun prefetchAssets(wallet: Wallet, assetIds: List<AssetId>) {
        val missingAssetIds = assetIds
            .distinct()
            .filterNot { assetsRepository.hasAsset(wallet.id, it) }

        if (missingAssetIds.isEmpty()) {
            return
        }

        val syncedAssetIds = loadAssets(missingAssetIds)
            .mapNotNull { storeAsset(wallet, it) }

        if (syncedAssetIds.isNotEmpty()) {
            assetsRepository.updateBalances(*syncedAssetIds.toTypedArray())
        }
    }

    private suspend fun loadAssets(assetIds: List<AssetId>): List<AssetBasic> {
        if (assetIds.isEmpty()) return emptyList()

        return runCatching { gemApiClient.getAssets(assetIds) }
            .getOrDefault(emptyList())
    }

    private suspend fun storeAsset(wallet: Wallet, asset: AssetBasic): AssetId? {
        val address = wallet.getAccount(asset.asset.id.chain)?.address ?: return null
        assetsRepository.add(wallet.id, address, asset, true)
        return asset.asset.id
    }
}
