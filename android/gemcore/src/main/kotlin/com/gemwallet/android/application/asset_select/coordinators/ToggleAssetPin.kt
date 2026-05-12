package com.gemwallet.android.application.asset_select.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId

interface ToggleAssetPin {
    suspend operator fun invoke(walletId: WalletId, assetId: AssetId)
}
