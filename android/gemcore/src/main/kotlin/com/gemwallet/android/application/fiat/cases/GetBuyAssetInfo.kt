package com.gemwallet.android.application.fiat.cases

import com.gemwallet.android.model.AssetData
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetBuyAssetInfo {
    operator fun invoke(assetId: AssetId): Flow<AssetData?>
}
