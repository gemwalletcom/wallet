package com.gemwallet.android.data.coordinators.add_asset

import com.gemwallet.android.application.add_asset.coordinators.SearchCustomToken
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.AssetId

class SearchCustomTokenImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
) : SearchCustomToken {

    override suspend fun invoke(assetId: AssetId): Boolean {
        val currency = sessionRepository.getCurrentCurrency()
        return assetsRepository.searchToken(assetId, currency)
    }
}
