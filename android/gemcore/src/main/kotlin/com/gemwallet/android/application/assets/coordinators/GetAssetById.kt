package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetAssetById {
    operator fun invoke(assetId: AssetId): Flow<Asset?>
}
