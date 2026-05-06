package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetAssetLinks
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import kotlinx.coroutines.flow.Flow

class GetAssetLinksImpl(
    private val assetsRepository: AssetsRepository,
) : GetAssetLinks {
    override fun invoke(assetId: AssetId): Flow<List<AssetLink>> =
        assetsRepository.getAssetLinks(assetId)
}
