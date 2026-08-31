package com.gemwallet.android.application.assets.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import kotlinx.coroutines.flow.Flow

interface GetAssetLinks {
    operator fun invoke(assetId: AssetId): Flow<List<AssetLink>>
}
