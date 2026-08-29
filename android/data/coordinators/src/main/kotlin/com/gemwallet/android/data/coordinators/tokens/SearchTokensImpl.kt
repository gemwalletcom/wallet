package com.gemwallet.android.data.coordinators.tokens

import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.session.SessionRepository
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
    private val sessionRepository: SessionRepository,
    private val searchService: GemSearchService,
    private val assetsService: GemAssetsService,
) : SearchTokensCase {

    override suspend fun search(query: String, currency: Currency, chains: List<Chain>): Boolean = withContext(Dispatchers.IO) {
        if (query.isEmpty()) {
            return@withContext false
        }
        val wallet = sessionRepository.session().value?.wallet ?: return@withContext false
        runCatchingCancellable { searchService.searchAssets(wallet.toJson(), query, currency.toJson()) }
            .getOrElse { return@withContext false }
            .isNotEmpty()
    }

    override suspend fun search(assetIds: List<AssetId>, currency: Currency): Boolean =
        runCatchingCancellable { assetsService.syncAssets(assetIds.map { it.toIdentifier() }, currency.toJson()) }
            .onFailure { Log.e(TAG, "assets sync failed", it) }
            .isSuccess

    override suspend fun search(assetId: AssetId, currency: Currency): Boolean =
        runCatchingCancellable { assetsService.ensureTokenAsset(assetId.toIdentifier()) }
            .onFailure { Log.e(TAG, "token asset lookup failed", it) }
            .isSuccess

    private companion object {
        const val TAG = "SearchTokens"
    }
}
