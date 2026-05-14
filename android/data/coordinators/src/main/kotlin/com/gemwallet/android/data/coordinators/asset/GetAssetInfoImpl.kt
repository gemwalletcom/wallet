package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

class GetAssetInfoImpl(
    private val assetsRepository: AssetsRepository,
) : GetAssetInfo {
    override fun invoke(assetId: AssetId): Flow<AssetInfo?> = assetsRepository.getAssetInfo(assetId)
}
