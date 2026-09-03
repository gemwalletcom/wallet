package com.gemwallet.android.application.asset_select.cases

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

interface GetChainAssets {
    operator fun invoke(chain: Chain): Flow<List<AssetInfo>>

    fun hidden(chain: Chain): Flow<List<AssetInfo>>
}
