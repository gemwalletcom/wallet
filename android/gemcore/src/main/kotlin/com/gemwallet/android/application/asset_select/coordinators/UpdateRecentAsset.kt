package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId

interface UpdateRecentAsset {
    suspend operator fun invoke(assetId: AssetId, walletId: String, type: RecentType)
}
