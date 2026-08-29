package com.gemwallet.android.data.coordinators.add_asset

import com.gemwallet.android.application.add_asset.cases.SearchCustomToken
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.AssetId

class SearchCustomTokenImpl(
    private val sessionRepository: SessionRepository,
    private val searchTokensCase: SearchTokensCase,
) : SearchCustomToken {

    override suspend fun invoke(assetId: AssetId): Boolean {
        val currency = sessionRepository.getCurrentCurrency()
        return searchTokensCase.search(assetId, currency)
    }
}
