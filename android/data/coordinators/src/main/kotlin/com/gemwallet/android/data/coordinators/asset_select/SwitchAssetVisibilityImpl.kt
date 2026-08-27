package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId

class SwitchAssetVisibilityImpl(
    private val enableAsset: EnableAsset,
) : SwitchAssetVisibility {
    override suspend fun invoke(walletId: WalletId, assetId: AssetId, visible: Boolean) {
        enableAsset(walletId, assetId, visible)
    }
}
