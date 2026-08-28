package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.cases.FeeAssetProvider
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.hasAvailable
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import uniffi.gemstone.chainFeeAssetIds

class ChainFeeAssetProvider(
    private val chain: Chain,
    private val assetsRepository: AssetsRepository,
) : FeeAssetProvider {

    private val feeAssetIds = chainFeeAssetIds(chain.string).mapNotNull { it.toAssetId() }.toSet()

    override fun getFeeAssets(): Flow<List<AssetInfo>> = combine(
        assetsRepository.getAssetsInfoByChain(chain),
        assetsRepository.getHiddenAssetsInfoByChain(chain),
    ) { visible, hidden ->
        (visible + hidden)
            .filter { it.asset.id in feeAssetIds && it.balance.balance.hasAvailable() }
    }
}
