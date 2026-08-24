package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.coordinators.FeeAssetProvider
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.asset.defaultAssets
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.hasAvailable
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

class TempoFeeAssetProvider(
    private val assetsRepository: AssetsRepository,
) : FeeAssetProvider {

    private val chain = Chain.Tempo
    private val supportedAssetIds = chain.defaultAssets.map { it.id }.toSet()

    override fun getFeeAssets(): Flow<List<AssetInfo>> = combine(
        assetsRepository.getAssetsInfoByChain(chain),
        assetsRepository.getHiddenAssetsInfoByChain(chain),
    ) { visible, hidden ->
        (visible + hidden)
            .filter { it.asset.id in supportedAssetIds && it.balance.balance.hasAvailable() }
    }
}
