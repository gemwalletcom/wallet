package com.gemwallet.android.data.coordinators.tokens

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.application.session.cases.GetSession
import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemSearchService

class SearchTokensImpl(
    private val getSession: GetSession,
    private val searchService: GemSearchService,
    private val assetsService: GemAssetsService,
) : SearchTokens {

    override suspend fun search(query: String, currency: Currency, chains: List<Chain>): Boolean = withContext(Dispatchers.IO) {
        if (query.isEmpty()) {
            return@withContext false
        }
        val wallet = getSession().value?.wallet ?: return@withContext false
        runCatchingCancellable { searchService.searchAssets(wallet.toJson(), query, currency.toGem()) }
            .getOrElse { return@withContext false }
            .isNotEmpty()
    }

    override suspend fun search(assetIds: List<AssetId>, currency: Currency): Boolean =
        runCatchingCancellable { assetsService.syncAssets(assetIds.map { it.toIdentifier() }, currency.toGem()) }
            .onFailure { Log.e(TAG, "assets sync failed", it) }
            .isSuccess

    private companion object {
        const val TAG = "SearchTokens"
    }
}
