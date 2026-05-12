package com.gemwallet.android.application.assets.coordinators

import com.gemwallet.android.model.ChainAssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetChainAssetInfo {
    operator fun invoke(assetId: AssetId): Flow<ChainAssetInfo?>
}
