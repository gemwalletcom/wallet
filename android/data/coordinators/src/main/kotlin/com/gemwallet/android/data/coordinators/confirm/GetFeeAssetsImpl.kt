package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.coordinators.FeeAssetProvider
import com.gemwallet.android.application.confirm.coordinators.GetFeeAssets
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOf

class GetFeeAssetsImpl(
    private val providers: Map<Chain, FeeAssetProvider>,
) : GetFeeAssets {

    override fun invoke(chain: Chain): Flow<List<AssetInfo>> = providers[chain]?.getFeeAssets() ?: flowOf(emptyList())
}
