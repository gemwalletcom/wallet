package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetAssetMarket
import com.gemwallet.android.data.repositories.gemstone.GemstoneAssetStore
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetMarket
import kotlinx.coroutines.flow.Flow

class GetAssetMarketImpl(
    private val assetStore: GemstoneAssetStore,
) : GetAssetMarket {
    override fun invoke(assetId: AssetId): Flow<AssetMarket?> =
        assetStore.observeAssetMarket(assetId)
}
