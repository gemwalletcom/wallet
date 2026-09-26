package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.ext.type
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.ChainAssetData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class ChainAssetQuery @Inject constructor(private val assetQuery: AssetQueryOptional) {

    operator fun invoke(walletId: String, assetId: AssetId): Flow<ChainAssetData?> = when (assetId.type()) {
        AssetSubtype.NATIVE -> assetQuery(walletId, assetId).map { it?.let { info -> ChainAssetData(info, info) } }

        AssetSubtype.TOKEN -> combine(
            assetQuery(walletId, assetId),
            assetQuery(walletId, AssetId(assetId.chain)),
        ) { asset, fee ->
            if (asset == null || fee == null) null else ChainAssetData(asset, fee)
        }
    }
}
