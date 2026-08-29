package com.gemwallet.android.data.coordinators.tokens

import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.application.tokens.cases.WalletSearchScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemSearchScope
import uniffi.gemstone.GemSearchService

class WalletSearchTokens(
    private val searchTokens: SearchTokensImpl,
    private val searchService: GemSearchService,
    private val getSession: GetSession,
) : SearchTokens by searchTokens, WalletSearchScope {

    override suspend fun search(query: String, currency: Currency, chains: List<Chain>): Boolean =
        searchScope(query, currency, WalletSearchTag.All)

    override suspend fun search(query: String, currency: Currency, chains: List<Chain>, scope: WalletSearchTag): Boolean =
        searchScope(query, currency, scope)

    private suspend fun searchScope(query: String, currency: Currency, scope: WalletSearchTag): Boolean = withContext(Dispatchers.IO) {
        val wallet = getSession().value?.wallet ?: return@withContext false
        runCatchingCancellable {
            searchService.search(wallet.toJson(), query, scope.toGem(), currency.toJson())
        }.getOrElse { false }
    }
}

private fun WalletSearchTag.toGem(): GemSearchScope = when (this) {
    WalletSearchTag.All -> GemSearchScope.All
    is WalletSearchTag.List -> GemSearchScope.List(id)
}
