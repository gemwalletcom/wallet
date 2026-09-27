package com.gemwallet.android.data.services.store.queries

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAssetData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class ChainAssetQuery @Inject constructor(private val assetQuery: AssetQueryOptional) {

    operator fun invoke(walletId: String, assetId: AssetId, feeAssetId: AssetId): Flow<ChainAssetData?> = when (feeAssetId == assetId) {
        true -> assetQuery(walletId, assetId).map { it?.let { info -> ChainAssetData(info, info) } }

        false -> combine(
            assetQuery(walletId, assetId),
            assetQuery(walletId, feeAssetId),
        ) { asset, fee ->
            if (asset == null || fee == null) null else ChainAssetData(asset, fee)
        }
    }
}
