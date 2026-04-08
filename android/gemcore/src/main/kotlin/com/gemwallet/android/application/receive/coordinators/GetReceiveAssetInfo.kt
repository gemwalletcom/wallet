package com.gemwallet.android.application.receive.coordinators

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetReceiveAssetInfo {
    fun getReceiveAssetInfo(assetId: AssetId): Flow<AssetInfo?>
}
