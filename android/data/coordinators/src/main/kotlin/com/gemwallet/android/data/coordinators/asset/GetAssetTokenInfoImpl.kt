package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetAssetTokenInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

class GetAssetTokenInfoImpl(
    private val assetsRepository: AssetsRepository,
) : GetAssetTokenInfo {
    override fun invoke(assetId: AssetId): Flow<AssetInfo?> =
        assetsRepository.getTokenInfo(assetId)
}
