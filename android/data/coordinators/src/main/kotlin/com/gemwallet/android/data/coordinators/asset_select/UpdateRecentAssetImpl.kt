package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId

class UpdateRecentAssetImpl(
    private val assetsRepository: AssetsRepository,
) : UpdateRecentAsset {
    override suspend fun invoke(assetId: AssetId, walletId: String, type: RecentType) =
        assetsRepository.addRecentActivity(assetId, walletId, type)
}
