package com.gemwallet.android.data.repositories.tokens

import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.cases.tokens.SyncAssetPrices
import com.gemwallet.android.data.repositories.prices.PricesRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toUpdateRecord
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemSearchService

class TokensRepository(
    private val assetsDao: AssetsDao,
    private val pricesDao: PricesDao,
    private val pricesRepository: PricesRepository,
    private val sessionRepository: SessionRepository,
    private val searchService: GemSearchService,
    private val assetsService: GemAssetsService,
) : SearchTokensCase, SyncAssetPrices {

    override suspend fun search(query: String, currency: Currency, chains: List<Chain>): Boolean = withContext(Dispatchers.IO) {
        if (query.isEmpty()) {
            return@withContext false
        }
        val wallet = sessionRepository.session().value?.wallet ?: return@withContext false
        runCatchingCancellable { searchService.searchAssets(wallet.toJson(), query, currency.toJson()) }
            .getOrElse { return@withContext false }
            .isNotEmpty()
    }

    override suspend fun search(assetIds: List<AssetId>, currency: Currency): Boolean {
        updateAssets(assets(assetIds), currency)
        return true
    }

    override suspend fun search(assetId: AssetId, currency: Currency): Boolean {
        val tokenId = assetId.tokenId ?: return false
        return runCatchingCancellable { assetsService.getOrFetchTokenAsset(assetId.toIdentifier()) }
            .map { true }
            .getOrElse { search(tokenId, currency) }
    }

    override suspend fun invoke(assetIds: List<AssetId>, currency: Currency) = withContext(Dispatchers.IO) {
        val unique = assetIds.distinct()
        if (unique.isEmpty()) return@withContext
        val priced = pricesDao.getByAssets(unique.map { it.toIdentifier() })
            .map { it.assetId }
            .toSet()
        val missing = unique.filter { it.toIdentifier() !in priced }
        if (missing.isEmpty()) return@withContext
        runCatching {
            updateAssets(assets(missing), currency)
        }
        Unit
    }

    private suspend fun assets(assetIds: List<AssetId>): List<AssetBasic> =
        assetsService.getAssets(assetIds.map { it.toIdentifier() }, null).map { it.decodeJson<AssetBasic>() }

    internal suspend fun updateAssets(assets: List<AssetBasic>, currency: Currency) {
        if (assets.isEmpty()) {
            return
        }
        runCatching {
            assetsDao.insert(assets.map { it.toRecord() })
            assetsDao.updateBasicAssets(assets.map { it.toUpdateRecord() })
        }
        runCatching {
            pricesRepository.updatePrices(assets, currency)
        }
    }
}

fun listPriorityQuery(listId: String) = "tag:$listId"
