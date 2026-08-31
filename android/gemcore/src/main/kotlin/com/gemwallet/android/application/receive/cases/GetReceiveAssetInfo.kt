package com.gemwallet.android.application.receive.cases

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetReceiveAssetInfo {
    operator fun invoke(assetId: AssetId): Flow<AssetInfo?>
}
