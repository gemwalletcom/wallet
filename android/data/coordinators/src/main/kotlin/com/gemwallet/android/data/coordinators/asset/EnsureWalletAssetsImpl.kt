package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.application.assets.coordinators.EnsureWalletAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet

class EnsureWalletAssetsImpl(
    private val assetsRepository: AssetsRepository,
    private val enableAsset: EnableAsset,
) : EnsureWalletAssets {

    override suspend fun ensureWalletAssets(wallet: Wallet, assetIds: List<AssetId>) {
        val requestedAssetIds = assetIds.distinct()
        if (requestedAssetIds.isEmpty()) {
            return
        }

        val linked = assetsRepository.hasWalletAssets(wallet.id, requestedAssetIds)
        val missing = requestedAssetIds
            .filterNot(linked::contains)
            .filter { wallet.getAccount(it.chain) != null }

        if (missing.isEmpty()) {
            return
        }

        enableAsset(wallet.id, missing)
    }
}
