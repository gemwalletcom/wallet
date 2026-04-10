package com.gemwallet.android.application.asset_select.coordinators

import com.wallet.core.primitives.AssetId

interface ToggleAssetPin {
    suspend fun togglePin(walletId: String, assetId: AssetId)
}
