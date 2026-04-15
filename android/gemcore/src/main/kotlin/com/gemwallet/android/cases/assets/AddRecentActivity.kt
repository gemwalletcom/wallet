package com.gemwallet.android.cases.assets

import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId

interface AddRecentActivity {
    suspend fun addRecentActivity(
        assetId: AssetId,
        walletId: String,
        type: RecentType,
        toAssetId: AssetId? = null,
    )
}