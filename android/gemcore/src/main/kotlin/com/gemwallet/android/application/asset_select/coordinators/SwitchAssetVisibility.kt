package com.gemwallet.android.application.asset_select.coordinators

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AssetId

interface SwitchAssetVisibility {
    suspend operator fun invoke(walletId: String, account: Account, assetId: AssetId, visible: Boolean)
}
