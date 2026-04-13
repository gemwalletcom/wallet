package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AssetId

class SwitchAssetVisibilityImpl(
    private val assetsRepository: AssetsRepository,
) : SwitchAssetVisibility {
    override suspend fun invoke(walletId: String, account: Account, assetId: AssetId, visible: Boolean) =
        assetsRepository.switchVisibility(walletId, account, assetId, visible)
}
