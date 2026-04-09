package com.gemwallet.android.data.coordinators.add_asset

import com.gemwallet.android.application.add_asset.coordinators.ObserveToken
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

class ObserveTokenImpl(
    private val assetsRepository: AssetsRepository,
) : ObserveToken {

    override suspend fun invoke(assetId: AssetId): Flow<Asset?> {
        return assetsRepository.getToken(assetId)
    }
}
