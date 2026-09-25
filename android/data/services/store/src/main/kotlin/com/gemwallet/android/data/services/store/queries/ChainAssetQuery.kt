package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.ext.type
import com.gemwallet.android.model.ChainAssetInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class ChainAssetQuery @Inject constructor(private val assetQuery: AssetQueryOptional) {

    operator fun invoke(walletId: String, assetId: AssetId): Flow<ChainAssetInfo?> = when (assetId.type()) {
        AssetSubtype.NATIVE -> assetQuery(walletId, assetId).map { it?.let { info -> ChainAssetInfo(info, info) } }

        AssetSubtype.TOKEN -> combine(
            assetQuery(walletId, assetId),
            assetQuery(walletId, AssetId(assetId.chain)),
        ) { asset, fee ->
            if (asset == null || fee == null) null else ChainAssetInfo(asset, fee)
        }
    }
}
