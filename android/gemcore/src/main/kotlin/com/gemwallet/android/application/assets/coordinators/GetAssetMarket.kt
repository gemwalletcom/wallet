package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetMarket
import kotlinx.coroutines.flow.Flow

interface GetAssetMarket {
    operator fun invoke(assetId: AssetId): Flow<AssetMarket?>
}
