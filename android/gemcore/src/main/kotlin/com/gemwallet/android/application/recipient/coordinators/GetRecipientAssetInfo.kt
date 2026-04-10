package com.gemwallet.android.application.recipient.coordinators

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetRecipientAssetInfo {
    fun getAssetInfo(assetId: AssetId): Flow<AssetInfo?>
}
