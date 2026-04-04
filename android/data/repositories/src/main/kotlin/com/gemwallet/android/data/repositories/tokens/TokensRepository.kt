package com.gemwallet.android.data.repositories.tokens

import com.gemwallet.android.blockchain.services.TokenService
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsPriorityDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toPriceRecord
import com.gemwallet.android.data.service.store.database.entities.toRecordPriority
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetProperties
import com.wallet.core.primitives.AssetScore
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
    private val gemApiClient: GemApiClient,
    private val tokenService: TokenService,
) : SearchTokensCase {

    override suspend fun search(query: String, currency: Currency, chains: List<Chain>, tags: List<AssetTag>): Boolean = withContext(Dispatchers.IO) {
        if (query.isEmpty() && tags.isEmpty()) {
            return@withContext false
        }
        val tagsQuery = tags.toGemQuery()
        val tokens = try {
            val chainsQuery = if (chains.isEmpty()) {
                ""
            } else {
                chains.joinToString(",") { it.string }
            }
            gemApiClient.search(query, chainsQuery, tagsQuery)
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
        val result = gemApiClient.getAssets(assetIds)
        updateAssets(result, currency)
        return true
    }

    override suspend fun search(assetId: AssetId, currency: Currency): Boolean {
        val tokenId = assetId.tokenId ?: return false
        val asset = tokenService.getTokenData(assetId) ?: return search(tokenId, currency)
        val record = AssetBasic(asset = asset, score = AssetScore(0), properties = AssetProperties(
            isEnabled = false,
            isBuyable = false,
            isSellable = false,
            isSwapable = false,
            isStakeable = false,
            isEarnable = false,
            hasImage = true,
        ))
        .toRecord()
        runCatching { assetsDao.insert(record) }
        return true
    }

    private suspend fun updateAssets(assets: List<AssetBasic>, currency: Currency) {
        runCatching { assetsDao.insert(assets.map { it.toRecord() }) }
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
