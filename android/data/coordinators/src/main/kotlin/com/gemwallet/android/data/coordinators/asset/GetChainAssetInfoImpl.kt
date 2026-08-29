package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.ext.type
import com.gemwallet.android.model.ChainAssetInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map

class GetChainAssetInfoImpl(
    private val getAssetTokenInfo: GetAssetTokenInfo,
) : GetChainAssetInfo {
    override fun invoke(assetId: AssetId): Flow<ChainAssetInfo?> {
        val assetInfo = getAssetTokenInfo(assetId)
        return when (assetId.type()) {
            AssetSubtype.NATIVE -> assetInfo.map { it?.let { ChainAssetInfo(it, it) } }
            AssetSubtype.TOKEN -> combine(
                assetInfo,
                getAssetTokenInfo(AssetId(assetId.chain)),
            ) { asset, fee ->
                if (asset == null || fee == null) null else ChainAssetInfo(asset, fee)
            }
        }
    }
}
