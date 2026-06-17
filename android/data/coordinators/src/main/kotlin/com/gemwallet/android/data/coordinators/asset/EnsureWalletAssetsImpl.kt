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

        val linked = assetsRepository.hasWalletAssets(wallet.id.id, requestedAssetIds)
        val unlinked = requestedAssetIds
            .filterNot(linked::contains)
            .filter { wallet.getAccount(it.chain) != null }
        if (unlinked.isEmpty()) {
            return
        }

        val existing = assetsRepository.hasAssets(unlinked)
        val toEnable = unlinked.filter(existing::contains)

        if (toEnable.isEmpty()) {
            return
        }

        enableAsset(wallet.id, toEnable)
    }
}
