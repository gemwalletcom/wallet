package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.cases.GetChainAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first

class GetChainAssetsImpl(
    private val assetsRepository: AssetsRepository,
) : GetChainAssets {
    override fun invoke(chain: Chain): Flow<List<AssetInfo>> = assetsRepository.getAssetsInfoByChain(chain)

    override fun hidden(chain: Chain): Flow<List<AssetInfo>> = assetsRepository.getHiddenAssetsInfoByChain(chain)

    override suspend fun updateBalances(chain: Chain) {
        val assets = assetsRepository.getAssetsInfoByChain(chain).first() +
            assetsRepository.getHiddenAssetsInfoByChain(chain).first()
        assetsRepository.updateBalances(assets)
    }
}
