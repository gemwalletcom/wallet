package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.cases.tokens.SyncAssetPrices
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.firstOrNull

class EnableAssetImpl(
    private val sessionRepository: SessionRepository,
    private val syncAssetPrices: SyncAssetPrices,
    private val assetsRepository: AssetsRepository,
) : EnableAsset {

    override suspend fun invoke(walletId: String, assetId: AssetId) =
        invoke(walletId, listOf(assetId))

    override suspend fun invoke(walletId: String, assetIds: List<AssetId>) {
        val unique = assetIds.distinct()
        if (unique.isEmpty()) return

        val enabled = assetsRepository.getAssetsInfo(unique)
            .firstOrNull()
            .orEmpty()
            .filter { it.walletId == walletId && it.metadata?.isBalanceEnabled == true }
            .mapTo(hashSetOf()) { it.asset.id }

        val missing = unique - enabled
        if (missing.isEmpty()) return

        syncAssetPrices(missing, sessionRepository.getCurrentCurrency())
        missing.forEach { assetsRepository.linkAssetToWallet(walletId, it, visible = true) }
        assetsRepository.updateBalances(*missing.toTypedArray())
    }
}
