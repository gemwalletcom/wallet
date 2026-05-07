package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetAssetMarket
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetMarket
import kotlinx.coroutines.flow.Flow

class GetAssetMarketImpl(
    private val assetsRepository: AssetsRepository,
) : GetAssetMarket {
    override fun invoke(assetId: AssetId): Flow<AssetMarket?> =
        assetsRepository.getAssetMarket(assetId)
}
