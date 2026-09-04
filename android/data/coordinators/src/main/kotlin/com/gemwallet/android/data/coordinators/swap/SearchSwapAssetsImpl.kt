package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.cases.SearchSwapAssets
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.domains.asset.eligible
import com.gemwallet.android.domains.asset.queryFilters
import uniffi.gemstone.GemAssetAction
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemSwapServiceInterface
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.flowOn

@OptIn(ExperimentalCoroutinesApi::class)
class SearchSwapAssetsImpl(
    private val searchService: AssetsSearchService,
    private val swapService: GemSwapServiceInterface,
) : SearchSwapAssets {

    override fun invoke(
        wallet: Wallet?,
        query: String,
        swapItemType: SwapItemType,
        oppositeAssetId: AssetId?,
    ): Flow<List<AssetInfo>> {
        if (wallet == null) {
            return emptyFlow()
        }
        val action = when (swapItemType) {
            SwapItemType.Pay -> GemAssetAction.SWAP_PAY
            SwapItemType.Receive -> GemAssetAction.SWAP_RECEIVE
        }
        val items = if (oppositeAssetId == null) {
            searchService.search(query, byAllWallets = false, filters = action.queryFilters())
        } else {
            flow { emit(swapService.supportedAssets(oppositeAssetId.toIdentifier())) }
                .flatMapLatest { supported ->
                    searchService.swapSearch(
                        wallet,
                        query,
                        supported.chains.map { it.requireChain() },
                        supported.assetIds.mapNotNull { it.toAssetId() },
                    )
                }
        }
        return items
            .catch { emit(emptyList()) }
            .map { action.eligible(it) }
            .flowOn(Dispatchers.IO)
    }
}
