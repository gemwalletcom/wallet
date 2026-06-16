package com.gemwallet.android.data.repositories.tokens

import com.gemwallet.android.application.assets.coordinators.GemSearch
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.entities.toSearchRecord
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualMetadata
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class WalletSearchTokens(
    private val tokensRepository: TokensRepository,
    private val gemSearch: GemSearch,
    private val perpetualRepository: PerpetualRepository,
    private val searchDao: SearchDao,
) : SearchTokensCase by tokensRepository {

    override suspend fun search(query: String, currency: Currency, chains: List<Chain>, tags: List<AssetTag>): Boolean = withContext(Dispatchers.IO) {
        if (query.isEmpty() && tags.isEmpty()) {
            return@withContext false
        }
        val response = runCatchingCancellable {
            gemSearch.search(query = query, chains = chains, tags = tags)
        }.getOrElse { return@withContext false }
        val priorityQuery = tags.toPriorityQuery(query)
        tokensRepository.updateAssets(response.assets, currency)
        if (response.assets.isEmpty()) {
            searchDao.deleteAssets(priorityQuery)
        } else {
            searchDao.put(response.assets.toSearchRecord(priorityQuery))
        }
        val hasAssets = response.assets.isNotEmpty()
        val perpetuals = if (tags.isEmpty()) response.perpetuals else emptyList()
        if (perpetuals.isNotEmpty()) {
            runCatchingCancellable {
                perpetualRepository.putPerpetuals(
                    perpetuals.map { PerpetualData(perpetual = it.perpetual, asset = it.asset, metadata = PerpetualMetadata(isPinned = false)) }
                )
                searchDao.put(perpetuals.toSearchRecord(priorityQuery))
            }
        }
        hasAssets || perpetuals.isNotEmpty()
    }
}
