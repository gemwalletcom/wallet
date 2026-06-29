package com.gemwallet.android.data.repositories.tokens

import com.gemwallet.android.application.assets.coordinators.GemSearch
import com.gemwallet.android.cases.tokens.SearchListTokens
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.service.store.database.AssetListDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toSearchRecord
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.domains.search.includesPerpetuals
import com.gemwallet.android.domains.search.isAll
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.AssetList
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualMetadata
import com.wallet.core.primitives.PerpetualSearchData
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class WalletSearchTokens(
    private val tokensRepository: TokensRepository,
    private val gemSearch: GemSearch,
    private val perpetualRepository: PerpetualRepository,
    private val searchDao: SearchDao,
    private val assetListDao: AssetListDao,
) : SearchTokensCase by tokensRepository, SearchListTokens {

    override suspend fun search(query: String, currency: Currency, chains: List<Chain>, tags: List<AssetTag>): Boolean {
        val scope = tags.firstOrNull()?.let { WalletSearchTag.Filter(it) } ?: WalletSearchTag.All
        return searchScope(query, currency, chains, scope)
    }

    override suspend fun searchList(listId: String, currency: Currency, chains: List<Chain>): Boolean =
        searchScope(query = "", currency = currency, chains = chains, scope = WalletSearchTag.List(listId))

    private suspend fun searchScope(query: String, currency: Currency, chains: List<Chain>, scope: WalletSearchTag): Boolean = withContext(Dispatchers.IO) {
        if (scope.isAll && query.isEmpty()) {
            return@withContext false
        }
        val response = runCatchingCancellable {
            gemSearch.search(query = query, chains = chains, scope = scope)
        }.getOrElse { return@withContext false }
        val key = scope.searchKey(query)
        tokensRepository.updateAssets(response.assets, currency)
        if (response.assets.isEmpty()) {
            searchDao.deleteAssets(key)
        } else {
            searchDao.put(response.assets.toSearchRecord(key))
        }
        val perpetuals = if (scope.includesPerpetuals) response.perpetuals else emptyList()
        storePerpetuals(perpetuals, key)
        if (scope.isAll) {
            storeLists(response.lists, key)
        }
        response.assets.isNotEmpty() || perpetuals.isNotEmpty()
    }

    private suspend fun storePerpetuals(perpetuals: List<PerpetualSearchData>, key: String) {
        if (perpetuals.isEmpty()) {
            return
        }
        runCatchingCancellable {
            perpetualRepository.putPerpetuals(
                perpetuals.map { PerpetualData(perpetual = it.perpetual, asset = it.asset, metadata = PerpetualMetadata(isPinned = false)) }
            )
            searchDao.put(perpetuals.toSearchRecord(key))
        }
    }

    private suspend fun storeLists(lists: List<AssetList>, key: String) {
        if (lists.isEmpty()) {
            searchDao.deleteLists(key)
            return
        }
        assetListDao.upsert(lists.toRecord())
        searchDao.put(lists.toSearchRecord(key))
    }
}

private fun WalletSearchTag.searchKey(query: String): String = when (this) {
    WalletSearchTag.All -> emptyList<AssetTag>().toPriorityQuery(query)
    is WalletSearchTag.Filter -> listOf(tag).toPriorityQuery(query)
    is WalletSearchTag.List -> listPriorityQuery(id)
}
