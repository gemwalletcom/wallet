package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetAssetLinks
import com.gemwallet.android.data.adapters.gemstone.GemstoneAssetStore
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import kotlinx.coroutines.flow.Flow

class GetAssetLinksImpl(
    private val assetStore: GemstoneAssetStore,
) : GetAssetLinks {
    override fun invoke(assetId: AssetId): Flow<List<AssetLink>> =
        assetStore.observeAssetLinks(assetId)
}
