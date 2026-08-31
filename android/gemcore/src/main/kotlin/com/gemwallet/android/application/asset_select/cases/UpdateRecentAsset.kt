package com.gemwallet.android.application.asset_select.cases

import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId

interface UpdateRecentAsset {
    suspend operator fun invoke(assetId: AssetId, type: RecentType)
}
