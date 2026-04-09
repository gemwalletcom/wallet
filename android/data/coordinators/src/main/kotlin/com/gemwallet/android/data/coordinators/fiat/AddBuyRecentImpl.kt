package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.AddBuyRecent
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId

class AddBuyRecentImpl(
    private val assetsRepository: AssetsRepository,
) : AddBuyRecent {

    override suspend fun invoke(assetId: AssetId, walletId: String) {
        assetsRepository.addRecentActivity(assetId, walletId, RecentType.Buy)
    }
}
