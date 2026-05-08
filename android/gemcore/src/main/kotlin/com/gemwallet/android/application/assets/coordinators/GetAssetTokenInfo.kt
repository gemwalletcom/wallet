package com.gemwallet.android.application.assets.coordinators

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetAssetTokenInfo {
    operator fun invoke(assetId: AssetId): Flow<AssetInfo?>
}
