package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.wallet.core.primitives.AssetId

class ToggleAssetPinImpl(
    private val assetsRepository: AssetsRepository,
) : ToggleAssetPin {
    override suspend fun invoke(walletId: String, assetId: AssetId) {
        assetsRepository.togglePin(walletId, assetId)
    }
}
