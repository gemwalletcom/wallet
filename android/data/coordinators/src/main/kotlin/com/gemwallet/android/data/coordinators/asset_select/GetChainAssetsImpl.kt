package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.GetChainAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

class GetChainAssetsImpl(
    private val assetsRepository: AssetsRepository,
) : GetChainAssets {
    override fun invoke(chain: Chain): Flow<List<AssetInfo>> = assetsRepository.getAssetsInfoByChain(chain)
}
