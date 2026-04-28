package com.gemwallet.android.application.asset_select.coordinators

import com.wallet.core.primitives.AssetId

interface SwitchAssetVisibility {
    suspend operator fun invoke(walletId: String, assetId: AssetId, visible: Boolean)
}
