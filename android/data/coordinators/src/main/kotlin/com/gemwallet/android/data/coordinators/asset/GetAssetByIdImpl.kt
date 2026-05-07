package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetAssetById
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

class GetAssetByIdImpl(
    private val assetsRepository: AssetsRepository,
) : GetAssetById {
    override fun invoke(assetId: AssetId): Flow<Asset?> =
        assetsRepository.asset(assetId)
}
