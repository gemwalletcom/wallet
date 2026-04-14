package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.EnsureWalletAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.firstOrNull

class EnsureWalletAssetsImpl(
    private val assetsRepository: AssetsRepository,
) : EnsureWalletAssets {

    override suspend fun ensureWalletAssets(wallet: Wallet, assetIds: List<AssetId>) {
        val requestedAssetIds = assetIds.distinct()
        if (requestedAssetIds.isEmpty()) {
            return
        }

        val missingAssetIds = requestedAssetIds.filterNot(
            assetsRepository.hasWalletAssets(wallet.id, requestedAssetIds)::contains
        )

        if (missingAssetIds.isEmpty()) {
            return
        }

        val assets = assetsRepository.getTokensInfo(missingAssetIds.map { it.toIdentifier() })
            .firstOrNull()
            ?.associateBy { it.asset.id }
            ?: emptyMap()

        val syncedAssetIds = missingAssetIds.mapNotNull { assetId ->
            val asset = assets[assetId]?.asset ?: return@mapNotNull null
            val accountAddress = wallet.getAccount(asset.id.chain)?.address ?: return@mapNotNull null

            assetsRepository.add(
                walletId = wallet.id,
                accountAddress = accountAddress,
                asset = asset,
                visible = true,
            )
            asset.id
        }

        if (syncedAssetIds.isNotEmpty()) {
            assetsRepository.updateBalances(*syncedAssetIds.toTypedArray())
        }
    }
}
