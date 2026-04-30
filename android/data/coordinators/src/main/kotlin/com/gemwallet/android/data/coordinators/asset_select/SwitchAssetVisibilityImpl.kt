package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.wallet.core.primitives.AssetId

class SwitchAssetVisibilityImpl(
    private val enableAsset: EnableAsset,
    private val assetsRepository: AssetsRepository,
) : SwitchAssetVisibility {
    override suspend fun invoke(walletId: String, assetId: AssetId, visible: Boolean) {
        if (visible) {
            enableAsset(walletId, assetId)
        } else {
            assetsRepository.linkAssetToWallet(walletId, assetId, visible = false)
        }
    }
}
