package com.gemwallet.android.application.asset_select.cases

import com.wallet.core.primitives.RecentActivityType
import com.wallet.core.primitives.AssetId

interface UpdateRecentAsset {
    suspend operator fun invoke(assetId: AssetId, type: RecentActivityType)
}
