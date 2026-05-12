package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.coordinators.GetSwapSupported
import com.gemwallet.android.application.swap.coordinators.SearchSwapAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.isSwapSupport
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.hasAvailable
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.SwapperAssetList

@OptIn(ExperimentalCoroutinesApi::class)
class SearchSwapAssetsImpl(
    private val assetsRepository: AssetsRepository,
    private val getSwapSupported: GetSwapSupported,
) : SearchSwapAssets {

    override fun invoke(
        wallet: Wallet?,
        query: String,
        swapItemType: SwapItemType,
        oppositeAssetId: AssetId?,
        tag: AssetTag?,
    ): Flow<List<AssetInfo>> {
        if (wallet == null) {
            return emptyFlow()
        }
        return flow {
            if (oppositeAssetId == null) {
                val chains = wallet.accounts.map { it.chain }.filter { it.isSwapSupport() }
                val chainNames = chains.map { it.string }
                emit(SwapperAssetList(chainNames, emptyList()))
                val assetIds = chains.flatMap { getSwapSupported.getSwapSupportChains(AssetId(it)).assetIds }
                emit(SwapperAssetList(chainNames, assetIds))
            } else {
                emit(getSwapSupported.getSwapSupportChains(oppositeAssetId))
            }
        }
        .flatMapLatest { supported ->
            assetsRepository.swapSearch(
                wallet,
                query,
                supported.chains.mapNotNull { it.toChain() },
                supported.assetIds.mapNotNull { it.toAssetId() },
                tag?.let { listOf(it) } ?: emptyList(),
            )
        }
        .catch { emit(emptyList()) }
        .map { items ->
            items.filter { assetInfo ->
                assetInfo.metadata?.isSwapEnabled == true &&
                    if (swapItemType == SwapItemType.Pay) {
                        assetInfo.balance.balance.hasAvailable()
                    } else {
                        true
                    }
            }
        }
    }
}
