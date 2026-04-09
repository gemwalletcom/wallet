package com.gemwallet.android.application.add_asset.coordinators

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface ObserveToken {
    suspend operator fun invoke(assetId: AssetId): Flow<Asset?>
}
