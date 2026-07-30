package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

interface GetChainAssets {
    operator fun invoke(chain: Chain): Flow<List<AssetInfo>>

    fun hidden(chain: Chain): Flow<List<AssetInfo>>

    suspend fun updateBalances(chain: Chain)
}
