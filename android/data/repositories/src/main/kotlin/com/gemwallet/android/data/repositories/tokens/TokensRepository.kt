package com.gemwallet.android.data.repositories.tokens

import com.gemwallet.android.application.assets.coordinators.SearchAssets
import com.gemwallet.android.blockchain.services.TokenService
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.cases.tokens.SyncAssetPrices
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsPriorityDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toPriceRecord
import com.gemwallet.android.data.service.store.database.entities.toRecordPriority
import com.gemwallet.android.data.service.store.database.entities.toUpdateRecord
import com.gemwallet.android.domains.asset.defaultBasic
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext

class TokensRepository (
    private val assetsDao: AssetsDao,
    private val pricesDao: PricesDao,
    private val assetsPriorityDao: AssetsPriorityDao,
    private val searchAssets: SearchAssets,
    private val tokenService: TokenService,
) : SearchTokensCase, SyncAssetPrices {

    override suspend fun search(query: String, currency: Currency, chains: List<Chain>, tags: List<AssetTag>): Boolean = withContext(Dispatchers.IO) {
        if (query.isEmpty() && tags.isEmpty()) {
            return@withContext false
        }
        val tokens = try {
            searchAssets.search(
                query = query,
                chains = chains,
                tags = tags,
            )
        } catch (_: Throwable) {
            return@withContext false
        }
        val assets = if (tokens.isEmpty()) {
            val assets = tokenService.search(query)
            runCatching { assetsDao.insert(assets.map { it.toRecord() }) }
            assets
        } else {
            updateAssets(tokens, currency)
            assetsPriorityDao.put(tokens.toRecordPriority(tags.toPriorityQuery(query)))
            tokens
        }
        assets.isNotEmpty()
    }

    override suspend fun search(assetIds: List<AssetId>, currency: Currency): Boolean {
        val result = searchAssets.getAssets(assetIds)
        updateAssets(result, currency)
        return true
    }

    override suspend fun search(assetId: AssetId, currency: Currency): Boolean {
        val tokenId = assetId.tokenId ?: return false
        val asset = tokenService.getTokenData(assetId) ?: return search(tokenId, currency)
        runCatching { assetsDao.insert(asset.defaultBasic.toRecord()) }
        return true
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
            val assets = searchAssets.getAssets(missing)
            updateAssets(assets, currency)
        }
        Unit
    }

    private suspend fun updateAssets(assets: List<AssetBasic>, currency: Currency) {
        if (assets.isEmpty()) {
            return
        }
        runCatching {
            assetsDao.insert(assets.map { it.toRecord() })
            assetsDao.updateBasicAssets(assets.map { it.toUpdateRecord() })
        }
        runCatching {
            val rate = pricesDao.getRates(currency).firstOrNull() ?: return@runCatching
            val prices = assets.toPriceRecord(rate.toDTO())

            if (prices.isNotEmpty()) {
                pricesDao.insert(prices)
            }
        }
    }
}

private fun List<AssetTag>.toGemQuery() = if (isEmpty()) {
    ""
} else {
    joinToString(",") { it.string }
}

fun List<AssetTag>.toPriorityQuery(query: String) = if (isEmpty()) query.trim() else "${query.trim()}::${toGemQuery()}"
