package com.gemwallet.android.application.assets.cases

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetAssetInfo {
    operator fun invoke(assetId: AssetId): Flow<AssetInfo?>
}
