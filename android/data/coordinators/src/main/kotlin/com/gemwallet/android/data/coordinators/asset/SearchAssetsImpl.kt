package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GemSearch
import com.gemwallet.android.application.assets.coordinators.SearchAssets
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.domains.search.apiTag
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SearchResponse
import uniffi.gemstone.GemAssetsService

class SearchAssetsImpl(
    private val assetsService: GemAssetsService,
) : SearchAssets, GemSearch {

    override suspend fun search(
        query: String,
        chains: List<Chain>,
        scope: WalletSearchTag,
    ): SearchResponse = assetsService.search(
        query = query,
        chains = chains.map { it.string },
        tags = listOfNotNull(scope.apiTag),
    ).decodeJson()

    override suspend fun searchAssets(
        query: String,
        chains: List<Chain>,
    ): List<AssetBasic> = assetsService.searchAssets(query, chains.map { it.string })
        .map { it.decodeJson<AssetBasic>() }

    override suspend fun getAssets(assetIds: List<AssetId>): List<AssetBasic> =
        assetsService.getAssets(assetIds.map { it.toIdentifier() }, null)
            .map { it.decodeJson<AssetBasic>() }
}
