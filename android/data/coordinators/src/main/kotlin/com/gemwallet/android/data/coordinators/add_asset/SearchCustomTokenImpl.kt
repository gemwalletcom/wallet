package com.gemwallet.android.data.coordinators.add_asset

import com.gemwallet.android.application.add_asset.cases.SearchCustomToken
import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.wallet.core.primitives.AssetId

class SearchCustomTokenImpl(
    private val getCurrentCurrency: GetCurrentCurrency,
    private val searchTokensCase: SearchTokens,
) : SearchCustomToken {

    override suspend fun invoke(assetId: AssetId): Boolean {
        val currency = getCurrentCurrency.getCurrentCurrency()
        return searchTokensCase.search(assetId, currency)
    }
}
